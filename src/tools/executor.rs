use crate::config::AppConfig;
use crate::tools::ToolSchema;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub working_directory: PathBuf,
    pub config: Arc<AppConfig>,
    pub session_id: Option<uuid::Uuid>,
}

impl ExecutionContext {
    pub fn new(working_directory: PathBuf, config: Arc<AppConfig>) -> Self {
        Self {
            working_directory,
            config,
            session_id: None,
        }
    }

    pub fn with_session(mut self, session_id: uuid::Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
        }
    }

    pub fn to_json(&self) -> Value {
        if self.success {
            serde_json::json!({
                "success": true,
                "output": self.output
            })
        } else {
            serde_json::json!({
                "success": false,
                "error": self.error
            })
        }
    }
}

impl std::fmt::Display for ToolResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(f, "{}", self.output)
        } else {
            write!(f, "Error: {}", self.error.as_deref().unwrap_or("Unknown error"))
        }
    }
}

#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult>;
    fn schema(&self) -> ToolSchema;
    fn requires_confirmation(&self) -> bool {
        false
    }
}
