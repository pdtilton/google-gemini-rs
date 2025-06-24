//! Request types and wrappers for Google AI Models. See: https://ai.google.dev/api/generate-content

use std::collections::HashMap;

use rust_mcp_sdk::{error::McpSdkError, schema::ToolInputSchema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use super::common::{Content, HarmCategory, Modality};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    McpSdk(#[from] McpSdkError),
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    #[serde(alias = "typeunspecified")]
    #[default]
    TypeUnspecified,
    #[serde(alias = "string")]
    String,
    #[serde(alias = "number")]
    Number,
    #[serde(alias = "integer")]
    Integer,
    #[serde(alias = "boolean")]
    Boolean,
    #[serde(alias = "array")]
    Array,
    #[serde(alias = "object")]
    Object,
    #[serde(alias = "null")]
    Null,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    pub r#type: Type,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub r#enum: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_items: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Schema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub any_of: Vec<Schema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub property_ordering: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f32>,
}

impl TryFrom<ToolInputSchema> for Schema {
    type Error = Error;

    fn try_from(value: ToolInputSchema) -> Result<Self, Error> {
        // Behold the power of serde: convert the MCP tool schema to the
        // Gemini tool schema.
        Ok(serde_json::from_value::<Schema>(json!(value))?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Schema>,
}

pub fn map_fn_name(index: usize, name: &str) -> String {
    format!("{index}_{name}")
}

pub fn unmap_fn_name(name: &str) -> Result<String, Error> {
    Ok(name
        .split_once('_')
        .ok_or_else(|| Error::NotFound("Function name: {name}".to_string()))?
        .1
        .to_string())
}

impl From<&rust_mcp_sdk::schema::Tool> for FunctionDeclaration {
    fn from(value: &rust_mcp_sdk::schema::Tool) -> Self {
        Self {
            name: value.name.clone(),
            description: value
                .description
                .clone()
                .unwrap_or_else(|| "None".to_string()),
            parameters: value.input_schema.clone().try_into().ok(),
            response: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Mode {
    ModeUnspecified,
    ModeDynamic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicRetrievalConfig {
    pub mode: Mode,
    pub dynamic_threshold: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearchRetrieval {
    pub dynamic_retrieval_config: DynamicRetrievalConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlContext {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub function_declarations: Vec<FunctionDeclaration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_retrieval: Option<GoogleSearchRetrieval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context: Option<UrlContext>,
}

impl From<Vec<rust_mcp_sdk::schema::Tool>> for Tool {
    fn from(value: Vec<rust_mcp_sdk::schema::Tool>) -> Self {
        Self {
            function_declarations: value.iter().map(|t| t.into()).collect(),
            google_search_retrieval: None,
            code_execution: None,
            google_search: None,
            url_context: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionCallingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<Mode>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed_function_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_calling_config: Option<FunctionCallingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmBlockThreshold {
    BlockNone,
    BlockOnlyHigh,
    BlockMediumAndAbove,
    #[default]
    BlockLowAndAbove,
    HarmBlockThresholdUnspecified,
    Off,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetySettings {
    pub category: HarmCategory,
    pub threshold: HarmBlockThreshold,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrebuiltVoiceConfig {
    pub voice_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceConfig {
    pub prebuilt_voice_config: PrebuiltVoiceConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechConfig {
    pub voice_config: VoiceConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingConfig {
    pub include_thoughts: bool,
    pub thinking_budget: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaResolution {
    MediaResolutionUnspecified,
    MediaResolutionLow,
    MediaResolutionMedium,
    MediaResolutionHigh,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<Schema>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub response_modalities: Vec<Modality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_enhanced_civic_answers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<SpeechConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_resolution: Option<MediaResolution>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateContentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
    /// The chat history (user and model, except for the last user message which is a message)
    pub contents: Vec<Content>,
    /// The tools to use
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_config: Option<ToolConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub safety_settings: Vec<SafetySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,
}
