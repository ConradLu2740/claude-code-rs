use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct GrepTool;

impl GrepTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
struct GrepInput {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    glob: Option<String>,
    #[serde(default)]
    case_insensitive: bool,
    #[serde(default)]
    output_mode: Option<String>,
    #[serde(default)]
    head_limit: Option<usize>,
}

#[async_trait]
impl ToolExecutor for GrepTool {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult> {
        let args: GrepInput = serde_json::from_value(input)
            .context("Invalid arguments for grep tool")?;

        let base_path = match args.path {
            Some(ref p) => PathBuf::from(p),
            None => ctx.working_directory.clone(),
        };

        if !base_path.exists() {
            return Ok(ToolResult::error(format!("Path not found: {:?}", base_path)));
        }

        let regex_pattern = if args.case_insensitive {
            regex::RegexBuilder::new(&args.pattern)
                .case_insensitive(true)
                .build()
        } else {
            regex::Regex::new(&args.pattern)
        }
        .with_context(|| format!("Invalid regex pattern: {}", args.pattern))?;

        let glob_pattern = args.glob
            .as_ref()
            .map(|g| glob::Pattern::new(g))
            .transpose()
            .context("Invalid glob pattern")?;

        let mut matches = Vec::new();
        let mut match_count = 0;
        let head_limit = args.head_limit.unwrap_or(100);

        if base_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&base_path) {
                for (line_num, line) in content.lines().enumerate() {
                    if regex_pattern.is_match(line) {
                        match_count += 1;
                        if matches.len() < head_limit {
                            let output = format!("{}:{}:{}", base_path.display(), line_num + 1, line);
                            matches.push(output);
                        }
                    }
                }
            }
        } else {
            for entry in WalkDir::new(&base_path)
                .follow_links(true)
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.') && 
                    name != "node_modules" && 
                    name != "target" &&
                    name != "vendor"
                })
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }

                if let Some(ref glob) = glob_pattern {
                    let relative = entry.path().strip_prefix(&base_path).unwrap_or(entry.path());
                    if !glob.matches_path(relative) {
                        continue;
                    }
                }

                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for (line_num, line) in content.lines().enumerate() {
                        if regex_pattern.is_match(line) {
                            match_count += 1;
                            
                            if matches.len() < head_limit {
                                let file_path = entry.path().display().to_string();
                                let output = match args.output_mode.as_deref() {
                                    Some("content") => format!("{}:{}:{}", file_path, line_num + 1, line),
                                    _ => file_path.clone(),
                                };
                                
                                if !matches.contains(&output) {
                                    matches.push(output);
                                }
                            }
                        }
                    }
                }

                if matches.len() >= head_limit {
                    break;
                }
            }
        }

        if matches.is_empty() {
            Ok(ToolResult::success(format!(
                "No matches found for pattern '{}' in {:?}",
                args.pattern, base_path
            )))
        } else {
            let output = if match_count > matches.len() {
                format!(
                    "Found {} matches (showing first {}):\n{}",
                    match_count,
                    matches.len(),
                    matches.join("\n")
                )
            } else {
                format!(
                    "Found {} matches:\n{}",
                    matches.len(),
                    matches.join("\n")
                )
            };
            Ok(ToolResult::success(output))
        }
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "grep",
            "Search for a regex pattern in files. Returns matching lines with file paths and line numbers.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "The regular expression pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "The file or directory to search in (defaults to current working directory)"
                    },
                    "glob": {
                        "type": "string",
                        "description": "Glob pattern to filter files (e.g., \"*.rs\")"
                    },
                    "case_insensitive": {
                        "type": "boolean",
                        "description": "Whether to perform case-insensitive search"
                    },
                    "output_mode": {
                        "type": "string",
                        "enum": ["content", "files_with_matches"],
                        "description": "Output mode: 'content' shows matching lines, 'files_with_matches' shows only file paths"
                    },
                    "head_limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default 100)"
                    }
                },
                "required": ["pattern"]
            }),
        )
    }
}
