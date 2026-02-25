use crate::config::{AppConfig, ConfigLoader};
use anyhow::Result;

pub fn run_config(config: &AppConfig, generate: bool) -> Result<()> {
    if generate {
        let default_config = ConfigLoader::generate_default_config();
        println!("{}", default_config);
        return Ok(());
    }

    println!("Current Configuration:");
    println!("{}", "-".repeat(40));
    println!("LLM Provider: {:?}", config.llm.provider);
    println!("Model: {}", if config.llm.model.is_empty() {
        config.llm.provider.default_model()
    } else {
        &config.llm.model
    });
    println!("Max Tokens: {}", config.llm.max_tokens);
    println!("Temperature: {}", config.llm.temperature);
    println!();
    println!("Data Directory: {:?}", config.storage.data_dir);
    println!("Session Directory: {:?}", config.storage.session_dir);
    println!();
    println!("Enabled Tools: {:?}", config.tools.enabled);
    println!("Blocked Commands: {:?}", config.sandbox.blocked_commands);

    Ok(())
}
