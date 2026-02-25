# LLM Provider Documentation

## Overview

Claude Code RS supports multiple LLM providers through a unified trait-based interface. Each provider implements the `LlmClient` trait with streaming support.

## Supported Providers

| Provider | Status | Free Tier | Models |
|----------|--------|-----------|--------|
| Zhipu AI | ✅ Stable | Daily free calls | glm-4, glm-4-flash, glm-3-turbo |
| DeepSeek | ✅ Stable | Monthly quota | deepseek-chat, deepseek-coder |
| OpenAI | ✅ Stable | Paid | gpt-4, gpt-4-turbo, gpt-3.5-turbo |
| Qwen | 🚧 Planned | Free quota | qwen-turbo, qwen-plus |
| Moonshot | 🚧 Planned | Free quota | moonshot-v1 |
| Anthropic | 🚧 Planned | Paid | claude-3-opus, claude-3-sonnet |

## LlmClient Trait

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Returns the provider name (e.g., "zhipu", "deepseek")
    fn provider_name(&self) -> &str;
    
    /// Non-streaming completion
    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Message>;
    
    /// Streaming completion
    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<LlmStream>;
    
    /// Estimate token count for text
    fn count_tokens(&self, text: &str) -> usize;
}
```

## Configuration

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

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `CCODE_API_KEY` | API key for the provider | `id.secret` (Zhipu) |
| `CCODE_PROVIDER` | Provider name | `zhipu`, `deepseek`, `openai` |
| `CCODE_MODEL` | Model name | `glm-4-flash`, `gpt-4` |
| `CCODE_BASE_URL` | Custom API endpoint | `https://api.openai.com/v1` |

### Config File

```toml
[llm]
provider = "zhipu"
model = "glm-4-flash"
api_key = "your_api_key"  # Or use environment variable
base_url = ""  # Optional: custom endpoint
max_tokens = 4096
temperature = 0.7
timeout_secs = 120
```

## Zhipu AI

### Setup

1. Register at https://open.bigmodel.cn
2. Create an API Key (format: `id.secret`)
3. Configure:

```bash
export CCODE_PROVIDER="zhipu"
export CCODE_API_KEY="your_id.your_secret"
```

### Authentication

Zhipu AI uses JWT tokens generated from the API key:

```rust
fn generate_jwt_token(api_key: &str, exp_seconds: i64) -> Result<String> {
    let parts: Vec<&str> = api_key.split('.').collect();
    let (id, secret) = (parts[0], parts[1]);
    
    let header = Header::new(Algorithm::HS256);
    let claims = Claims {
        api_key: id.to_string(),
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64 + exp_seconds,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as i64,
    };
    
    encode(&header, &claims, &EncodingKey::new(secret.as_bytes()))
}
```

### Supported Models

| Model | Context | Description |
|-------|---------|-------------|
| `glm-4` | 128K | Latest flagship model |
| `glm-4-flash` | 128K | Fast, cost-effective |
| `glm-3-turbo` | 32K | Previous generation |

### API Endpoint

```
POST https://open.bigmodel.cn/api/paas/v4/chat/completions
```

## DeepSeek

### Setup

1. Register at https://platform.deepseek.com
2. Create an API Key
3. Configure:

```bash
export CCODE_PROVIDER="deepseek"
export CCODE_API_KEY="your_deepseek_key"
```

### Supported Models

| Model | Context | Description |
|-------|---------|-------------|
| `deepseek-chat` | 64K | General purpose |
| `deepseek-coder` | 64K | Code-specialized |

### API Endpoint

```
POST https://api.deepseek.com/v1/chat/completions
```

### Features

- OpenAI-compatible API
- Streaming support
- Function calling
- Low latency

## OpenAI-Compatible APIs

### Setup

```bash
export CCODE_PROVIDER="openai"
export CCODE_BASE_URL="https://api.openai.com/v1"
export CCODE_API_KEY="your_openai_key"
export CCODE_MODEL="gpt-4"
```

### Compatible Services

| Service | Base URL |
|---------|----------|
| OpenAI | `https://api.openai.com/v1` |
| Azure OpenAI | `https://your-resource.openai.azure.com/openai/deployments/your-deployment` |
| LocalAI | `http://localhost:8080/v1` |
| Ollama | `http://localhost:11434/v1` |
| vLLM | `http://localhost:8000/v1` |

### Supported Models

| Model | Context | Description |
|-------|---------|-------------|
| `gpt-4` | 8K | Most capable |
| `gpt-4-turbo` | 128K | Fast GPT-4 |
| `gpt-4o` | 128K | Multimodal |
| `gpt-3.5-turbo` | 16K | Cost-effective |

## Streaming Implementation

### Stream Event Types

```rust
pub enum StreamEvent {
    /// Text content delta
    ContentDelta(String),
    
    /// Tool call started
    ToolCallStart { id: String, name: String },
    
    /// Tool call arguments delta
    ToolCallDelta { id: String, delta: String },
    
    /// Message complete
    MessageStop,
    
    /// Error occurred
    Error(String),
}
```

### Streaming Flow

```
1. Client calls stream_complete()
   │
   ▼
2. Build request body with stream: true
   │
   ▼
3. Send POST request
   │
   ▼
4. Parse SSE (Server-Sent Events)
   │
   ├── data: {"choices":[{"delta":{"content":"Hello"}}]}
   │   └── Emit ContentDelta("Hello")
   │
   ├── data: {"choices":[{"delta":{"tool_calls":[...]}}]}
   │   └── Emit ToolCallStart / ToolCallDelta
   │
   └── data: [DONE]
       └── Emit MessageStop
```

