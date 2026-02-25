use crate::config::AppConfig;
use crate::tools::{create_default_registry, ExecutionContext, ToolExecutor};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

pub async fn run_tool(
    config: &AppConfig,
    name: &str,
    input: Option<&str>,
    working_directory: PathBuf,
) -> Result<()> {
    let registry = create_default_registry();
    
    let tool = registry.get(name)
        .ok_or_else(|| anyhow!("Unknown tool: {}", name))?;

    let input_value: Value = match input {
        Some(json_str) => serde_json::from_str(json_str)?,
        None => Value::Object(Default::default()),
    };

    println!("Executing tool: {}", name);
    println!("Input: {}", serde_json::to_string_pretty(&input_value)?);
    println!();

    let ctx = ExecutionContext::new(working_directory, Arc::new(config.clone()));
    let result = tool.execute(input_value, &ctx).await?;

    if result.success {
        println!("Result:\n{}", result.output);
    } else {
        println!("Error: {}", result.error.unwrap_or_default());
    }

    Ok(())
}
