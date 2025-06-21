//! Response types and wrappers for Google AI Models. See: https://ai.google.dev/api/generate-content

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::common::{Content, HarmCategory, HarmProbability, Modality};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinishReason {
    FinishReasonUnspecified,
    Stop,
    MaxTokens,
    Safety,
    Recitation,
    Language,
    Other,
    BlockList,
    ProhibitedContent,
    Spii,
    MalformedFunctionCall,
    ImageSafety,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRating {
    pub category: HarmCategory,
    pub probability: HarmProbability,
    #[serde(default)]
    pub blocked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    #[serde(default)]
    pub start_index: Option<u32>,
    #[serde(default)]
    pub end_index: Option<u32>,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    #[serde(default)]
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingPassageId {
    pub passage_id: String,
    pub part_index: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticRetrieverChunk {
    pub source: String,
    pub chunk: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributionSourceId {
    pub grounding_passage: GroundingPassageId,
    pub semantic_retriever_chunk: SemanticRetrieverChunk,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingAttribution {
    pub source_id: AttributionSourceId,
    pub content: Content,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Web {
    pub uri: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingChunk {
    pub web: Web,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub part_index: i32,
    pub start_index: i32,
    pub end_index: i32,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingSupport {
    #[serde(default)]
    pub grounding_chunk_indices: Vec<i32>,
    #[serde(default)]
    pub confidence_scores: Vec<f32>,
    pub segment: Segment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEntryPoint {
    #[serde(default)]
    pub rendered_content: Option<String>,
    #[serde(default)]
    pub sdk_blob: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalMetadata {
    #[serde(default)]
    pub google_search_dynamic_retrieval_score: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingMetadata {
    #[serde(default)]
    pub grounding_chunks: Vec<GroundingChunk>,
    #[serde(default)]
    pub grounding_supports: Vec<GroundingSupport>,
    #[serde(default)]
    pub web_search_queries: Vec<String>,
    #[serde(default)]
    pub search_entry_point: Option<SearchEntryPoint>,
    pub retrieval_metadata: RetrievalMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "candidate")]
pub struct LogCandidate {
    pub token: String,
    pub token_id: i32,
    pub log_probability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopCandidates {
    pub candidates: LogCandidate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogprobsResult {
    #[serde(default)]
    pub top_candidates: Vec<TopCandidates>,
    #[serde(default)]
    pub chosen_candidates: Vec<LogCandidate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlRetrievalContext {
    pub retrieved_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlRetrievalMetadata {
    #[serde(default)]
    pub url_retrieval_contexts: Vec<UrlRetrievalContext>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
    #[serde(default)]
    pub finish_reason: Option<FinishReason>,
    #[serde(default)]
    pub safety_ratings: Vec<SafetyRating>,
    #[serde(default)]
    pub citation_metadata: Option<CitationMetadata>,
    #[serde(default)]
    pub grounding_attributions: Vec<GroundingAttribution>,
    #[serde(default)]
    pub grounding_metadata: Option<GroundingMetadata>,
    #[serde(default)]
    pub avg_logprobs: Option<f32>,
    #[serde(default)]
    pub logprobs_result: Option<LogprobsResult>,
    #[serde(default)]
    pub url_retrieval_metadata: Option<UrlRetrievalMetadata>,
    #[serde(default)]
    pub index: Option<i32>,
    #[serde(default)]
    pub token_count: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockReason {
    BlockReasonUnspecified,
    Safety,
    Other,
    BlockList,
    ProhibitedContent,
    ImageSafety,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedBack {
    #[serde(default)]
    pub block_reason: Option<BlockReason>,
    #[serde(default)]
    pub safety_ratings: Vec<SafetyRating>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModalityTokenCount {
    pub modality: Modality,
    pub token_count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetadata {
    #[serde(default)]
    pub prompt_token_count: Option<i32>,
    #[serde(default)]
    pub cached_content_token_count: Option<i32>,
    #[serde(default)]
    pub candidates_token_count: Option<i32>,
    #[serde(default)]
    pub tool_use_prompt_token_count: Option<i32>,
    #[serde(default)]
    pub thoughts_token_count: Option<i32>,
    #[serde(default)]
    pub total_token_count: Option<i32>,
    #[serde(default)]
    pub prompt_tokens_details: Vec<ModalityTokenCount>,
    #[serde(default)]
    pub cache_tokens_details: Vec<ModalityTokenCount>,
    #[serde(default)]
    pub candidates_tokens_details: Vec<ModalityTokenCount>,
    #[serde(default)]
    pub tool_use_prompt_tokens_details: Vec<ModalityTokenCount>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentResponse {
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    #[serde(default)]
    pub prompt_feedback: Option<PromptFeedBack>,
    #[serde(default)]
    pub usage_metadata: Option<UsageMetadata>,
    #[serde(default)]
    pub model_version: Option<String>,
    #[serde(default)]
    pub error: Option<Value>,
}
