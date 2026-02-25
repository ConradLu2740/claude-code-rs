use crate::config::AppConfig;
use crate::core::Conversation;
use crate::llm::{LlmClient, LlmClientConfig};
use crate::tools::create_default_registry;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::sync::Arc;

pub async fn run_ask(
    config: AppConfig,
    message: &str,
    json_output: bool,
    _working_directory: PathBuf,
) -> Result<()> {
    let config = Arc::new(config);
    
    let client = create_client(&config)?;
    let tool_registry = Arc::new(create_default_registry());
    
    let mut conversation = Conversation::new(tool_registry);
    conversation.set_system_prompt(crate::core::DEFAULT_SYSTEM_PROMPT);
    conversation.add_user_message(message);

    let messages = conversation.get_messages_for_api();
    let tools = conversation.get_tool_definitions();

    let response = client.complete(messages, tools).await?;

    if json_output {
        let response_text = match &response.content {
            crate::llm::MessageContent::Text(text) => text.clone(),
            crate::llm::MessageContent::Parts(parts) => {
                parts.iter()
                    .filter_map(|p| p.text.as_ref())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };
        let json = serde_json::json!({
            "message": message,
            "response": response_text,
            "tool_calls": response.tool_calls,
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        match &response.content {
            crate::llm::MessageContent::Text(text) => {
                println!("{}", text);
            }
            crate::llm::MessageContent::Parts(parts) => {
                for part in parts {
                    if let Some(text) = &part.text {
                        println!("{}", text);
                    }
                }
            }
        }
    }

    Ok(())
}

fn create_client(config: &AppConfig) -> Result<Box<dyn LlmClient>> {
    let api_key = config.llm.api_key.clone()
        .ok_or_else(|| anyhow!("API key not configured"))?;

    let base_url = config.llm.base_url.clone()
        .unwrap_or_else(|| config.llm.provider.default_base_url().to_string());

    let model = if config.llm.model.is_empty() {
        config.llm.provider.default_model().to_string()
    } else {
        config.llm.model.clone()
    };

    let client_config = LlmClientConfig {
        api_key,
        base_url,
        model,
        max_tokens: config.llm.max_tokens,
        temperature: config.llm.temperature,
        timeout_secs: config.llm.timeout_secs,
    };

    let provider_name = match config.llm.provider {
        crate::config::LlmProvider::Zhipu => "zhipu",
        crate::config::LlmProvider::DeepSeek => "deepseek",
        crate::config::LlmProvider::Qwen => "qwen",
        crate::config::LlmProvider::Moonshot => "moonshot",
        crate::config::LlmProvider::OpenAI => "openai",
        crate::config::LlmProvider::Anthropic => "anthropic",
    };

    crate::llm::openai::create_client(client_config, provider_name)
}
