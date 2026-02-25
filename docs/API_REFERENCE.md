# API Reference

## Core Types

### Message

```rust
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
    pub name: Option<String>,
}
```

#### Constructors

| Method | Description |
|--------|-------------|
| `Message::user(content)` | Create user message |
| `Message::assistant(content)` | Create assistant message |
| `Message::system(content)` | Create system message |
| `Message::tool_result(id, content)` | Create tool result message |
| `with_tool_calls(calls)` | Add tool calls to message |

#### Example

```rust
let user_msg = Message::user("Hello, how are you?");
let system_msg = Message::system("You are a helpful assistant.");
let tool_result = Message::tool_result("call_123", "File contents...");
```

### Role

```rust
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}
```

### MessageContent

```rust
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}
```

### ToolCall

```rust
pub struct ToolCall {
    pub id: String,
    pub call_type: String,
    pub function: FunctionCall,
}

pub struct FunctionCall {
    pub name: String,
    pub arguments: String,  // JSON string
}
```

#### Constructor

```rust
let call = ToolCall::new("call_123", "read", r#"{"file_path": "/src/main.rs"}"#);
```

### ToolDefinition

```rust
pub struct ToolDefinition {
    pub tool_type: String,
    pub function: ToolFunction,
}

pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}
```

#### Constructor

```rust
let def = ToolDefinition::new(
    "read",
    "Read file contents",
    json!({
        "type": "object",
        "properties": {
            "file_path": { "type": "string" }
        },
        "required": ["file_path"]
    })
);
```

### StreamEvent

```rust
pub enum StreamEvent {
    ContentDelta(String),
    ToolCallStart { id: String, name: String },
    ToolCallDelta { id: String, delta: String },
    MessageStop,
    Error(String),
}
```

### Usage

```rust
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}
```

## LLM Client API

### LlmClient Trait

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    fn provider_name(&self) -> &str;
    
    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Message>;
    
    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<LlmStream>;
    
    fn count_tokens(&self, text: &str) -> usize;
}
```

### LlmClientConfig

```rust
pub struct LlmClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub timeout_secs: u64,
}
```

### create_client Factory

```rust
pub fn create_client(config: LlmClientConfig, provider: &str) -> Result<Box<dyn LlmClient>>;
```

#### Example

```rust
let config = LlmClientConfig {
    api_key: env::var("CCODE_API_KEY")?,
    base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
    model: "glm-4-flash".to_string(),
    max_tokens: 4096,
    temperature: 0.7,
    timeout_secs: 120,
};

let client = create_client(config, "zhipu")?;
```

## Tool System API

### ToolExecutor Trait

```rust
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult>;
    fn schema(&self) -> ToolSchema;
    fn requires_confirmation(&self) -> bool { false }
}
```

### ToolSchema

```rust
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}
```

#### Constructors

```rust
// Full constructor
let schema = ToolSchema::new("name", "description", json!({...}));

