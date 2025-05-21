use std::path::Path;

use base64::prelude::*;
use enum_iterator::all;
use file_format::FileFormat;
use thiserror::Error;

use crate::google::{
    GoogleModel,
    common::{Blob, Content, HarmCategory, Modality, Part, Role},
    request::{GenerateContentRequest, GenerationConfig, HarmBlockThreshold, SafetySettings},
    response::ContentResponse,
};

const URL_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const URL_EXTENSION: &str = ":streamGenerateContent";

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Request(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Wrapper struct which stores the HTTP Reqwest client and the request history.  The `send`
/// methods are used to send text and images without having to manage the history manually.
#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    model: GoogleModel,
    key: String,
    request: GenerateContentRequest,
}

/// The model may return more than one output since we use streaming.  This wrapper
/// is used as a helper to consolidate the outputs.
#[derive(Debug)]
pub struct Responses(Vec<ContentResponse>);

impl Responses {
    pub fn inner(&self) -> &[ContentResponse] {
        &self.0
    }
}

impl Responses {
    /// Squash multiple text responses into a single string.
    pub fn text(&self) -> Option<String> {
        let mut text = String::new();
        for content in &self.0 {
            for candidate in &content.candidates {
                for part in &candidate.content.parts {
                    if let Part::Text(txt) = part {
                        text += txt
                    }
                }
            }
        }
        if text.is_empty() { None } else { Some(text) }
    }

    /// Helper to extract the image mime types and Base64 encoded data.
    pub fn images(&self) -> Vec<(String, String)> {
        let mut images = Vec::new();
        for content in &self.0 {
            for candidate in &content.candidates {
                for part in &candidate.content.parts {
                    if let Part::InlineData(blob) = part {
                        images.push((blob.mime_type.clone(), blob.data.clone()));
                    }
                }
            }
        }

        images
    }
}

impl Client {
    /// Creates a new instance of a Reqwest client.  The client is setup to utilize the given
    /// Google Gemini model.
    pub fn new(model: &GoogleModel, key: &str) -> Self {
        Client {
            client: reqwest::Client::new(),
            model: model.clone(),
            key: key.to_string(),
            request: GenerateContentRequest {
                system_instruction: None,
                contents: vec![],
                tools: vec![],
                tool_config: None,
                safety_settings: vec![],
                generation_config: None,
                cached_content: None,
            },
        }
    }

    /// Mutates the client by setting sane default configurations based on the model.
    pub fn with_defaults(&mut self) -> Self {
        let safety_settings = all::<HarmCategory>()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|cat| SafetySettings {
                category: cat,
                threshold: HarmBlockThreshold::default(),
            })
            .collect();

        let generation_config = match &self.model {
            GoogleModel::Gemini20FlashExpImageGen(_) => GenerationConfig {
                response_modalities: vec![Modality::Text, Modality::Image],
                ..Default::default()
            },
            GoogleModel::Gemini20Flash(_) | GoogleModel::Gemini25Flash(_) => GenerationConfig {
                response_modalities: vec![Modality::Text],
                ..Default::default()
            },
        };

        self.request.safety_settings = safety_settings;
        self.request.generation_config = Some(generation_config);

        self.to_owned()
    }

    /// Mutate the client by setting the specified safety settings.
    pub fn with_safety(&mut self, safety_settings: &[SafetySettings]) -> Self {
        self.request.safety_settings = safety_settings.to_vec();

        self.to_owned()
    }

    /// Mutate the client by setting the specified system instructions.  Some models do
    /// not support system instructions, so in these cases we front-load the system instructions
    /// as user text content.
    pub fn with_instructions(&mut self, system_instruction: &str) -> &mut Self {
        match self.model {
            GoogleModel::Gemini20FlashExpImageGen(_) => {
                // The 2.0 flash experimentation image gen model does not support system instructions
                // as this time, so we'll front-load the instructions as a user message.
                let mut contents = vec![Content {
                    parts: vec![Part::Text(system_instruction.to_string())],
                    role: Role::User,
                }];

                contents.extend(self.request.contents.clone());

                self.request.contents = contents;
            }
            GoogleModel::Gemini20Flash(_) | GoogleModel::Gemini25Flash(_) => {
                self.request.system_instruction = Some(Content {
                    role: Role::User,
                    parts: vec![Part::Text(system_instruction.to_string())],
                });
            }
        }

        self
    }

    pub fn with_options(&mut self, options: &GenerationConfig) -> &mut Self {
        let options = match &self.model {
            GoogleModel::Gemini20FlashExpImageGen(_) => options.clone(),
            GoogleModel::Gemini20Flash(_) | GoogleModel::Gemini25Flash(_) => GenerationConfig {
                response_modalities: vec![Modality::Text],
                ..options.clone()
            },
        };
        self.request.generation_config = Some(options.clone());
        self
    }

    /// Since we're dealing with streams it is possible (?) for the stream to contain
    /// a mixture of successful responses and errors.  For simplicity we bail on error
    /// and return just the error, while we reconsolidate all successful responses.
    fn merge_response(&mut self, responses: &[ContentResponse]) -> Result<Responses, Error> {
        let mut success = Vec::new();

        for response in responses {
            if let Some(error) = &response.error {
                return Err(Error::Request(serde_json::to_string(error)?));
            } else {
                for candidate in &response.candidates {
                    if !candidate.content.parts.is_empty() {
                        self.request.contents.push(candidate.content.clone());
                    }
                }
                success.push(response.clone());
            }
        }

        Ok(Responses(success))
    }

    async fn post(&mut self) -> Result<Responses, Error> {
        self.merge_response(
            &self
                .client
                .post(self.url())
                .header("Content-Type", "application/json")
                .query(&[("key", &self.key)])
                .json(&self.request)
                .send()
                .await?
                .json::<Vec<ContentResponse>>()
                .await?,
        )
    }

    /// Send the given text to the model.  Returns the responses or an error
    /// message if an error was returned.
    pub async fn send_text(&mut self, text: &str) -> Result<Responses, Error> {
        self.request.contents.push(Content {
            parts: vec![Part::Text(text.to_string())],
            role: Role::User,
        });

        self.post().await
    }

    pub async fn send_image(
        &mut self,
        message: Option<String>,
        img: &Path,
    ) -> Result<Responses, Error> {
        let format = FileFormat::from_file(img)?;

        let data = BASE64_URL_SAFE.encode(&tokio::fs::read(img).await?);

        self.send_image_bytes(message, format.media_type(), &data)
            .await
    }

    /// Send the given image to the model.  This must be a UTF-8 Base64 encoded
    /// string which is required by the Google API.  Optional text may be sent with
    /// the image to create a single consolidated message.  Returns the responses
    /// or an error message if an error was returned.
    pub async fn send_image_bytes(
        &mut self,
        message: Option<String>,
        mime_type: &str,
        data: &str,
    ) -> Result<Responses, Error> {
        let mut parts = Vec::new();

        if let Some(message) = message {
            parts.push(Part::Text(message.to_string()));
        }

        parts.push(Part::InlineData(Blob {
            mime_type: mime_type.to_string(),
            data: data.to_string(),
        }));

        self.request.contents.push(Content {
            parts,
            role: Role::User,
        });

        self.post().await
    }

    fn url(&self) -> String {
        format!("{URL_BASE}/{}{URL_EXTENSION}", self.model.name())
    }

    /// Returns the entire session content.
    pub fn history(&self) -> &[Content] {
        &self.request.contents
    }
}
