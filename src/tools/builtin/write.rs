use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct WriteTool;

impl WriteTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct WriteInput {
    file_path: String,
    content: String,
}

#[async_trait]
impl ToolExecutor for WriteTool {
    async fn execute(&self, input: Value, _ctx: &ExecutionContext) -> Result<ToolResult> {
        let args: WriteInput = serde_json::from_value(input)
            .context("Invalid arguments for write tool")?;

        let path = PathBuf::from(&args.file_path);
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        std::fs::write(&path, &args.content)
            .with_context(|| format!("Failed to write file: {}", args.file_path))?;

        let lines = args.content.lines().count();
        let bytes = args.content.len();

        Ok(ToolResult::success(format!(
            "Successfully wrote {} bytes ({} lines) to {}",
            bytes, lines, args.file_path
        )))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "write",
            "Write content to a file. Creates the file if it doesn't exist, overwrites if it does. Creates parent directories as needed.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The absolute path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file"
                    }
                },
                "required": ["file_path", "content"]
            }),
        )
    }

    fn requires_confirmation(&self) -> bool {
        true
    }
}
