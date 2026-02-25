use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct EditTool;

impl EditTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct EditInput {
    file_path: String,
    old_str: String,
    new_str: String,
}

#[async_trait]
impl ToolExecutor for EditTool {
    async fn execute(&self, input: Value, _ctx: &ExecutionContext) -> Result<ToolResult> {
        let args: EditInput = serde_json::from_value(input)
            .context("Invalid arguments for edit tool")?;

        if args.old_str == args.new_str {
            return Ok(ToolResult::error("old_str and new_str are identical, no changes needed"));
        }

        let path = PathBuf::from(&args.file_path);
        
        if !path.exists() {
            return Ok(ToolResult::error(format!("File not found: {}", args.file_path)));
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {}", args.file_path))?;

        if !content.contains(&args.old_str) {
            return Ok(ToolResult::error(format!(
                "Could not find the text to replace in file: {}",
                args.file_path
            )));
        }

        let occurrences = content.matches(&args.old_str).count();
        if occurrences > 1 {
            return Ok(ToolResult::error(format!(
                "Found {} occurrences of old_str. Please provide a more specific search string.",
                occurrences
            )));
        }

        let new_content = content.replace(&args.old_str, &args.new_str);

        std::fs::write(&path, &new_content)
            .with_context(|| format!("Failed to write file: {}", args.file_path))?;

        Ok(ToolResult::success(format!(
            "Successfully edited {} (replaced 1 occurrence)",
            args.file_path
        )))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "edit",
            "Edit a file by replacing a specific string with a new string. The old_str must appear exactly once in the file.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The absolute path to the file to edit"
                    },
                    "old_str": {
                        "type": "string",
                        "description": "The text to search for (must appear exactly once)"
                    },
                    "new_str": {
                        "type": "string",
                        "description": "The text to replace it with"
                    }
                },
                "required": ["file_path", "old_str", "new_str"]
            }),
        )
    }

    fn requires_confirmation(&self) -> bool {
        true
    }
}