### Example: Zhipu Streaming

```rust
async fn stream_complete(
    &self,
    messages: Vec<Message>,
    tools: Vec<ToolDefinition>,
) -> Result<LlmStream> {
    let token = self.generate_jwt_token(3600)?;
    
    let body = json!({
        "model": self.config.model,
        "messages": messages_to_openai_format(&messages),
        "tools": tools.iter().map(|t| t).collect::<Vec<_>>(),
        "stream": true,
    });
    
    let response = HTTP_CLIENT
        .post(&format!("{}/chat/completions", self.config.base_url))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;
    
    let stream = response.bytes_stream()
        .map(|result| {
            // Parse SSE and emit StreamEvent
        });
    
    Ok(Box::pin(stream))
}
```

## Function Calling

### Tool Definition Format

```rust
pub struct ToolDefinition {
    pub tool_type: String,  // "function"
    pub function: ToolFunction,
}

pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: Value,  // JSON Schema
}
```

### Example Tool Definition

```json
{
  "type": "function",
  "function": {
    "name": "read",
    "description": "Read file contents",
    "parameters": {
      "type": "object",
      "properties": {
        "file_path": {
          "type": "string",
          "description": "Path to the file"
        }
      },
      "required": ["file_path"]
    }
  }
}
```

### Tool Call Response

```json
{
  "role": "assistant",
  "content": null,
  "tool_calls": [
    {
      "id": "call_abc123",
      "type": "function",
      "function": {
        "name": "read",
        "arguments": "{\"file_path\": \"/src/main.rs\"}"
      }
    }
  ]
}
```

### Tool Result Message

```json
{
  "role": "tool",
  "tool_call_id": "call_abc123",
  "content": "1→fn main() {\n2→    println!(\"Hello\");\n3→}"
}
```

## Adding a New Provider

### Step 1: Create Provider File

```rust
// src/llm/my_provider.rs
use super::{LlmClient, LlmClientConfig, Message, StreamEvent, ToolDefinition};
use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;

pub struct MyProviderClient {
    config: LlmClientConfig,
}

impl MyProviderClient {
    pub fn new(config: LlmClientConfig) -> Result<Self> {
        Ok(Self { config })
    }
}

#[async_trait]
impl LlmClient for MyProviderClient {
    fn provider_name(&self) -> &str {
        "my_provider"
    }
    
    async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Message> {
        // Implement non-streaming completion
    }
    
    async fn stream_complete(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Result<LlmStream> {
        // Implement streaming completion
    }
    
    fn count_tokens(&self, text: &str) -> usize {
        // Estimate tokens (rough: chars / 4)
        text.len() / 4
    }
}
```

### Step 2: Register in mod.rs

```rust
// src/llm/mod.rs
pub mod my_provider;
```

### Step 3: Add to Factory

```rust
// src/llm/openai.rs
pub fn create_client(config: LlmClientConfig, provider: &str) -> Result<Box<dyn LlmClient>> {
    match provider {
        "zhipu" => Ok(Box::new(zhipu::ZhipuClient::new(config)?)),
        "deepseek" => Ok(Box::new(deepseek::DeepSeekClient::new(config)?)),
        "openai" => Ok(Box::new(OpenAIClient::new(config)?)),
        "my_provider" => Ok(Box::new(my_provider::MyProviderClient::new(config)?)),
        _ => Err(anyhow!("Unknown provider: {}", provider)),
    }
}
```

### Step 4: Add Configuration

```rust
// src/config/settings.rs
// Update default provider list if needed
```

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| 401 Unauthorized | Invalid API key | Check key format and validity |
| 400 Bad Request | Invalid request body | Check message format |
| 429 Rate Limited | Too many requests | Implement backoff |
| 500 Server Error | Provider issue | Retry with backoff |

### Error Response Example

```json
{
  "error": {
    "message": "Invalid API key",
    "type": "invalid_request_error",
    "code": "invalid_api_key"
  }
}
```

## Performance Optimization

### HTTP Connection Pooling

All providers use the shared HTTP client:

```rust
use crate::utils::HTTP_CLIENT;

let response = HTTP_CLIENT
    .post(&url)
    .json(&body)
    .send()
    .await?;
```

### Token Caching

For repeated completions, consider caching:
- System prompts
- Tool definitions
- Context that doesn't change

### Request Batching

For multiple independent requests:
```rust
let futures: Vec<_> = prompts
    .iter()
    .map(|p| client.complete(p.clone(), vec![]))
    .collect();

let results = futures::future::join_all(futures).await;
```

## Testing

### Mock Client

```rust
pub struct MockLlmClient {
    responses: VecDeque<Message>,
}

impl MockLlmClient {
    pub fn new(responses: Vec<Message>) -> Self {
        Self {
            responses: responses.into_iter().collect(),
        }
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    fn provider_name(&self) -> &str { "mock" }
    
    async fn complete(
        &self,
        _messages: Vec<Message>,
        _tools: Vec<ToolDefinition>,
    ) -> Result<Message> {
        Ok(self.responses.front().cloned().unwrap_or_default())
    }
    
    // ... other methods
}
```

### Integration Test

```rust
#[tokio::test]
#[ignore] // Requires API key
async fn test_zhipu_completion() {
    let config = LlmClientConfig {
        api_key: std::env::var("ZHIPU_API_KEY").unwrap(),
        base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
        model: "glm-4-flash".to_string(),
        ..Default::default()
    };
    
    let client = ZhipuClient::new(config).unwrap();
    let messages = vec![Message::user("Hello")];
    
    let response = client.complete(messages, vec![]).await.unwrap();
    assert!(!response.content.is_empty());
}
```
