# Tool System Documentation

## Overview

The tool system provides a pluggable architecture for LLM function calling. Tools allow the AI to interact with the filesystem, execute commands, and access web resources.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      ToolRegistry                            │
│  HashMap<String, Box<dyn ToolExecutor>>                      │
├─────────────────────────────────────────────────────────────┤
│                      ToolExecutor Trait                      │
│  - execute(input: Value, ctx: &ExecutionContext)            │
│  - schema() -> ToolSchema                                    │
│  - requires_confirmation() -> bool                           │
├─────────────────────────────────────────────────────────────┤
│                      Built-in Tools                          │
│  read │ write │ edit │ glob │ grep │ ls │ shell │ web      │
└─────────────────────────────────────────────────────────────┘
```

## ToolExecutor Trait

```rust
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute the tool with given input
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult>;
    
    /// Return the tool's JSON schema for LLM function calling
    fn schema(&self) -> ToolSchema;
    
    /// Whether this tool requires user confirmation before execution
    fn requires_confirmation(&self) -> bool {
        false
    }
}
```

## ToolSchema

```rust
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}
```

## ExecutionContext

```rust
pub struct ExecutionContext {
    pub working_directory: PathBuf,
    pub config: Arc<AppConfig>,
    pub session_id: Option<Uuid>,
}
```

## ToolResult

```rust
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(output: impl Into<String>) -> Self;
    pub fn error(error: impl Into<String>) -> Self;
    pub fn to_json(&self) -> Value;
}
```

## Built-in Tools

### 1. Read Tool (`read`)

Read file contents with line numbers.

**Schema**:
```json
{
  "name": "read",
  "description": "Read file contents with line numbers",
  "parameters": {
    "type": "object",
    "properties": {
      "file_path": {
        "type": "string",
        "description": "The absolute path to the file to read"
      },
      "offset": {
        "type": "integer",
        "description": "Line number to start reading from"
      },
      "limit": {
        "type": "integer",
        "description": "Maximum number of lines to read"
      }
    },
    "required": ["file_path"]
  }
}
```

**Example**:
```json
{"file_path": "/path/to/file.rs", "offset": 10, "limit": 50}
```

**Output**:
```
    10→pub fn main() {
    11→    println!("Hello");
    12→}
```

### 2. Write Tool (`write`)

Write content to a file (creates or overwrites).

**Schema**:
```json
{
  "name": "write",
  "description": "Write content to a file",
  "parameters": {
    "type": "object",
    "properties": {
      "file_path": {
        "type": "string",
        "description": "The absolute path to the file to write"
      },
      "content": {
        "type": "string",
        "description": "The content to write to the file"
      }
    },
    "required": ["file_path", "content"]
  }
}
```

**Example**:
```json
{
  "file_path": "/path/to/new_file.rs",
  "content": "fn main() {\n    println!(\"Hello\");\n}"
}
```

### 3. Edit Tool (`edit`)

Find and replace text in a file.

**Schema**:
```json
{
  "name": "edit",
  "description": "Find and replace text in a file",
  "parameters": {
    "type": "object",
    "properties": {
      "file_path": {
        "type": "string",
        "description": "The absolute path to the file to edit"
      },
      "old_str": {
        "type": "string",
        "description": "The text to find and replace"
      },
      "new_str": {
        "type": "string",
        "description": "The replacement text"
      }
    },
    "required": ["file_path", "old_str", "new_str"]
  }
}
```

**Example**:
```json
{
  "file_path": "/path/to/file.rs",
  "old_str": "println!(\"Hello\")",
  "new_str": "println!(\"World\")"
}
```

**Behavior**:
- Only replaces the first occurrence
- Returns error if `old_str` not found
- Creates backup before editing

### 4. Glob Tool (`glob`)

Find files matching a pattern.

**Schema**:
```json
{
  "name": "glob",
  "description": "Find files matching a glob pattern",
  "parameters": {
    "type": "object",
    "properties": {
      "pattern": {
        "type": "string",
        "description": "The glob pattern (e.g., **/*.rs)"
      },
      "path": {
        "type": "string",
        "description": "The directory to search in"
      }
    },
    "required": ["pattern"]
  }
}
```

**Example**:
```json
{"pattern": "**/*.rs", "path": "/path/to/project"}
```

**Output**:
```
src/main.rs
src/lib.rs
src/utils/mod.rs
```

### 5. Grep Tool (`grep`)

Search file contents with regex.

**Schema**:
```json
{
  "name": "grep",
  "description": "Search file contents with regex",
  "parameters": {
    "type": "object",
    "properties": {
      "pattern": {
        "type": "string",
        "description": "The regex pattern to search for"
      },
      "path": {
        "type": "string",
        "description": "The directory or file to search in"
      },
      "glob": {
        "type": "string",
        "description": "File pattern to filter (e.g., *.rs)"
      }
    },
    "required": ["pattern"]
  }
}
```

**Example**:
```json
{"pattern": "fn main", "path": "src", "glob": "*.rs"}
```

**Output**:
```
src/main.rs:1:fn main() {
src/bin/app.rs:5:fn main() {
```

### 6. LS Tool (`ls`)

List directory contents.

**Schema**:
```json
{
  "name": "ls",
  "description": "List directory contents",
  "parameters": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "The directory path to list"
      }
    },
    "required": ["path"]
  }
}
```

**Example**:
```json
{"path": "/path/to/project"}
```

**Output**:
```
- src/
  - main.rs
  - lib.rs
