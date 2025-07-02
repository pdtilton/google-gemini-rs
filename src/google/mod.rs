//! Wrapper types for supported Google AI Models

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
const GEMINI_2_5_PRO: &str = "gemini-2.5-pro";

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct Gemini25Pro {
    name: String,
}

impl Default for Gemini25Pro {
    fn default() -> Self {
        Self {
            name: GEMINI_2_5_PRO.to_string(),
        }
    }
}

impl Gemini25Pro {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct Gemini25Flash {
    name: String,
}

impl Default for Gemini25Flash {
    fn default() -> Self {
        Self {
            name: GEMINI_2_5_FLASH.to_string(),
        }
    }
}

impl Gemini25Flash {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct Gemini20Flash {
    name: String,
}

impl Default for Gemini20Flash {
    fn default() -> Self {
        Self {
            name: GEMINI_2_0_FLASH.to_string(),
        }
    }
}

impl Gemini20Flash {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Gemini20FlashExpImageGen {
    name: String,
}

impl Default for Gemini20FlashExpImageGen {
    fn default() -> Self {
        Self {
            name: GEMINI_2_0_FLASH_EXP_IMAGE_GEN.to_string(),
        }
    }
}

impl Gemini20FlashExpImageGen {
    fn name(&self) -> String {
        self.name.clone()
    }
}

/// Supported Google AI models.  Some models have different capabilities than others, so this
/// enum may be used to branch the different capabilities.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum GoogleModel {
    Gemini20FlashExpImageGen(Gemini20FlashExpImageGen),
    Gemini20Flash(Gemini20Flash),
    Gemini25Flash(Gemini25Flash),
    Gemini25Pro(Gemini25Pro),
}

impl TryFrom<&str> for GoogleModel {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        match value {
            GEMINI_2_5_PRO => Ok(GoogleModel::Gemini25Pro(Gemini25Pro::default())),
            GEMINI_2_5_FLASH => Ok(GoogleModel::Gemini25Flash(Gemini25Flash::default())),
            GEMINI_2_0_FLASH => Ok(GoogleModel::Gemini20Flash(Gemini20Flash::default())),
            GEMINI_2_0_FLASH_EXP_IMAGE_GEN => Ok(GoogleModel::Gemini20FlashExpImageGen(
                Gemini20FlashExpImageGen::default(),
            )),
            _ => Err(Error::NotFound(format!("No such model: {value}"))),
        }
    }
}

impl TryFrom<String> for GoogleModel {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl Display for GoogleModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl GoogleModel {
    pub fn name(&self) -> String {
        match self {
            GoogleModel::Gemini20FlashExpImageGen(g) => g.name(),
            GoogleModel::Gemini20Flash(g) => g.name(),
            GoogleModel::Gemini25Flash(g) => g.name(),
            GoogleModel::Gemini25Pro(g) => g.name(),
        }
    }
}
