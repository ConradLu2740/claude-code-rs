# Documentation Index

## Overview

Welcome to the Claude Code RS documentation. This directory contains technical documentation for developers and contributors.

## Documents

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | System architecture and design patterns |
| [TOOLS.md](TOOLS.md) | Tool system documentation and custom tool development |
| [LLM_PROVIDERS.md](LLM_PROVIDERS.md) | LLM provider integration and configuration |
| [API_REFERENCE.md](API_REFERENCE.md) | API reference for all public types and functions |

## Quick Links

### For Users

- [README.md](../README.md) - Installation and usage guide
- [LLM_PROVIDERS.md](LLM_PROVIDERS.md) - How to configure LLM providers

### For Developers

- [ARCHITECTURE.md](ARCHITECTURE.md) - Understand the codebase structure
- [TOOLS.md](TOOLS.md) - Create custom tools
- [API_REFERENCE.md](API_REFERENCE.md) - API documentation

### For Contributors

- [ARCHITECTURE.md](ARCHITECTURE.md) - Architecture overview
- [TOOLS.md](TOOLS.md) - Adding new tools
- [LLM_PROVIDERS.md](LLM_PROVIDERS.md) - Adding new LLM providers

## Project Structure

```
claude-code-rs/
├── src/
│   ├── cli/           # CLI interface
│   ├── core/          # Core logic (conversation, session)
│   ├── llm/           # LLM clients
│   ├── tools/         # Tool system
│   ├── config/        # Configuration
│   └── utils/         # Utilities
├── docs/              # This directory
├── Cargo.toml         # Dependencies
└── README.md          # Main documentation
```

## Getting Started

1. Read [README.md](../README.md) for installation
2. Configure your LLM provider using [LLM_PROVIDERS.md](LLM_PROVIDERS.md)
3. Explore [ARCHITECTURE.md](ARCHITECTURE.md) to understand the system
4. Check [API_REFERENCE.md](API_REFERENCE.md) for detailed API docs

## Contributing

See [README.md](../README.md#contributing) for contribution guidelines.

When adding new features:
1. Tools → See [TOOLS.md](TOOLS.md)
2. LLM Providers → See [LLM_PROVIDERS.md](LLM_PROVIDERS.md)
3. Update relevant documentation
