use std::{path::Path, sync::Arc};

use base64::prelude::*;
use enum_iterator::all;
use file_format::FileFormat;
use rust_mcp_sdk::McpClient;
use serde_json::Value;
use thiserror::Error;

use crate::google::{
    GoogleModel,
    common::{Blob, Content, FileData, FunctionCall, HarmCategory, Modality, Part, Role},
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
    #[error("Agent Request")]
    Request { code: i32, message: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    MpcSdk(#[from] rust_mcp_sdk::error::McpSdkError),
    #[error("{0}")]
    UnsupportedConfig(String),
    #[error("{0}")]
    NotFound(String),
}

impl From<&Value> for Error {
    fn from(value: &Value) -> Self {
        let mut code = 0;
        let mut message = String::new();
        if let Ok(map) = serde_json::from_value::<serde_json::Map<String, Value>>(value.clone()) {
            if let Some(cd) = map.get("code") {
                code = serde_json::from_value::<i32>(cd.clone()).unwrap_or(0);
            }
            if let Some(msg) = map.get("message") {
                message = serde_json::from_value::<String>(msg.clone())
                    .unwrap_or_else(|_| "Unknown error".to_string());
            }
        }
        Error::Request { code, message }
    }
}

/// Wrapper struct which stores the HTTP Reqwest client and the request history.  The `send`
/// methods are used to send text and images without having to manage the history manually.
#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    model: GoogleModel,
    key: String,
    request: GenerateContentRequest,
    mcps: Vec<Arc<rust_mcp_sdk::mcp_client::ClientRuntime>>,
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
    pub async fn new(model: &GoogleModel, key: &str) -> Result<Self, Error> {
        Ok(Client {
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
            mcps: vec![],
        })
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
            GoogleModel::Gemini20Flash(_)
            | GoogleModel::Gemini25Flash(_)
            | GoogleModel::Gemini25Pro(_) => GenerationConfig {
                response_modalities: vec![Modality::Text],
                ..Default::default()
            },
        };

        self.request.safety_settings = safety_settings;
        self.request.generation_config = Some(generation_config);

        self.to_owned()
    }

    pub async fn with_tools_client(
        &mut self,
        mcps: Vec<Arc<rust_mcp_sdk::mcp_client::ClientRuntime>>,
    ) -> Result<Self, Error> {
        let mut tools = Vec::new();

        if matches!(self.model, GoogleModel::Gemini20FlashExpImageGen(_)) {
            return Err(Error::UnsupportedConfig(format!(
                "Model {} does not support tool calls",
                self.model
            )));
        }

        self.mcps = mcps;

        for client in &self.mcps {
            tools.push(client.list_tools(None).await?.tools.into())
        }

        self.request.tools = tools;

        Ok(self.to_owned())
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
            GoogleModel::Gemini20Flash(_)
            | GoogleModel::Gemini25Flash(_)
            | GoogleModel::Gemini25Pro(_) => {
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
            GoogleModel::Gemini20Flash(_)
            | GoogleModel::Gemini25Flash(_)
            | GoogleModel::Gemini25Pro(_) => GenerationConfig {
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
    fn merge_response(
        &mut self,
        responses: &[ContentResponse],
    ) -> Result<Vec<ContentResponse>, Error> {
        let mut success = Vec::new();

        for response in responses {
            if let Some(error) = &response.error {
                //return Err(Error::Request(serde_json::to_string(error)?));
                return Err(error.into());
            } else {
                for candidate in &response.candidates {
                    if !candidate.content.parts.is_empty() {
                        self.request.contents.push(candidate.content.clone());
                    }
                }
                success.push(response.clone());
            }
        }

        Ok(success)
    }

    async fn tool_call(&self, function_call: &FunctionCall) -> Result<Vec<Part>, Error> {
        let mut parts = vec![];

        let index = self
            .request
            .tools
            .iter()
            .enumerate()
            .find(|(_i, t)| {
                t.function_declarations
                    .iter()
                    .any(|f| f.name == function_call.name)
            })
            .ok_or_else(|| Error::NotFound(function_call.name.clone()))?
            .0;

        let t = self.mcps.get(index).ok_or_else(|| {
            Error::NotFound(format!("Tool for function call {}", function_call.name))
        })?;

        let response = t
            .call_tool(rust_mcp_sdk::schema::CallToolRequestParams {
                arguments: function_call.args.clone(),
                name: function_call.name.clone(),
            })
            .await?;

        for content in &response.content {
            let part = match content {
                rust_mcp_sdk::schema::CallToolResultContentItem::TextContent(text_content) => {
                    Part::FunctionResponse(crate::google::common::FunctionResponse {
                        id: None,
                        name: function_call.name.clone(),
                        response: serde_json::from_str::<serde_json::Map<String, Value>>(
                            &serde_json::to_string(text_content)?,
                        )?,
                    })
                }
                rust_mcp_sdk::schema::CallToolResultContentItem::ImageContent(image_content) => {
                    Part::FunctionResponse(crate::google::common::FunctionResponse {
                        id: None,
                        name: function_call.name.clone(),
                        response: serde_json::from_str::<serde_json::Map<String, Value>>(
                            &serde_json::to_string(image_content)?,
                        )?,
                    })
                }
                rust_mcp_sdk::schema::CallToolResultContentItem::AudioContent(audio_content) => {
                    Part::FunctionResponse(crate::google::common::FunctionResponse {
                        id: None,
                        name: function_call.name.clone(),
                        response: serde_json::from_str::<serde_json::Map<String, Value>>(
                            &serde_json::to_string(audio_content)?,
                        )?,
                    })
                }
                rust_mcp_sdk::schema::CallToolResultContentItem::EmbeddedResource(
                    embedded_resource,
                ) => Part::FunctionResponse(crate::google::common::FunctionResponse {
                    id: None,
                    name: function_call.name.clone(),
                    response: serde_json::from_str::<serde_json::Map<String, Value>>(
                        &serde_json::to_string(embedded_resource)?,
                    )?,
                }),
            };

            parts.push(part);
        }

        Ok(parts)
    }

    /// Processes tool requests from the model.  We need to push all results onto the content
    /// request stack for the history.
    async fn process_tools(&mut self, in_responses: &[ContentResponse]) -> Result<bool, Error> {
        let mut fn_calls = Vec::new();

        for in_response in in_responses {
            for in_candidate in &in_response.candidates {
                for in_part in &in_candidate.content.parts {
                    match in_part {
                        Part::Thought(_)
                        | Part::Text(_)
                        | Part::InlineData(_)
                        | Part::FileData(_)
                        | Part::ExecutableCode(_)
                        | Part::CodeExecutionResult(_)
                        | Part::FunctionResponse(_) => {}
                        Part::FunctionCall(function_call) => {
                            fn_calls.push(function_call.clone());
                        }
                    }
                }
            }
        }

        if !fn_calls.is_empty() {
            for function_call in &fn_calls {
                let parts = self.tool_call(function_call).await?;

                self.request.contents.push(Content {
                    parts,
                    role: Role::User,
                });
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn do_post(&mut self) -> Result<Vec<ContentResponse>, Error> {
        let request = self
            .client
            .post(self.url())
            .header("Content-Type", "application/json")
            .query(&[("key", &self.key)])
            .json(&self.request);

        let responses = request.send().await?.json::<Vec<ContentResponse>>().await?;

        self.merge_response(&responses)
    }

    async fn post(&mut self) -> Result<Responses, Error> {
        let mut responses = self.do_post().await?;

        // Process all functions that the model maay be calling and feed the results
        // back in.
        while self.process_tools(&responses).await? {
            responses = self.do_post().await?;
        }

        Ok(Responses(responses))
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

    pub async fn send_image(&mut self, blob: &Blob) -> Result<Responses, Error> {
        self.request.contents.push(Content {
            parts: vec![Part::InlineData(blob.clone())],
            role: Role::User,
        });

        self.post().await
    }

    pub async fn send_file_data(&mut self, data: &FileData) -> Result<Responses, Error> {
        self.request.contents.push(Content {
            parts: vec![Part::FileData(data.clone())],
            role: Role::User,
        });

        self.post().await
    }

    pub async fn send_image_file(
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
