use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct GlobTool;

impl GlobTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct GlobInput {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
}

#[async_trait]
impl ToolExecutor for GlobTool {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult> {
        let args: GlobInput = serde_json::from_value(input)
            .context("Invalid arguments for glob tool")?;

        let base_path = match args.path {
            Some(ref p) => PathBuf::from(p),
            None => ctx.working_directory.clone(),
        };

        if !base_path.exists() {
            return Ok(ToolResult::error(format!("Directory not found: {:?}", base_path)));
        }

        let pattern = glob::Pattern::new(&args.pattern)
            .with_context(|| format!("Invalid glob pattern: {}", args.pattern))?;

        let mut matches = Vec::new();
        
        for entry in WalkDir::new(&base_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let relative = entry.path().strip_prefix(&base_path).unwrap_or(entry.path());
            if pattern.matches_path(relative) {
                matches.push(entry.path().display().to_string());
            }
        }

        matches.sort();

        if matches.is_empty() {
            Ok(ToolResult::success(format!(
                "No files matching pattern '{}' in {:?}",
                args.pattern, base_path
            )))
        } else {
            Ok(ToolResult::success(format!(
                "Found {} files matching '{}':\n{}",
                matches.len(),
                args.pattern,
                matches.join("\n")
            )))
        }
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "glob",
            "Find files matching a glob pattern. Returns a list of matching file paths.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The glob pattern to match files against (e.g., \"**/*.rs\", \"src/**/*.py\")"
                    },
                    "path": {
                        "type": "string",
                        "description": "The directory to search in (defaults to current working directory)"
                    }
                },
                "required": ["pattern"]
            }),
        )
    }
}
