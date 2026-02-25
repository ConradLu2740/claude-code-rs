use crate::llm::{Message, StreamEvent, ToolDefinition};
use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

pub type LlmStream = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>;

#[async_trait]
pub trait LlmClient: Send + Sync {
    fn provider_name(&self) -> &str;
    
    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Message>;
    
    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<LlmStream>;
    
    fn count_tokens(&self, text: &str) -> usize;
}

#[derive(Debug, Clone)]
pub struct LlmClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub timeout_secs: u64,
}
