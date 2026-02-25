use crate::llm::{LlmClient, LlmClientConfig, LlmStream, Message, StreamEvent, ToolDefinition};
use crate::llm::message::{ChatCompletionChunk, ChatCompletionResponse, messages_to_openai_format};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;

pub struct DeepSeekClient {
    config: LlmClientConfig,
    client: Client,
}

impl DeepSeekClient {
    pub fn new(config: LlmClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    fn build_request_body(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
        stream: bool,
    ) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages_to_openai_format(&messages),
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "stream": stream,
        });

        if !tools.is_empty() {
            body["tools"] = serde_json::json!(tools);
        }

        body
    }

    async fn execute_request(
        &self,
        body: serde_json::Value,
    ) -> Result<ChatCompletionResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await
            .context("Failed to send request to DeepSeek API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("DeepSeek API error: {}", error_text));
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .context("Failed to parse DeepSeek API response")?;

        Ok(completion)
    }
}

#[async_trait]
impl LlmClient for DeepSeekClient {
    fn provider_name(&self) -> &str {
        "deepseek"
    }

    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Message> {
        let body = self.build_request_body(messages, tools, false);
        let response = self.execute_request(body).await?;

        response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message)
            .ok_or_else(|| anyhow!("No response from DeepSeek API"))
    }

    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<LlmStream> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = self.build_request_body(messages, tools, true);

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<StreamEvent>>(100);

        let event_source = EventSource::new(
            self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .json(&body),
        )
        .context("Failed to create event source")?;

        tokio::spawn(async move {
            let mut event_source = event_source;
            
            while let Some(event) = event_source.next().await {
                match event {
                    Ok(Event::Open) => continue,
                    Ok(Event::Message(message)) => {
                        if message.data == "[DONE]" {
                            let _ = tx.send(Ok(StreamEvent::MessageStop)).await;
                            break;
                        }

                        match serde_json::from_str::<ChatCompletionChunk>(&message.data) {
                            Ok(chunk) => {
                                for choice in chunk.choices {
                                    if let Some(content) = &choice.delta.content {
                                        if !content.is_empty() {
                                            let _ = tx.send(Ok(StreamEvent::ContentDelta(content.clone()))).await;
                                        }
                                    }

                                    if let Some(tool_calls) = &choice.delta.tool_calls {
                                        for tc in tool_calls {
                                            if let Some(id) = &tc.id {
                                                if let Some(func) = &tc.function {
                                                    if let Some(name) = &func.name {
                                                        let _ = tx.send(Ok(StreamEvent::ToolCallStart {
                                                            id: id.clone(),
                                                            name: name.clone(),
                                                        })).await;
                                                    }
                                                }
                                            }
                                            if let Some(func) = &tc.function {
                                                if let Some(args) = &func.arguments {
                                                    let _ = tx.send(Ok(StreamEvent::ToolCallDelta {
                                                        id: tc.id.clone().unwrap_or_default(),
                                                        delta: args.clone(),
                                                    })).await;
                                                }
                                            }
                                        }
                                    }

                                    if choice.finish_reason.is_some() {
                                        let _ = tx.send(Ok(StreamEvent::MessageStop)).await;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(anyhow!("Failed to parse chunk: {}", e))).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(anyhow!("Event source error: {}", e))).await;
                        break;
                    }
                }
            }

            event_source.close();
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }

    fn count_tokens(&self, text: &str) -> usize {
        text.chars().count() / 2
    }
}