- Cargo.toml
- README.md
```

### 7. Shell Tool (`shell`)

Execute shell commands in a sandboxed environment.

**Schema**:
```json
{
  "name": "shell",
  "description": "Execute a shell command",
  "parameters": {
    "type": "object",
    "properties": {
      "command": {
        "type": "string",
        "description": "The shell command to execute"
      },
      "cwd": {
        "type": "string",
        "description": "Working directory for the command"
      },
      "timeout": {
        "type": "integer",
        "description": "Timeout in seconds (default 60)"
      }
    },
    "required": ["command"]
  }
}
```

**Example**:
```json
{"command": "cargo build --release", "timeout": 120}
```

**Security**:
- Requires user confirmation (`requires_confirmation() = true`)
- Blocked commands: `rm -rf`, `format`, `del`, `mkfs`, `dd`
- Configurable timeout and output limits
- Blocked paths protection

### 8. Web Search Tool (`web_search`)

Search the web using search engines.

**Schema**:
```json
{
  "name": "web_search",
  "description": "Search the web for information",
  "parameters": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "The search query"
      },
      "num_results": {
        "type": "integer",
        "description": "Number of results to return (default 5)"
      }
    },
    "required": ["query"]
  }
}
```

**Example**:
```json
{"query": "Rust async programming best practices", "num_results": 5}
```

**Output**:
```json
[
  {
    "title": "Async Programming in Rust",
    "url": "https://example.com/async-rust",
    "snippet": "Learn async programming..."
  }
]
```

### 9. Web Fetch Tool (`web_fetch`)

Fetch and parse webpage content.

**Schema**:
```json
{
  "name": "web_fetch",
  "description": "Fetch and parse webpage content",
  "parameters": {
    "type": "object",
    "properties": {
      "url": {
        "type": "string",
        "description": "The URL to fetch"
      }
    },
    "required": ["url"]
  }
}
```

**Example**:
```json
{"url": "https://doc.rust-lang.org/book/"}
```

**Output**:
Markdown-formatted content extracted from the webpage.

## Creating Custom Tools

### Step 1: Implement ToolExecutor

```rust
// src/tools/builtin/my_tool.rs
use crate::tools::{ExecutionContext, ToolExecutor, ToolResult, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct MyTool {
    config: MyToolConfig,
}

#[derive(Debug, Deserialize)]
struct MyToolInput {
    param1: String,
    param2: Option<i32>,
}

#[async_trait]
impl ToolExecutor for MyTool {
    async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<ToolResult> {
        // Parse input
        let args: MyToolInput = serde_json::from_value(input)
            .context("Invalid input for my_tool")?;
        
        // Access context
        let workdir = &ctx.working_directory;
        let config = &ctx.config;
        
        // Do work
        let result = do_something(&args.param1)?;
        
        Ok(ToolResult::success(result))
    }
    
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "my_tool",
            "Description of what my_tool does",
            json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "description": "Description of param1"
                    },
                    "param2": {
                        "type": "integer",
                        "description": "Optional parameter"
                    }
                },
                "required": ["param1"]
            })
        )
    }
    
    fn requires_confirmation(&self) -> bool {
        true  // If the tool is potentially destructive
    }
}
```

### Step 2: Register the Tool

```rust
// src/tools/builtin/mod.rs
pub mod my_tool;

