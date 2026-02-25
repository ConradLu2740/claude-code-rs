# Claude Code RS

<div align="center">

**A Claude Code style AI programming assistant CLI tool built with Rust**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[English](#english) | [中文文档](#中文文档)

</div>

---

<a name="english"></a>

## Features

- **Multi-LLM Support**: Zhipu AI, DeepSeek, OpenAI-compatible APIs
- **Streaming Chat**: Real-time streaming output with context memory
- **Built-in Tools**: 9 powerful tools for file operations and web access
- **Session Management**: Persistent conversation storage and recovery
- **Sandbox Execution**: Safe command execution with timeout control
- **Performance Optimized**: HTTP connection pooling, regex caching

## Architecture

```
claude-code-rs/
├── src/
│   ├── cli/                 # CLI interface
│   │   ├── args.rs          # Argument parsing (clap)
│   │   ├── repl.rs          # Interactive REPL
│   │   └── commands/        # Command handlers
│   ├── llm/                 # LLM clients
│   │   ├── client.rs        # Trait definition
│   │   ├── zhipu.rs         # Zhipu AI client
│   │   ├── deepseek.rs      # DeepSeek client
│   │   └── openai.rs        # OpenAI-compatible client
│   ├── tools/               # Tool system
│   │   ├── executor.rs      # Tool execution
│   │   ├── registry.rs      # Tool registry
│   │   └── builtin/         # Built-in tools
│   ├── core/                # Core logic
│   │   ├── conversation.rs  # Conversation management
│   │   ├── context.rs       # Context window management
│   │   └── session.rs       # Session persistence
│   ├── config/              # Configuration
│   │   ├── settings.rs      # Config structures
│   │   └── loader.rs        # Config loading
│   └── utils/               # Utilities
│       ├── http.rs          # Shared HTTP client
│       └── terminal.rs      # Terminal utilities
└── target/release/ccode.exe # Compiled binary
```

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/ConradLu2740/claude-code-rs.git
cd claude-code-rs

# Build release version
cargo build --release

# Binary location: ./target/release/ccode.exe
```

### Requirements

- Rust 1.75+
- Windows / macOS / Linux

## Quick Start

```bash
# Set your API key (Zhipu AI example)
export CCODE_API_KEY="your_api_key_id.your_api_key_secret"

# Start interactive chat
./target/release/ccode

# Or specify provider
export CCODE_PROVIDER="deepseek"
export CCODE_API_KEY="your_deepseek_key"
./target/release/ccode
```

## Commands

| Command | Description |
|---------|-------------|
| `ccode` | Start interactive REPL session |
| `ccode ask "question"` | Single question mode |
| `ccode --provider zhipu` | Use specific LLM provider |
| `ccode --model glm-4-flash` | Use specific model |
| `ccode session list` | List saved sessions |
| `ccode session show <id>` | Show session details |
| `ccode session delete <id>` | Delete a session |
| `ccode tools` | List available tools |
| `ccode config` | Show current configuration |
| `ccode config --generate` | Generate default config |

## Built-in Tools

| Tool | Description | Example |
|------|-------------|---------|
| `read` | Read file contents | `{"file_path": "/path/to/file.rs"}` |
| `write` | Write content to file | `{"file_path": "/path/to/file.rs", "content": "..."}` |
| `edit` | Find and replace in file | `{"file_path": "...", "old_str": "...", "new_str": "..."}` |
| `glob` | Find files by pattern | `{"pattern": "**/*.rs"}` |
| `grep` | Search content with regex | `{"pattern": "fn main", "path": "src"}` |
| `ls` | List directory contents | `{"path": "/path/to/dir"}` |
| `shell` | Execute shell commands | `{"command": "cargo build"}` |
| `web_search` | Search the web | `{"query": "Rust async programming"}` |
| `web_fetch` | Fetch webpage content | `{"url": "https://example.com"}` |

## Configuration

### Config File Locations (Priority Order)

1. `./.ccode/config.toml` (Project level)
2. `~/.config/ccode/config.toml` (Linux/macOS)
3. `~/.ccode/config.toml` (Windows)

### Generate Default Config

```bash
ccode config --generate > .ccode/config.toml
```

### Config Example

```toml
[llm]
provider = "zhipu"
model = "glm-4-flash"
timeout_secs = 120

[tools]
enabled = ["read", "write", "edit", "glob", "grep", "ls", "shell", "web_search", "web_fetch"]

[storage]
session_dir = "~/.ccode/sessions"

[sandbox]
enabled = true
max_execution_time_secs = 60
blocked_commands = ["rm -rf", "format", "del"]

[ui]
streaming = true
show_token_usage = true
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CCODE_API_KEY` | API key for LLM provider | - |
| `CCODE_PROVIDER` | LLM provider | `zhipu` |
| `CCODE_MODEL` | Model name | Provider default |
| `CCODE_BASE_URL` | Custom API endpoint | Provider default |

## LLM Providers

### Free Tier Providers

| Provider | Free Quota | Get API Key |
|----------|------------|-------------|
| **Zhipu AI** | Daily free calls | https://open.bigmodel.cn |
| **DeepSeek** | Monthly free quota | https://platform.deepseek.com |

### Zhipu AI Setup

1. Register at https://open.bigmodel.cn
2. Get API Key (format: `id.secret`)
3. Set environment variable:
   ```bash
   export CCODE_API_KEY="your_id.your_secret"
   export CCODE_PROVIDER="zhipu"
   ```

### DeepSeek Setup

1. Register at https://platform.deepseek.com
2. Get API Key
3. Set environment variable:
   ```bash
   export CCODE_API_KEY="your_deepseek_key"
   export CCODE_PROVIDER="deepseek"
   ```

### OpenAI-Compatible APIs

```bash
export CCODE_PROVIDER="openai"
export CCODE_BASE_URL="https://api.openai.com/v1"
export CCODE_API_KEY="your_openai_key"
export CCODE_MODEL="gpt-4"
```

## Development

### Project Structure

- **CLI Layer** (`cli/`): Command-line interface using clap
- **LLM Layer** (`llm/`): LLM client implementations with streaming support
- **Tool Layer** (`tools/`): Extensible tool system with async execution
- **Core Layer** (`core/`): Conversation and session management
- **Config Layer** (`config/`): Configuration loading and validation

### Add New Tool

1. Create file `src/tools/builtin/my_tool.rs`:

```rust
use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct MyTool;

#[async_trait]
impl ToolExecutor for MyTool {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult> {
        // Implementation
        Ok(ToolResult::success("Done"))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new("my_tool", "Description", json!({
            "type": "object",
            "properties": { /* ... */ }
        }))
    }
}
```

2. Register in `src/tools/builtin/mod.rs`:

```rust
pub mod my_tool;
// In create_default_registry():
registry.register("my_tool", Box::new(my_tool::MyTool));
```

### Add New LLM Provider

1. Create file `src/llm/my_provider.rs`:

```rust
use super::{LlmClient, LlmClientConfig, Message, StreamChunk};
use async_trait::async_trait;

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
    // Implement required methods
}
```

2. Register in `src/llm/mod.rs` and `openai.rs::create_client()`

### Build & Test

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run
cargo run -- [args]

# Check
cargo check
cargo clippy
```

