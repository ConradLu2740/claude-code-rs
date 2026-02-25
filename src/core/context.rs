use crate::llm::Message;
use anyhow::Result;
use std::collections::VecDeque;

pub struct ContextManager {
    max_tokens: usize,
    strategy: ContextStrategy,
    current_tokens: usize,
}

#[derive(Debug, Clone)]
pub enum ContextStrategy {
    TruncateOldest,
    SlidingWindow { window_size: usize },
    KeepSystemAndRecent { recent_count: usize },
}

impl ContextManager {
    pub fn new(max_tokens: usize, strategy: ContextStrategy) -> Self {
        Self {
            max_tokens,
            strategy,
            current_tokens: 0,
        }
    }

    pub fn manage(&mut self, messages: &mut Vec<Message>) -> Result<()> {
        self.current_tokens = self.estimate_tokens(messages);
        
        while self.current_tokens > self.max_tokens && !messages.is_empty() {
            match &self.strategy {
                ContextStrategy::TruncateOldest => {
                    if let Some(removed) = messages.first() {
                        self.current_tokens -= self.estimate_message_tokens(removed);
                    }
                    messages.remove(0);
                }
                ContextStrategy::SlidingWindow { window_size } => {
                    if messages.len() > *window_size {
                        let remove_count = messages.len() - window_size;
                        for msg in messages.iter().take(remove_count) {
                            self.current_tokens -= self.estimate_message_tokens(msg);
                        }
                        messages.drain(0..remove_count);
                    } else {
                        break;
                    }
                }
                ContextStrategy::KeepSystemAndRecent { recent_count } => {
                    if messages.len() <= *recent_count + 1 {
                        break;
                    }
                    let start_idx = 1;
                    let end_idx = messages.len().saturating_sub(*recent_count);
                    if start_idx < end_idx {
                        for msg in messages.iter().take(end_idx).skip(start_idx) {
                            self.current_tokens -= self.estimate_message_tokens(msg);
                        }
                        messages.drain(start_idx..end_idx);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn estimate_tokens(&self, messages: &[Message]) -> usize {
        messages.iter().map(|m| self.estimate_message_tokens(m)).sum()
    }

    fn estimate_message_tokens(&self, message: &Message) -> usize {
        let role_tokens = 4;
        let content_tokens = match &message.content {
            crate::llm::MessageContent::Text(text) => text.len() / 4,
            crate::llm::MessageContent::Parts(parts) => {
                parts.iter().map(|p| {
                    p.text.as_ref().map(|t| t.len() / 4).unwrap_or(0)
                }).sum()
            }
        };
        let tool_tokens = message.tool_calls.as_ref().map(|tc| {
            tc.iter().map(|t| {
                t.function.name.len() / 4 + 
                t.function.arguments.to_string().len() / 4
            }).sum::<usize>()
        }).unwrap_or(0);
        
        role_tokens + content_tokens + tool_tokens + 10
    }

    pub fn current_tokens(&self) -> usize {
        self.current_tokens
    }

    pub fn remaining_tokens(&self) -> usize {
        self.max_tokens.saturating_sub(self.current_tokens)
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(128000, ContextStrategy::KeepSystemAndRecent { recent_count: 20 })
    }
}

pub struct ContextSummary {
    pub total_messages: usize,
    pub total_tokens: usize,
    pub remaining_tokens: usize,
    pub utilization_percent: f32,
}

impl ContextManager {
    pub fn summary(&self, messages: &[Message]) -> ContextSummary {
        let total_tokens = self.estimate_tokens(messages);
        ContextSummary {
            total_messages: messages.len(),
            total_tokens,
            remaining_tokens: self.max_tokens.saturating_sub(total_tokens),
            utilization_percent: (total_tokens as f32 / self.max_tokens as f32) * 100.0,
        }
    }
}