// In create_default_registry():
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    // ... existing tools ...
    registry.register(Box::new(my_tool::MyTool::new()));
    
    registry
}
```

### Step 3: Add Configuration (Optional)

```rust
// src/config/settings.rs
pub struct ToolsConfig {
    pub enabled: Vec<String>,
    pub my_tool: MyToolConfig,  // Add custom config
}
```

## Tool Execution Flow

```
1. LLM returns tool_calls in response
   │
   ▼
2. Parse ToolCall { id, function: { name, arguments } }
   │
   ▼
3. Look up tool in ToolRegistry by name
   │
   ▼
4. Check requires_confirmation()
   │
   ├── Yes ──► Prompt user for confirmation
   │           │
   │           ├── Approved ──► Continue
   │           └── Denied ──► Return ToolResult::error("User denied")
   │
   ▼
5. Parse arguments JSON
   │
   ▼
6. Create ExecutionContext
   │
   ▼
7. Call tool.execute(input, ctx)
   │
   ▼
8. Return ToolResult
   │
   ▼
9. Create Message::tool_result(tool_call_id, result)
   │
   ▼
10. Add to conversation, continue LLM loop
```

## Error Handling

### Tool Errors

```rust
// Input validation error
let args: MyToolInput = match serde_json::from_value(input) {
    Ok(args) => args,
    Err(e) => return Ok(ToolResult::error(format!("Invalid input: {}", e))),
};

// Execution error
match do_something(&args.param1) {
    Ok(result) => Ok(ToolResult::success(result)),
    Err(e) => Ok(ToolResult::error(format!("Operation failed: {}", e))),
}
```

### Error Propagation

Tool errors are returned to the LLM as tool result messages, allowing the LLM to:
1. Understand what went wrong
2. Retry with different parameters
3. Inform the user
4. Try alternative approaches

## Security Considerations

### Sandboxing

- Shell commands run with configurable restrictions
- Blocked commands and paths
- Timeout limits
- Output size limits

### Input Validation

- All inputs are JSON-parsed with strict typing
- File paths are validated for traversal attacks
- URLs are validated for allowed schemes

### User Confirmation

Tools that modify state should return `true` from `requires_confirmation()`:
- `write` - Creates/overwrites files
- `edit` - Modifies files
- `shell` - Executes arbitrary commands

## Performance

### HTTP Connection Pooling

Web tools use the shared HTTP client:
```rust
use crate::utils::HTTP_CLIENT;

let response = HTTP_CLIENT.get(url).send().await?;
```

### Regex Caching

```rust
use once_cell::sync::Lazy;
use regex::Regex;

static MY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"pattern").unwrap()
});
```

## Testing Tools

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_my_tool() {
        let tool = MyTool::new();
        let ctx = ExecutionContext::new(
            PathBuf::from("."),
            Arc::new(AppConfig::default())
        );
        
        let input = json!({"param1": "test"});
        let result = tool.execute(input, &ctx).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}
```
