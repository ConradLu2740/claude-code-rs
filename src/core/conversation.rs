use crate::llm::{Message, ToolCall, ToolDefinition};
use crate::tools::ToolRegistry;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct Conversation {
    pub id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    tool_registry: Arc<ToolRegistry>,
    system_prompt: Option<String>,
}

impl Conversation {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            id: Uuid::new_v4(),
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tool_registry,
            system_prompt: None,
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    pub fn add_message(&mut self, message: Message) {
        self.updated_at = Utc::now();
        self.messages.push(message);
    }

    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::user(content));
    }

    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(Message::assistant(content));
    }

    pub fn add_tool_result(&mut self, tool_call_id: impl Into<String>, result: impl Into<String>) {
        self.add_message(Message::tool_result(tool_call_id, result));
    }

    pub fn get_messages_for_api(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        if let Some(ref prompt) = self.system_prompt {
            messages.push(Message::system(prompt));
        }

        messages.extend(self.messages.clone());
        messages
    }

    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tool_registry.get_all_definitions()
    }

    pub fn get_tool_registry(&self) -> Arc<ToolRegistry> {
        self.tool_registry.clone()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.updated_at = Utc::now();
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn truncate_messages(&mut self, max_messages: usize) {
        if self.messages.len() > max_messages {
            let keep_count = max_messages;
            let remove_count = self.messages.len() - keep_count;
            self.messages.drain(0..remove_count);
            self.updated_at = Utc::now();
        }
    }
}

pub struct ConversationBuilder {
    tool_registry: Arc<ToolRegistry>,
    system_prompt: Option<String>,
    initial_messages: Vec<Message>,
}

impl ConversationBuilder {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            tool_registry,
            system_prompt: None,
            initial_messages: Vec::new(),
        }
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_message(mut self, message: Message) -> Self {
        self.initial_messages.push(message);
        self
    }

    pub fn build(self) -> Conversation {
        let mut conv = Conversation::new(self.tool_registry);
        if let Some(prompt) = self.system_prompt {
            conv.set_system_prompt(prompt);
        }
        for msg in self.initial_messages {
            conv.add_message(msg);
        }
        conv
    }
}

pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are an AI programming assistant with access to tools for file operations, code execution, and project analysis.

When working with files:
- Always use absolute paths when provided
- Read files before editing them to understand their structure
- Make minimal, targeted edits rather than rewriting entire files

When executing commands:
- Consider the operating system (Windows/Mac/Linux)
- Prefer safe, non-destructive commands
- Always explain what a command will do before executing

When analyzing code:
- Look for patterns and conventions in the existing codebase
- Follow the style and structure of existing code
- Consider dependencies and imports

Be helpful, accurate, and thorough in your responses. Ask clarifying questions when needed."#;
