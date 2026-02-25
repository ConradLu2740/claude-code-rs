# Claude Code RS

A Claude Code style AI programming assistant CLI tool built with Rust.

## Features

- 🤖 **Multi-LLM Support**: Zhipu AI, DeepSeek, Qwen, Moonshot, OpenAI, Anthropic
- 💬 **Streaming Chat**: Real-time streaming output with context memory
- 🔧 **Built-in Tools**: 9 powerful tools for file operations and web access
- 💾 **Session Management**: Persistent conversation storage
- 🔒 **Sandbox Execution**: Safe command execution with timeout control

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/claude-code-rs.git
cd claude-code-rs

# Build
cargo build --release

# The binary will be at ./target/release/ccode.exe
```

## Quick Start

```bash
# Set your API key
export CCODE_API_KEY="your_api_key"

# Start interactive chat
./target/release/ccode

# Or use a specific provider
export CCODE_PROVIDER="deepseek"
./target/release/ccode
```

## Available Commands

| Command | Description |
|---------|-------------|
| `ccode` | Start interactive chat |
| `ccode ask "question"` | Single question mode |
| `ccode --provider zhipu` | Use specific LLM provider |
| `ccode session list` | List saved sessions |
| `ccode tools` | List available tools |
| `ccode config` | Show configuration |

## Built-in Tools

| Tool | Description |
|------|-------------|
| `read` | Read file contents with line numbers |
| `write` | Write content to file |
| `edit` | Find and replace in file |
| `glob` | Find files by pattern |
| `grep` | Search content with regex |
| `ls` | List directory contents |
| `shell` | Execute shell commands |
| `web_search` | Search the web |
| `web_fetch` | Fetch webpage content |

## Configuration

Configuration file location (in order of priority):
1. `./.ccode/config.toml`
2. `~/.config/ccode/config.toml` (Linux/Mac)
3. `~/.ccode/config.toml` (Windows)

Generate default config:
```bash
ccode config --generate > config.toml
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CCODE_API_KEY` | API key for LLM provider |
| `CCODE_PROVIDER` | LLM provider (zhipu/deepseek/qwen/moonshot/openai/anthropic) |
| `CCODE_MODEL` | Model name |
| `CCODE_BASE_URL` | Custom API endpoint |

## Free LLM Providers

| Provider | Free Tier | Registration |
|----------|-----------|--------------|
| Zhipu AI | Daily free calls | https://open.bigmodel.cn |
| DeepSeek | Monthly free quota | https://platform.deepseek.com |
| Qwen | Free quota | https://dashscope.aliyun.com |

## License

MIT
