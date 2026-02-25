use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct ReadTool;

impl ReadTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct ReadInput {
    file_path: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

#[async_trait]
impl ToolExecutor for ReadTool {
    async fn execute(&self, input: Value, _ctx: &ExecutionContext) -> Result<ToolResult> {
        let args: ReadInput = serde_json::from_value(input)
            .context("Invalid arguments for read tool")?;

        let path = PathBuf::from(&args.file_path);
        
        if !path.exists() {
            return Ok(ToolResult::error(format!("File not found: {}", args.file_path)));
        }

        if !path.is_file() {
            return Ok(ToolResult::error(format!("Not a file: {}", args.file_path)));
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {}", args.file_path))?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let (start, end) = match (args.offset, args.limit) {
            (Some(offset), Some(limit)) => {
                let start = offset.min(total_lines);
                let end = (start + limit).min(total_lines);
                (start, end)
            }
            (Some(offset), None) => {
                let start = offset.min(total_lines);
                (start, total_lines)
            }
            (None, Some(limit)) => {
                (0, limit.min(total_lines))
            }
            (None, None) => (0, total_lines),
        };

        let result_lines: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>6}→{}", start + i + 1, line))
            .collect();

        let output = if result_lines.len() < total_lines {
            format!(
                "File: {} (lines {}-{} of {})\n{}\n",
                args.file_path,
                start + 1,
                end,
                total_lines,
                result_lines.join("\n")
            )
        } else {
            format!(
                "File: {} ({} lines)\n{}\n",
                args.file_path,
                total_lines,
                result_lines.join("\n")
            )
        };

        Ok(ToolResult::success(output))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "read",
            "Read the contents of a file. Returns the file content with line numbers. Supports reading a specific range of lines.",
            json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The absolute path to the file to read"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "The line number to start reading from (1-indexed)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "The maximum number of lines to read"
                    }
                },
                "required": ["file_path"]
            }),
        )
    }
}
