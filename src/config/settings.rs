use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub tools: ToolsConfig,
    pub storage: StorageConfig,
    pub indexing: IndexingConfig,
    pub sandbox: SandboxConfig,
    pub ui: UiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            tools: ToolsConfig::default(),
            storage: StorageConfig::default(),
            indexing: IndexingConfig::default(),
            sandbox: SandboxConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LlmProvider {
    #[default]
    #[serde(rename = "zhipu")]
    Zhipu,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "qwen")]
    Qwen,
    #[serde(rename = "moonshot")]
    Moonshot,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
}

impl LlmProvider {
    pub fn default_base_url(&self) -> &'static str {
        match self {
            LlmProvider::Zhipu => "https://open.bigmodel.cn/api/paas/v4",
            LlmProvider::DeepSeek => "https://api.deepseek.com",
            LlmProvider::Qwen => "https://dashscope.aliyuncs.com/api/v1",
            LlmProvider::Moonshot => "https://api.moonshot.cn/v1",
            LlmProvider::OpenAI => "https://api.openai.com/v1",
            LlmProvider::Anthropic => "https://api.anthropic.com",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            LlmProvider::Zhipu => "glm-4-flash",
            LlmProvider::DeepSeek => "deepseek-chat",
            LlmProvider::Qwen => "qwen-turbo",
            LlmProvider::Moonshot => "moonshot-v1-8k",
            LlmProvider::OpenAI => "gpt-4o-mini",
            LlmProvider::Anthropic => "claude-3-haiku-20240307",
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::default(),
            api_key: None,
            base_url: None,
            model: String::new(),
            max_tokens: 4096,
            temperature: 0.7,
            timeout_secs: 120,
            retry_count: 3,
            retry_delay_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
    pub permissions: HashMap<String, ToolPermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub allowed: bool,
    pub ask_confirmation: bool,
    pub max_output_size: Option<usize>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            enabled: vec![
                "read".to_string(),
                "write".to_string(),
                "edit".to_string(),
                "glob".to_string(),
                "grep".to_string(),
                "shell".to_string(),
                "search".to_string(),
            ],
            disabled: vec![],
            permissions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub session_dir: PathBuf,
    pub index_dir: PathBuf,
    pub max_session_size: usize,
    pub max_sessions: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        let base = directories::ProjectDirs::from("com", "claude-code", "ccode")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        
        Self {
            data_dir: base.clone(),
            session_dir: base.join("sessions"),
            index_dir: base.join("index"),
            max_session_size: 100,
            max_sessions: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub enabled: bool,
    pub file_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub embedding_model: String,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            file_patterns: vec![
                "**/*.rs".to_string(),
                "**/*.go".to_string(),
                "**/*.py".to_string(),
                "**/*.js".to_string(),
                "**/*.ts".to_string(),
                "**/*.jsx".to_string(),
                "**/*.tsx".to_string(),
                "**/*.java".to_string(),
                "**/*.c".to_string(),
                "**/*.cpp".to_string(),
                "**/*.h".to_string(),
                "**/*.md".to_string(),
                "**/*.json".to_string(),
                "**/*.yaml".to_string(),
                "**/*.yml".to_string(),
                "**/*.toml".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
                "**/.venv/**".to_string(),
                "**/venv/**".to_string(),
            ],
            max_file_size: 1024 * 1024,
            chunk_size: 512,
            chunk_overlap: 50,
            embedding_model: "text-embedding-3-small".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
    pub max_execution_time_secs: u64,
    pub max_output_size: usize,
    pub allowed_paths: Vec<PathBuf>,
    pub blocked_paths: Vec<PathBuf>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_commands: vec![],
            blocked_commands: vec![
                "rm".to_string(),
                "rmdir".to_string(),
                "del".to_string(),
                "format".to_string(),
                "mkfs".to_string(),
                "dd".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
                "init".to_string(),
                "systemctl".to_string(),
            ],
            max_execution_time_secs: 60,
            max_output_size: 1024 * 1024,
            allowed_paths: vec![],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
                PathBuf::from("C:\\Windows\\System32"),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_token_count: bool,
    pub show_timing: bool,
    pub stream_output: bool,
    pub code_highlight: bool,
    pub markdown_render: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            show_token_count: true,
            show_timing: true,
            stream_output: true,
            code_highlight: true,
            markdown_render: true,
        }
    }
}
