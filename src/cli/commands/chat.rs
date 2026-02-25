use crate::cli::repl::ReplSession;
use crate::config::AppConfig;
use anyhow::Result;
use std::path::PathBuf;
use uuid::Uuid;

pub async fn run_chat(
    config: AppConfig,
    working_directory: PathBuf,
    session_id: Option<String>,
    _system_prompt: Option<String>,
) -> Result<()> {
    let id = session_id
        .and_then(|s| Uuid::parse_str(&s).ok());
    
    let mut repl = ReplSession::new(config, working_directory, id).await?;
    repl.run().await
}
