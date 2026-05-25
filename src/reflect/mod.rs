use serde::Serialize;

/// LLM 对 finding 的增强结果
#[derive(Debug, Serialize)]
pub struct LlmEnhancement {
    pub priority: String,
    pub explanation: String,
    pub confidence: String,
}
