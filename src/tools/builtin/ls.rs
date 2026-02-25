use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

pub struct LsTool;

impl LsTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct LsInput {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    ignore: Option<Vec<String>>,
}

#[async_trait]
impl ToolExecutor for LsTool {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult> {
        let args: LsInput = serde_json::from_value(input)
            .context("Invalid arguments for ls tool")?;

        let base_path = match args.path {
            Some(ref p) => PathBuf::from(p),
            None => ctx.working_directory.clone(),
        };

        if !base_path.exists() {
            return Ok(ToolResult::error(format!("Path not found: {:?}", base_path)));
        }

        if !base_path.is_dir() {
            return Ok(ToolResult::error(format!("Not a directory: {:?}", base_path)));
        }

        let ignore_patterns = args.ignore.unwrap_or_default();
        let ignore_patterns: Vec<glob::Pattern> = ignore_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        let mut entries = Vec::new();

        let read_dir = std::fs::read_dir(&base_path)
            .with_context(|| format!("Failed to read directory: {:?}", base_path))?;

        for entry in read_dir.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            
            let should_ignore = ignore_patterns.iter().any(|p| p.matches(&name));
            if should_ignore {
                continue;
            }

            let metadata = entry.metadata().ok();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

            entries.push((name, is_dir, size));
        }

        entries.sort_by(|a, b| {
            match (a.1, b.1) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.0.cmp(&b.0),
            }
        });

        let mut output = format!("Directory: {:?}\n\n", base_path);
        
        for (name, is_dir, size) in entries {
            if is_dir {
                output.push_str(&format!("📁 {}/\n", name));
            } else {
                let size_str = if size < 1024 {
                    format!("{}B", size)
                } else if size < 1024 * 1024 {
                    format!("{}KB", size / 1024)
                } else {
                    format!("{}MB", size / (1024 * 1024))
                };
                output.push_str(&format!("📄 {} ({})\n", name, size_str));
            }
        }

        Ok(ToolResult::success(output))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "ls",
            "List the contents of a directory. Shows files and subdirectories with their sizes.",
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The directory path to list (defaults to current working directory)"
                    },
                    "ignore": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Glob patterns to ignore (e.g., [\"node_modules\", \".git\"])"
                    }
                },
                "required": []
            }),
        )
    }
}