// Simple constructor for string properties
let schema = ToolSchema::simple(
    "name",
    "description",
    &[
        ("param1", "Description 1", true),
        ("param2", "Description 2", false),
    ]
);
```

### ToolResult

```rust
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}
```

#### Constructors

```rust
let success = ToolResult::success("Operation completed");
let error = ToolResult::error("Something went wrong");
```

#### Methods

| Method | Description |
|--------|-------------|
| `to_json(&self)` | Convert to JSON Value |
| `Display` | Format for output |

### ExecutionContext

```rust
pub struct ExecutionContext {
    pub working_directory: PathBuf,
    pub config: Arc<AppConfig>,
    pub session_id: Option<Uuid>,
}
```

#### Constructors

```rust
let ctx = ExecutionContext::new(PathBuf::from("."), Arc::new(config));
let ctx = ctx.with_session(session_id);
```

### ToolRegistry

```rust
pub struct ToolRegistry {
    // Internal HashMap
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new()` | Create empty registry |
| `register(tool)` | Register a tool |
| `get(name)` | Get tool by name |
| `get_all_definitions()` | Get all tool definitions |
| `list_tools()` | List tool names |
| `has_tool(name)` | Check if tool exists |

#### Example

```rust
let mut registry = ToolRegistry::new();
registry.register(Box::new(ReadTool::new()));

let tool = registry.get("read");
let definitions = registry.get_all_definitions();
```

## Conversation API

### Conversation

```rust
pub struct Conversation {
    pub id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new(registry)` | Create new conversation |
| `with_system_prompt(prompt)` | Set system prompt |
| `with_id(id)` | Set conversation ID |
| `add_message(msg)` | Add message |
| `add_user_message(content)` | Add user message |
| `add_assistant_message(content)` | Add assistant message |
| `add_tool_result(id, result)` | Add tool result |
| `get_messages_for_api()` | Get messages for LLM API |
| `get_tool_definitions()` | Get tool definitions |
| `clear()` | Clear all messages |
| `truncate_messages(max)` | Truncate to max messages |

### ConversationBuilder

```rust
let conv = ConversationBuilder::new(registry)
    .system_prompt("You are helpful")
    .with_message(Message::user("Hi"))
    .build();
```

## Session API

### Session

```rust
pub struct Session {
    pub id: Uuid,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
    pub metadata: SessionMetadata,
    pub working_directory: PathBuf,
}
```

### SessionMetadata

```rust
pub struct SessionMetadata {
    pub total_tokens: usize,
    pub message_count: usize,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub tags: Vec<String>,
}
```

### SessionManager

```rust
pub struct SessionManager {
    // Internal state
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new(storage_dir)` | Create manager |
| `create_session(workdir)` | Create new session |
| `load_session(id)` | Load session by ID |
| `save_session(session)` | Save session |
| `list_sessions()` | List all sessions |
| `delete_session(id)` | Delete session |
| `current_session()` | Get current session |
| `set_current_session(session)` | Set current session |

### SessionInfo

```rust
pub struct SessionInfo {
    pub id: Uuid,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}
```

## Context Management API

### ContextManager

```rust
pub struct ContextManager {
    max_tokens: usize,
    strategy: ContextStrategy,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new(max_tokens, strategy)` | Create manager |
| `manage(messages)` | Truncate messages to fit |
| `estimate_tokens(messages)` | Estimate total tokens |
| `current_tokens()` | Get current token count |
| `remaining_tokens()` | Get remaining tokens |
| `summary(messages)` | Get context summary |

### ContextStrategy

```rust
pub enum ContextStrategy {
    TruncateOldest,
    SlidingWindow { window_size: usize },
    KeepSystemAndRecent { recent_count: usize },
}
```

## Configuration API

### AppConfig

```rust
pub struct AppConfig {
    pub llm: LlmConfig,
    pub tools: ToolsConfig,
    pub storage: StorageConfig,
    pub indexing: IndexingConfig,
    pub sandbox: SandboxConfig,
    pub ui: UiConfig,
}
```

### LlmConfig

```rust
pub struct LlmConfig {
    pub provider: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: usize,
    pub temperature: f32,
    pub timeout_secs: u64,
}
```

### ToolsConfig

```rust
pub struct ToolsConfig {
    pub enabled: Vec<String>,
}
```

### StorageConfig

```rust
pub struct StorageConfig {
    pub session_dir: PathBuf,
}
```

### SandboxConfig

```rust
pub struct SandboxConfig {
    pub enabled: bool,
    pub allowed_commands: Vec<String>,
    pub blocked_commands: Vec<String>,
    pub max_execution_time_secs: u64,
    pub max_output_size: usize,
    pub blocked_paths: Vec<PathBuf>,
}
```

### UiConfig

```rust
pub struct UiConfig {
    pub streaming: bool,
    pub show_token_usage: bool,
    pub theme: String,
}
```

### ConfigLoader

```rust
pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> Result<AppConfig>;
    pub fn load_from_file(path: &Path) -> Result<AppConfig>;
    pub fn generate_default() -> String;
}
```

## Utility Functions

### messages_to_openai_format

```rust
pub fn messages_to_openai_format(messages: &[Message]) -> Vec<Value>;
```

Convert messages to OpenAI API format.

### HTTP Client

```rust
pub static HTTP_CLIENT: Lazy<Client>;

pub fn get_http_client() -> &'static Client;
```

Get the shared HTTP client instance.

## Error Types

### anyhow::Error

All errors are propagated using `anyhow::Error` with context:

```rust
result.context("Failed to read file")?;
```

### Common Error Patterns

```rust
// IO errors
std::fs::read_to_string(&path)
    .with_context(|| format!("Failed to read file: {:?}", path))?;

// JSON errors
serde_json::from_str::<T>(&content)
    .with_context(|| "Failed to parse JSON")?;

// API errors
if !response.status().is_success() {
    let error_text = response.text().await?;
    anyhow::bail!("API error: {}", error_text);
}
```

## Constants

### DEFAULT_SYSTEM_PROMPT

```rust
pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are an AI programming assistant..."#;
```

### Default Values

| Config | Default |
|--------|---------|
| `max_tokens` | 4096 |
| `temperature` | 0.7 |
| `timeout_secs` | 120 |
| `max_execution_time_secs` | 60 |
| `max_output_size` | 10000 lines |
| `session_dir` | `~/.ccode/sessions` |
