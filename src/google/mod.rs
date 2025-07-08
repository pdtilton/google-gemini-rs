//! Wrapper types for supported Google AI Models

use std::fmt::Display;

use thiserror::Error;

use crate::google::common::Modality;

pub mod common;
pub mod request;
pub mod response;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    NotFound(String),
}

const GEMINI_2_0_FLASH_EXP_IMAGE_GEN: &str = "gemini-2.0-flash-exp-image-generation";
const GEMINI_2_0_FLASH: &str = "gemini-2.0-flash";
const GEMINI_2_5_FLASH: &str = "gemini-2.5-flash";
const GEMINI_2_5_FLASH_LITE: &str = "gemini-2.5-flash-lite";
const GEMINI_2_5_PRO: &str = "gemini-2.5-pro";

/// Supported Google AI models.  Some models have different capabilities than others, so this
/// enum may be used to branch the different capabilities.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum GoogleModelVariant {
    Gemini20FlashExpImageGen,
    Gemini20Flash,
    Gemini25Flash,
    Gemini25Pro,
    Gemini25FlashLight,
}

impl GoogleModelVariant {
    fn name(&self) -> String {
        match self {
            GoogleModelVariant::Gemini20FlashExpImageGen => GEMINI_2_0_FLASH_EXP_IMAGE_GEN,
            GoogleModelVariant::Gemini20Flash => GEMINI_2_0_FLASH,
            GoogleModelVariant::Gemini25Flash => GEMINI_2_5_FLASH,
            GoogleModelVariant::Gemini25Pro => GEMINI_2_5_PRO,
            GoogleModelVariant::Gemini25FlashLight => GEMINI_2_5_FLASH_LITE,
        }
        .to_string()
    }

    fn inputs(&self) -> Vec<Modality> {
        match self {
            GoogleModelVariant::Gemini20FlashExpImageGen => vec![
                Modality::Text,
                Modality::Video,
                Modality::Image,
                Modality::Audio,
            ],
            GoogleModelVariant::Gemini20Flash => vec![
                Modality::Text,
                Modality::Video,
                Modality::Image,
                Modality::Audio,
            ],
            GoogleModelVariant::Gemini25Flash => vec![
                Modality::Text,
                Modality::Video,
                Modality::Image,
                Modality::Audio,
            ],
            GoogleModelVariant::Gemini25Pro => vec![
                Modality::Text,
                Modality::Video,
                Modality::Image,
                Modality::Audio,
            ],
            GoogleModelVariant::Gemini25FlashLight => vec![
                Modality::Text,
                Modality::Video,
                Modality::Image,
                Modality::Audio,
            ],
        }
    }

    fn outputs(&self) -> Vec<Modality> {
        match self {
            GoogleModelVariant::Gemini20FlashExpImageGen => {
                vec![Modality::Text, Modality::Image]
            }
            GoogleModelVariant::Gemini20Flash => vec![Modality::Text],
            GoogleModelVariant::Gemini25Flash => vec![Modality::Text],
            GoogleModelVariant::Gemini25Pro => vec![Modality::Text],
            GoogleModelVariant::Gemini25FlashLight => vec![Modality::Text],
        }
    }
}

#[derive(Debug, Clone)]
pub struct GoogleModel {
    pub variant: GoogleModelVariant,
    pub name: String,
    pub input: Vec<Modality>,
    pub output: Vec<Modality>,
}

impl GoogleModel {
    pub fn new(variant: GoogleModelVariant, suffix: Option<String>) -> Self {
        let name = if let Some(suffix) = suffix {
            format!("{}-{suffix}", variant.name())
        } else {
            variant.name()
        };

        let input = variant.inputs();
        let output = variant.outputs();

        Self {
            variant,
            name,
            input,
            output,
        }
    }
}

impl TryFrom<&str> for GoogleModel {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        let (model, preview) = if let Some((model, preview)) = value.split_once("-preview") {
            (model, Some(format!("preview{preview}")))
        } else {
            (value, None)
        };

        println!("Model: {model} preview: {preview:?}");

        let variant = match model {
            GEMINI_2_5_PRO => Ok(GoogleModelVariant::Gemini25Pro),
            GEMINI_2_5_FLASH => Ok(GoogleModelVariant::Gemini25Flash),
            GEMINI_2_5_FLASH_LITE => Ok(GoogleModelVariant::Gemini25FlashLight),
            GEMINI_2_0_FLASH => Ok(GoogleModelVariant::Gemini20Flash),
            GEMINI_2_0_FLASH_EXP_IMAGE_GEN => Ok(GoogleModelVariant::Gemini20FlashExpImageGen),
            _ => Err(Error::NotFound(format!("No such model: {value}"))),
        }?;

        Ok(GoogleModel::new(variant, preview))
    }
}

impl Display for GoogleModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
