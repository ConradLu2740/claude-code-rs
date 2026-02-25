use clap::Parser;
use claude_code_rs::cli::{CliArgs, Commands};
use claude_code_rs::config::{ConfigLoader, ensure_directories};
use claude_code_rs::cli::commands::{
    chat::run_chat,
    ask::run_ask,
    session::run_session,
    index::run_index,
    search::run_search,
    tool_cmd::run_tool,
    config_cmd::run_config,
    tools_cmd::run_tools,
};
use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = CliArgs::parse();

    let needs_api_key = !matches!(args.command, Some(Commands::Tools) | Some(Commands::Config { .. }));

    let mut loader = if let Some(config_path) = &args.config {
        ConfigLoader::with_path(config_path.clone())
    } else {
        ConfigLoader::new()
    };

    let mut config = loader.load_with_validation(needs_api_key)?;

    if let Some(api_key) = &args.api_key {
        config.llm.api_key = Some(api_key.clone());
    }

    if let Some(model) = &args.model {
        config.llm.model = model.clone();
    }

    match args.provider.as_str() {
        "zhipu" => config.llm.provider = claude_code_rs::config::LlmProvider::Zhipu,
        "deepseek" => config.llm.provider = claude_code_rs::config::LlmProvider::DeepSeek,
        "qwen" => config.llm.provider = claude_code_rs::config::LlmProvider::Qwen,
        "moonshot" => config.llm.provider = claude_code_rs::config::LlmProvider::Moonshot,
        "openai" => config.llm.provider = claude_code_rs::config::LlmProvider::OpenAI,
        "anthropic" => config.llm.provider = claude_code_rs::config::LlmProvider::Anthropic,
        _ => {}
    }

    ensure_directories(&config)?;

    let working_directory = args.workdir.canonicalize().unwrap_or_else(|_| args.workdir.clone());

    match args.command {
        None => {
            run_chat(config, working_directory, None, None).await?;
        }
        Some(Commands::Chat { session, system }) => {
            run_chat(config, working_directory, session, system).await?;
        }
        Some(Commands::Ask { message, json }) => {
            let msg = message.join(" ");
            run_ask(config, &msg, json, working_directory).await?;
        }
        Some(Commands::Session { action }) => {
            run_session(action, config.storage.session_dir)?;
        }
        Some(Commands::Index { path, force }) => {
            run_index(&config, path, force)?;
        }
        Some(Commands::Search { query, top_k }) => {
            let q = query.join(" ");
            run_search(&config, &q, top_k)?;
        }
        Some(Commands::Tool { name, input }) => {
            run_tool(&config, &name, input.as_deref(), working_directory).await?;
        }
        Some(Commands::Config { generate }) => {
            run_config(&config, generate)?;
        }
        Some(Commands::Tools) => {
            run_tools()?;
        }
    }

    Ok(())
}
