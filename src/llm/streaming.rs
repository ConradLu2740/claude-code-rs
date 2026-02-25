use crate::llm::StreamEvent;
use futures::{Stream, StreamExt};

pub struct StreamProcessor {
    content_buffer: String,
    tool_calls_buffer: Vec<ToolCallBuffer>,
}

#[derive(Debug, Clone, Default)]
struct ToolCallBuffer {
    id: String,
    name: String,
    arguments: String,
}

impl StreamProcessor {
    pub fn new() -> Self {
        Self {
            content_buffer: String::new(),
            tool_calls_buffer: Vec::new(),
        }
    }

    pub async fn process<S>(mut self, mut stream: S) -> StreamResult
    where
        S: Stream<Item = anyhow::Result<StreamEvent>> + Unpin,
    {
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::ContentDelta(delta)) => {
                    self.content_buffer.push_str(&delta);
                    print!("{}", delta);
                }
                Ok(StreamEvent::ToolCallStart { id, name }) => {
                    self.tool_calls_buffer.push(ToolCallBuffer {
                        id,
                        name,
                        arguments: String::new(),
                    });
                }
                Ok(StreamEvent::ToolCallDelta { id, delta }) => {
                    if let Some(tc) = self.tool_calls_buffer.iter_mut().find(|t| t.id == id) {
                        tc.arguments.push_str(&delta);
                    }
                }
                Ok(StreamEvent::MessageStop) => {
                    break;
                }
                Ok(StreamEvent::Error(e)) => {
                    return StreamResult::Error(e);
                }
                Err(e) => {
                    return StreamResult::Error(e.to_string());
                }
            }
        }

        StreamResult::Complete(StreamOutput {
            content: if self.content_buffer.is_empty() {
                None
            } else {
                Some(self.content_buffer)
            },
            tool_calls: if self.tool_calls_buffer.is_empty() {
                None
            } else {
                Some(
                    self.tool_calls_buffer
                        .into_iter()
                        .map(|tc| ProcessedToolCall {
                            id: tc.id,
                            name: tc.name,
                            arguments: tc.arguments,
                        })
                        .collect(),
                )
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct StreamOutput {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ProcessedToolCall>>,
}

#[derive(Debug, Clone)]
pub struct ProcessedToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

impl ProcessedToolCall {
    pub fn parse_arguments<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<T> {
        serde_json::from_str(&self.arguments)
            .map_err(|e| anyhow::anyhow!("Failed to parse tool arguments: {}", e))
    }
}

#[derive(Debug)]
pub enum StreamResult {
    Complete(StreamOutput),
    Error(String),
}
