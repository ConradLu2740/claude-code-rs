# Architecture Documentation

## System Overview

Claude Code RS is a CLI-based AI programming assistant built with Rust. It follows a layered architecture pattern with clear separation of concerns.

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI Layer (clap)                          │
│  args.rs │ repl.rs │ commands/                                    │
├─────────────────────────────────────────────────────────────────┤
│                        Core Layer                                 │
│  conversation.rs │ context.rs │ session.rs                        │
├─────────────────────────────────────────────────────────────────┤
│                        LLM Layer                                  │
│  client.rs (trait) │ zhipu.rs │ deepseek.rs │ openai.rs           │
├─────────────────────────────────────────────────────────────────┤
│                        Tool Layer                                 │
│  executor.rs │ registry.rs │ builtin/                             │
├─────────────────────────────────────────────────────────────────┤
│                        Infrastructure Layer                       │
│  config/ │ utils/ (http, terminal)                                │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. CLI Layer (`src/cli/`)

**Purpose**: Handle command-line interface and user interaction.

| File | Responsibility |
|------|----------------|
| `args.rs` | CLI argument parsing using `clap` derive macros |
| `repl.rs` | Interactive REPL session with readline support |
| `commands/` | Individual command handlers (chat, ask, session, etc.) |

**Key Types**:
```rust
pub enum Commands {
    Chat { session_id: Option<String>, system_prompt: Option<String> },
    Ask { message: String },
    Session { action: SessionAction },
    Tool { name: String, input: Option<String> },
    Config { generate: bool },
    Tools,
    Index { path: PathBuf, force: bool },
    Search { query: String, top_k: usize },
}
```

### 2. Core Layer (`src/core/`)

**Purpose**: Manage conversation state, context window, and session persistence.

#### Conversation (`conversation.rs`)

```rust
pub struct Conversation {
    pub id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    tool_registry: Arc<ToolRegistry>,
    system_prompt: Option<String>,
}
```

**Key Methods**:
- `add_message()` - Add a message to the conversation
- `get_messages_for_api()` - Get messages formatted for LLM API
- `get_tool_definitions()` - Get tool definitions for function calling
- `truncate_messages()` - Truncate old messages to fit context window

#### Context Manager (`context.rs`)

```rust
pub struct ContextManager {
    max_tokens: usize,
    strategy: ContextStrategy,
    current_tokens: usize,
}

pub enum ContextStrategy {
    TruncateOldest,
    SlidingWindow { window_size: usize },
    KeepSystemAndRecent { recent_count: usize },
}
```

**Token Estimation**: Uses character-based approximation (~4 chars per token).

#### Session (`session.rs`)

```rust
pub struct Session {
    pub id: Uuid,
    pub name: Option<String>,
    pub messages: Vec<Message>,
    pub metadata: SessionMetadata,
    pub working_directory: PathBuf,
}

pub struct SessionManager {
    storage_dir: PathBuf,
    current_session: Option<Session>,
}
```

**Storage Format**: JSON files in `~/.ccode/sessions/{uuid}.json`

### 3. LLM Layer (`src/llm/`)

**Purpose**: Abstract LLM provider interactions with streaming support.

#### Client Trait (`client.rs`)

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

pub struct LlmClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub timeout_secs: u64,
}
```

#### Message Types (`message.rs`)

```rust
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

pub struct ToolCall {
    pub id: String,
    pub call_type: String,
    pub function: FunctionCall,
}
```

#### Streaming Events (`message.rs`)

```rust
pub enum StreamEvent {
    ContentDelta(String),
    ToolCallStart { id: String, name: String },
    ToolCallDelta { id: String, delta: String },
    MessageStop,
    Error(String),
}
```

### 4. Tool Layer (`src/tools/`)

**Purpose**: Provide extensible tool system for LLM function calling.

#### Tool Executor Trait (`executor.rs`)

```rust
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult>;
    fn schema(&self) -> ToolSchema;
    fn requires_confirmation(&self) -> bool { false }
}

pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

pub struct ExecutionContext {
    pub working_directory: PathBuf,
    pub config: Arc<AppConfig>,
    pub session_id: Option<Uuid>,
}
```

#### Tool Registry (`registry.rs`)

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolExecutor>>,
}

impl ToolRegistry {
    pub fn register(&mut self, tool: Box<dyn ToolExecutor>);
    pub fn get(&self, name: &str) -> Option<&Box<dyn ToolExecutor>>;
    pub fn get_all_definitions(&self) -> Vec<ToolDefinition>;
    pub fn list_tools(&self) -> Vec<&str>;
}
```

### 5. Config Layer (`src/config/`)

**Purpose**: Load and manage application configuration.

