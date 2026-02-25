use crate::config::SandboxConfig;
use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub struct ShellTool {
    config: SandboxConfig,
}

impl ShellTool {
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }

    pub fn with_config(config: SandboxConfig) -> Self {
        Self { config }
    }

    fn is_command_blocked(&self, command: &str) -> bool {
        let cmd_lower = command.to_lowercase();
        for blocked in &self.config.blocked_commands {
            if cmd_lower.starts_with(&blocked.to_lowercase()) {
                return true;
            }
        }
        false
    }

    fn is_path_blocked(&self, path: &PathBuf) -> bool {
        for blocked in &self.config.blocked_paths {
            if path.starts_with(blocked) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Deserialize)]
struct ShellInput {
    command: String,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    timeout: Option<u64>,
}

#[async_trait]
impl ToolExecutor for ShellTool {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult> {
        let args: ShellInput = serde_json::from_value(input)
            .context("Invalid arguments for shell tool")?;

        if self.is_command_blocked(&args.command) {
            return Ok(ToolResult::error(format!(
                "Command '{}' is blocked for security reasons",
                args.command.split_whitespace().next().unwrap_or(&args.command)
            )));
        }

        let cwd = match args.cwd {
            Some(ref p) => {
                let path = PathBuf::from(p);
                if self.is_path_blocked(&path) {
                    return Ok(ToolResult::error(format!("Path {:?} is blocked for security reasons", path)));
                }
                path
            }
            None => ctx.working_directory.clone(),
        };

        let timeout = Duration::from_secs(args.timeout.unwrap_or(self.config.max_execution_time_secs));

        let shell = if cfg!(target_os = "windows") {
            "powershell"
        } else {
            "sh"
        };

        let shell_arg = if cfg!(target_os = "windows") {
            "-Command"
        } else {
            "-c"
        };

        let start = Instant::now();

        let mut child = Command::new(shell)
            .arg(shell_arg)
            .arg(&args.command)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn shell process")?;

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let timeout_time = tokio::time::sleep(timeout);
        tokio::pin!(timeout_time);

        loop {
            tokio::select! {
                _ = &mut timeout_time => {
                    let _ = child.kill().await;
                    return Ok(ToolResult::error(format!(
                        "Command timed out after {} seconds",
                        timeout.as_secs()
                    )));
                }
                result = stdout_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            if stdout_lines.len() < self.config.max_output_size {
                                stdout_lines.push(line);
                            }
                        }
                        Ok(None) => break,
                        Err(_) => continue,
                    }
                }
                result = stderr_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            if stderr_lines.len() < self.config.max_output_size {
                                stderr_lines.push(line);
                            }
                        }
                        Ok(None) => {}
                        Err(_) => continue,
                    }
                }
            }
        }

        let status = child.wait().await;
        let duration = start.elapsed();

        let exit_code = match status {
            Ok(s) => s.code(),
            Err(e) => return Ok(ToolResult::error(format!("Failed to wait for process: {}", e))),
        };

        let mut output = String::new();
        
        if !stdout_lines.is_empty() {
            output.push_str(&stdout_lines.join("\n"));
        }
        
        if !stderr_lines.is_empty() {
            if !output.is_empty() {
                output.push_str("\n\n--- stderr ---\n");
            }
            output.push_str(&stderr_lines.join("\n"));
        }

        let result = if exit_code == Some(0) {
            ToolResult::success(format!(
                "Command completed in {:.2}s\n{}",
                duration.as_secs_f64(),
                output
            ))
        } else {
            ToolResult::error(format!(
                "Command exited with code {:?} in {:.2}s\n{}",
                exit_code,
                duration.as_secs_f64(),
                output
            ))
        };

        Ok(result)
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "shell",
            "Execute a shell command. Returns the output and exit code. Commands run in a sandboxed environment with timeouts.",
            json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "The working directory for the command (defaults to current directory)"
                    },
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in seconds (default 60)"
                    }
                },
                "required": ["command"]
            }),
        )
    }

    fn requires_confirmation(&self) -> bool {
        true
    }
}