## Roadmap

- [ ] Code indexing with Tree-sitter
- [ ] Semantic search with vector embeddings
- [ ] More LLM providers (Qwen, Moonshot, Anthropic)
- [ ] Unit tests and integration tests
- [ ] Plugin system for custom tools
- [ ] Web UI interface

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'feat: add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<a name="中文文档"></a>

## 中文文档

### 功能特性

- **多 LLM 支持**: 智谱 AI、DeepSeek、OpenAI 兼容 API
- **流式对话**: 实时流式输出，支持上下文记忆
- **内置工具**: 9 个强大的文件操作和网络访问工具
- **会话管理**: 持久化对话存储和恢复
- **沙箱执行**: 安全的命令执行，支持超时控制
- **性能优化**: HTTP 连接池、正则表达式缓存

### 快速开始

```bash
# 设置 API Key（智谱 AI 示例）
export CCODE_API_KEY="your_api_key_id.your_api_key_secret"

# 启动交互式对话
./target/release/ccode
```

### 免费大模型 API

| 平台 | 免费额度 | 注册地址 |
|------|----------|----------|
| **智谱 AI** | 每日免费调用 | https://open.bigmodel.cn |
| **DeepSeek** | 每月免费额度 | https://platform.deepseek.com |

### 常用命令

```bash
ccode                    # 启动交互式 REPL
ccode ask "你的问题"      # 单次问答模式
ccode session list       # 列出保存的会话
ccode tools              # 列出可用工具
ccode config             # 显示当前配置
```

### 工具列表

| 工具 | 功能 |
|------|------|
| `read` | 读取文件内容 |
| `write` | 写入文件 |
| `edit` | 查找替换文件内容 |
| `glob` | 按模式查找文件 |
| `grep` | 正则搜索内容 |
| `ls` | 列出目录内容 |
| `shell` | 执行 Shell 命令 |
| `web_search` | 网络搜索 |
| `web_fetch` | 获取网页内容 |

### 开发路线

- [ ] Tree-sitter 代码索引
- [ ] 向量嵌入语义搜索
- [ ] 更多 LLM 提供商
- [ ] 单元测试和集成测试
- [ ] 插件系统
- [ ] Web UI 界面

### 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## Acknowledgments

- Inspired by [Claude Code](https://claude.ai/code)
- Built with [Tokio](https://tokio.rs/), [Clap](https://docs.rs/clap/), [Reqwest](https://docs.rs/reqwest/)
