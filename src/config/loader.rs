use crate::config::AppConfig;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct ConfigLoader {
    config_path: Option<PathBuf>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self { config_path: None }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            config_path: Some(path),
        }
    }

    pub fn load(&mut self) -> Result<AppConfig> {
        self.load_with_validation(true)
    }

    pub fn load_with_validation(&mut self, validate_api_key: bool) -> Result<AppConfig> {
        let mut config = AppConfig::default();

        let config_dirs = self.get_config_dirs();
        
        for dir in &config_dirs {
            let config_file = dir.join("config.toml");
            if config_file.exists() {
                self.config_path = Some(config_file.clone());
                self.merge_file(&config_file, &mut config)?;
                break;
            }
        }

        self.merge_env(&mut config)?;

        if validate_api_key {
            self.validate(&config)?;
        }

        Ok(config)
    }

    fn get_config_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = vec![];

        if let Ok(cwd) = std::env::current_dir() {
            dirs.push(cwd.join(".ccode"));
        }

        if let Some(project_dirs) = directories::ProjectDirs::from("com", "claude-code", "ccode") {
            dirs.push(project_dirs.config_dir().to_path_buf());
        }

        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(home).join(".config").join("ccode"));
        }

        if let Ok(home) = std::env::var("USERPROFILE") {
            dirs.push(PathBuf::from(home).join(".ccode"));
        }

        dirs
    }

    fn merge_file(&self, path: &PathBuf, config: &mut AppConfig) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let file_config: PartialConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        if let Some(llm) = file_config.llm {
            config.llm = llm;
        }
        if let Some(tools) = file_config.tools {
            config.tools = tools;
        }
        if let Some(storage) = file_config.storage {
            config.storage = storage;
        }
        if let Some(indexing) = file_config.indexing {
            config.indexing = indexing;
        }
        if let Some(sandbox) = file_config.sandbox {
            config.sandbox = sandbox;
        }
        if let Some(ui) = file_config.ui {
            config.ui = ui;
        }

        Ok(())
    }

    fn merge_env(&self, config: &mut AppConfig) -> Result<()> {
        if let Ok(api_key) = std::env::var("CCODE_API_KEY") {
            config.llm.api_key = Some(api_key);
        }

        if let Ok(base_url) = std::env::var("CCODE_BASE_URL") {
            config.llm.base_url = Some(base_url);
        }

        if let Ok(provider) = std::env::var("CCODE_PROVIDER") {
            config.llm.provider = match provider.to_lowercase().as_str() {
                "zhipu" => crate::config::LlmProvider::Zhipu,
                "deepseek" => crate::config::LlmProvider::DeepSeek,
                "qwen" => crate::config::LlmProvider::Qwen,
                "moonshot" => crate::config::LlmProvider::Moonshot,
                "openai" => crate::config::LlmProvider::OpenAI,
                "anthropic" => crate::config::LlmProvider::Anthropic,
                _ => config.llm.provider.clone(),
            };
        }

        if let Ok(model) = std::env::var("CCODE_MODEL") {
            config.llm.model = model;
        }

        Ok(())
    }

    fn validate(&self, config: &AppConfig) -> Result<()> {
        if config.llm.api_key.is_none() {
            anyhow::bail!(
                "API key is required. Set CCODE_API_KEY environment variable or configure in config file."
            );
        }

        if config.llm.model.is_empty() {
            let model = config.llm.provider.default_model().to_string();
            tracing::info!("Using default model: {}", model);
        }

        Ok(())
    }

    pub fn get_config_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }

    pub fn generate_default_config() -> String {
        let config = AppConfig::default();
        toml::to_string_pretty(&config).unwrap_or_default()
    }
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct PartialConfig {
    llm: Option<crate::config::LlmConfig>,
    tools: Option<crate::config::ToolsConfig>,
    storage: Option<crate::config::StorageConfig>,
    indexing: Option<crate::config::IndexingConfig>,
    sandbox: Option<crate::config::SandboxConfig>,
    ui: Option<crate::config::UiConfig>,
}

pub fn ensure_directories(config: &AppConfig) -> Result<()> {
    std::fs::create_dir_all(&config.storage.data_dir)
        .with_context(|| "Failed to create data directory")?;
    std::fs::create_dir_all(&config.storage.session_dir)
        .with_context(|| "Failed to create session directory")?;
    std::fs::create_dir_all(&config.storage.index_dir)
        .with_context(|| "Failed to create index directory")?;
    Ok(())
}