#### Configuration Structure (`settings.rs`)

```rust
pub struct AppConfig {
    pub llm: LlmConfig,
    pub tools: ToolsConfig,
    pub storage: StorageConfig,
    pub indexing: IndexingConfig,
    pub sandbox: SandboxConfig,
    pub ui: UiConfig,
}

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

#### Loading Priority (`loader.rs`)

1. Command-line arguments (highest)
2. Environment variables
3. Project config (`./.ccode/config.toml`)
4. User config (`~/.config/ccode/config.toml`)
5. Default values (lowest)

### 6. Utilities (`src/utils/`)

#### HTTP Client (`http.rs`)

```rust
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(120))
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client")
});
```

**Features**:
- Connection pooling (10 idle connections per host)
- 60-second idle timeout
- 120-second request timeout
- Shared across all LLM clients and web tools

## Data Flow

### Chat Flow

```
User Input
    │
    ▼
┌─────────────┐
│   REPL      │
└─────┬───────┘
      │
      ▼
┌─────────────┐     ┌──────────────┐
│ Conversation│────►│ ToolRegistry │
└─────┬───────┘     └──────────────┘
      │                    │
      ▼                    │
┌─────────────┐            │
│ LlmClient   │◄───────────┘
└─────┬───────┘
      │
      ▼
┌─────────────┐
│ API Request │
│ (streaming) │
└─────┬───────┘
      │
      ▼
┌─────────────┐
│ Tool Call?  │──Yes──►┌─────────────┐
└─────┬───────┘        │ ToolExecutor│
      │                └──────┬──────┘
      No                      │
      │                       ▼
      ▼                ┌─────────────┐
┌─────────────┐        │ Tool Result │
│ Response    │◄───────└─────────────┘
└─────────────┘
```

### Tool Execution Flow

```
LLM Response (tool_calls)
    │
    ▼
┌─────────────────┐
│ Parse ToolCall  │
└───────┬─────────┘
        │
        ▼
┌─────────────────┐
│ Lookup in       │
│ ToolRegistry    │
└───────┬─────────┘
        │
        ▼
┌─────────────────┐
│ requires_       │──Yes──►┌─────────────┐
│ confirmation?   │        │ User Prompt │
└───────┬─────────┘        └──────┬──────┘
        │                         │
        No                        │
        │                         │
        ▼◄────────────────────────┘
┌─────────────────┐
│ execute()       │
│ with input JSON │
└───────┬─────────┘
        │
        ▼
┌─────────────────┐
│ ToolResult      │
│ (success/error) │
└───────┬─────────┘
        │
        ▼
┌─────────────────┐
│ Add as Tool     │
│ Message to      │
│ Conversation    │
└─────────────────┘
```

## Performance Optimizations

### 1. HTTP Connection Pooling

- Single shared `reqwest::Client` instance
- Connection reuse across requests
- Configurable pool size and timeout

### 2. Regex Caching

```rust
static HTML_TAG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<[^>]+>").unwrap()
});
```

### 3. Lazy Static Initialization

- HTTP client initialized on first use
- Regex patterns compiled once

## Error Handling

### Current Approach

- Uses `anyhow` for error propagation
- `.context()` for adding error context
- `?` operator for automatic conversion

### Error Flow

```
Tool Execution Error
    │
    ▼
ToolResult::error(message)
    │
    ▼
Returned to LLM as tool result
    │
    ▼
LLM can retry or inform user
```

## Security

### Sandbox Configuration

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

### Default Blocked Commands

- `rm -rf`
- `format` (Windows)
- `del` (Windows)
- `mkfs`
- `dd`

### API Key Security

- Loaded from environment variables or config files
- Never logged or displayed
- Config files should have restricted permissions

## Extensibility

### Adding a New Tool

1. Create `src/tools/builtin/my_tool.rs`
2. Implement `ToolExecutor` trait
3. Register in `create_default_registry()`

### Adding a New LLM Provider

1. Create `src/llm/my_provider.rs`
2. Implement `LlmClient` trait
3. Add to `create_client()` factory function
4. Update configuration options

## Future Improvements

### Planned Features

1. **Code Indexing**: Tree-sitter based parsing
2. **Semantic Search**: Vector embeddings with sqlite-vec
3. **Plugin System**: Dynamic tool loading
4. **Custom Error Types**: Structured error handling
5. **Metrics**: Prometheus-compatible metrics
6. **Tracing**: OpenTelemetry integration

### Architecture Changes

1. **Event-driven Tool Execution**: Async event bus
2. **Plugin Architecture**: WASM-based plugins
3. **Multi-turn Tool Orchestration**: Complex tool chains
