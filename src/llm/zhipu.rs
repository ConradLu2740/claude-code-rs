use crate::llm::{LlmClient, LlmClientConfig, LlmStream, Message, StreamEvent, ToolDefinition};
use crate::llm::message::{ChatCompletionChunk, ChatCompletionResponse, messages_to_openai_format};
use crate::utils::HTTP_CLIENT;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::wrappers::ReceiverStream;

pub struct ZhipuClient {
    config: LlmClientConfig,
}

impl ZhipuClient {
    pub fn new(config: LlmClientConfig) -> Result<Self> {
        Ok(Self { config })
    }

    fn generate_jwt_token(api_key: &str) -> Result<String> {
        let parts: Vec<&str> = api_key.split('.').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid Zhipu API key format. Expected: id.secret"));
        }
        
        let id = parts[0];
        let secret = parts[1];
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get timestamp")?
            .as_millis() as u64;
        
        let exp = timestamp + 3600000;
        
        let header = serde_json::json!({
            "alg": "HS256",
            "sign_type": "SIGN"
        });
        
        let payload = serde_json::json!({
            "api_key": id,
            "exp": exp,
            "timestamp": timestamp
        });
        
        let header_b64 = BASE64.encode(serde_json::to_string(&header)?.as_bytes());
        let payload_b64 = BASE64.encode(serde_json::to_string(&payload)?.as_bytes());
        
        let message = format!("{}.{}", header_b64, payload_b64);
        
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|_| anyhow!("Failed to create HMAC"))?;
        mac.update(message.as_bytes());
        let signature = BASE64.encode(mac.finalize().into_bytes());
        
        Ok(format!("{}.{}", message, signature))
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
        let token = Self::generate_jwt_token(&self.config.api_key)?;
        
        let response = HTTP_CLIENT
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Zhipu API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Zhipu API error: {}", error_text));
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .context("Failed to parse Zhipu API response")?;

        Ok(completion)
    }
}

#[async_trait]
impl LlmClient for ZhipuClient {
    fn provider_name(&self) -> &str {
        "zhipu"
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
            .ok_or_else(|| anyhow!("No response from Zhipu API"))
    }

    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<LlmStream> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = self.build_request_body(messages, tools, true);
        let token = Self::generate_jwt_token(&self.config.api_key)?;

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<StreamEvent>>(100);

        let event_source = EventSource::new(
            HTTP_CLIENT
                .post(&url)
                .header("Authorization", format!("Bearer {}", token))
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
