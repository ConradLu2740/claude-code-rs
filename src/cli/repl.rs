use crate::cli::output::OutputFormatter;
use crate::config::{AppConfig, LlmProvider};
use crate::core::{Conversation, ConversationBuilder, DEFAULT_SYSTEM_PROMPT, SessionManager};
use crate::llm::{LlmClient, LlmClientConfig, Message, StreamEvent, ToolCall};
use crate::tools::{create_default_registry, ExecutionContext, ToolRegistry};
use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

pub struct ReplSession {
    config: Arc<AppConfig>,
    conversation: Conversation,
    session_manager: SessionManager,
    client: Box<dyn LlmClient>,
    tool_registry: Arc<ToolRegistry>,
    formatter: OutputFormatter,
    working_directory: PathBuf,
    prompt_tokens: usize,
    completion_tokens: usize,
}

impl ReplSession {
    pub async fn new(
        config: AppConfig,
        working_directory: PathBuf,
        session_id: Option<Uuid>,
    ) -> Result<Self> {
        let config = Arc::new(config);
        
        let tool_registry = Arc::new(create_default_registry());
        
        let client = create_llm_client(&config)?;
        
        let mut session_manager = SessionManager::new(config.storage.session_dir.clone())?;
        
        let conversation = if let Some(id) = session_id {
            if let Some(session) = session_manager.load_session(id)? {
                let mut conv = ConversationBuilder::new(tool_registry.clone())
                    .system_prompt(DEFAULT_SYSTEM_PROMPT)
                    .build();
                conv.messages = session.messages;
                conv
            } else {
                return Err(anyhow!("Session not found: {}", id));
            }
        } else {
            session_manager.create_session(working_directory.clone())?;
            ConversationBuilder::new(tool_registry.clone())
                .system_prompt(DEFAULT_SYSTEM_PROMPT)
                .build()
        };

        let formatter = OutputFormatter::new();

        Ok(Self {
            config,
            conversation,
            session_manager,
            client,
            tool_registry,
            formatter,
            working_directory,
            prompt_tokens: 0,
            completion_tokens: 0,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.formatter.print_welcome();

        if let Some(session) = self.session_manager.current_session() {
            self.formatter.print_session_info(
                &session.id.to_string()[..8],
                session.messages.len(),
            );
        }

        loop {
            print!("{}", self.formatter.user_style.apply_to("\nYou: "));
            std::io::Write::flush(&mut std::io::stdout())?;

            let input = match self.read_input() {
                Some(input) => input,
                None => continue,
            };

            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }

            if input.starts_with('/') {
                if self.handle_command(input).await? {
                    break;
                }
                continue;
            }

            if let Err(e) = self.process_message(input).await {
                self.formatter.print_error(&e.to_string());
                self.conversation.clear();
                self.formatter.print_system("Conversation cleared due to error. Please try again.");
            }
        }

        Ok(())
    }

    fn read_input(&self) -> Option<String> {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => None,
            Ok(_) => Some(input),
            Err(_) => None,
        }
    }

    async fn handle_command(&mut self, input: &str) -> Result<bool> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts.get(0).map(|s| s.to_lowercase()).unwrap_or_default();

        match command.as_str() {
            "/exit" | "/quit" | "/q" => {
                self.formatter.print_system("Goodbye!");
                return Ok(true);
            }
            "/help" | "/h" | "/?" => {
                self.print_help();
            }
            "/clear" => {
                self.conversation.clear();
                self.formatter.print_success("Conversation cleared");
            }
            "/save" => {
                if let Some(session) = self.session_manager.current_session_mut() {
                    session.messages = self.conversation.messages.clone();
                    self.session_manager.save_current_session()?;
                    self.formatter.print_success("Session saved");
                }
            }
            "/sessions" => {
                let sessions = self.session_manager.list_sessions()?;
                self.formatter.print_system(&format!("Found {} sessions:", sessions.len()));
                for info in sessions.iter().take(10) {
                    println!(
                        "  {} - {} messages ({})",
                        info.id.to_string()[..8].to_string(),
                        info.message_count,
                        info.updated_at.format("%Y-%m-%d %H:%M")
                    );
                }
            }
            "/tools" => {
                let tools = self.tool_registry.list_tools();
                self.formatter.print_system(&format!("Available tools: {}", tools.join(", ")));
            }
            "/tokens" => {
                self.formatter.print_token_usage(self.prompt_tokens, self.completion_tokens);
            }
            _ => {
                self.formatter.print_error(&format!("Unknown command: {}", command));
                self.print_help();
            }
        }

        Ok(false)
    }

    fn print_help(&self) {
        println!();
        println!("Available commands:");
        println!("  /help, /h     - Show this help message");
        println!("  /exit, /q     - Exit the session");
        println!("  /clear        - Clear the conversation");
        println!("  /save         - Save the current session");
        println!("  /sessions     - List saved sessions");
        println!("  /tools        - List available tools");
        println!("  /tokens       - Show token usage");
        println!();
    }

    async fn process_message(&mut self, input: &str) -> Result<()> {
        if !input.is_empty() {
            self.conversation.add_user_message(input);
        }

        let messages = self.conversation.get_messages_for_api();
        let tools = self.conversation.get_tool_definitions();

        let stream = self.client.stream_complete(messages, tools).await?;

        let mut response_content = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut current_tool_id: Option<String> = None;
        let mut current_tool_name: Option<String> = None;
        let mut current_tool_args = String::new();

        self.formatter.print_assistant_stream("\nAssistant: ");

        let mut stream = stream;
        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::ContentDelta(delta)) => {
                    response_content.push_str(&delta);
                    self.formatter.print_assistant_stream(&delta);
                }
                Ok(StreamEvent::ToolCallStart { id, name }) => {
                    if let (Some(prev_id), Some(prev_name)) = (current_tool_id.take(), current_tool_name.take()) {
                        tool_calls.push(ToolCall::new(prev_id, prev_name, current_tool_args.clone()));
                    }
                    current_tool_id = Some(id);
                    current_tool_name = Some(name);
                    current_tool_args = String::new();
                }
                Ok(StreamEvent::ToolCallDelta { id, delta }) => {
                    if current_tool_id.as_ref() == Some(&id) {
                        current_tool_args.push_str(&delta);
                    }
                }
                Ok(StreamEvent::MessageStop) => {
                    if let (Some(id), Some(name)) = (current_tool_id.take(), current_tool_name.take()) {
                        tool_calls.push(ToolCall::new(id, name, current_tool_args.clone()));
                    }
                    break;
                }
                Ok(StreamEvent::Error(e)) => {
                    println!();
                    return Err(anyhow!("Stream error: {}", e));
                }
                Err(e) => {
                    println!();
                    return Err(anyhow!("Stream error: {}", e));
                }
            }
        }

        println!();

        if !tool_calls.is_empty() {
            let mut assistant_message = Message::assistant(response_content.clone());
            assistant_message.tool_calls = Some(tool_calls.clone());
            self.conversation.add_message(assistant_message);

            for tool_call in &tool_calls {
                if let Err(e) = self.execute_tool_call(tool_call).await {
                    self.formatter.print_error(&format!("Tool execution failed: {}", e));
                    self.conversation.add_tool_result(&tool_call.id, format!("Error: {}", e));
                }
            }

            Box::pin(self.process_message("")).await?;
        } else if !response_content.is_empty() {
            self.conversation.add_assistant_message(&response_content);
        }

        if let Some(session) = self.session_manager.current_session_mut() {
            session.messages = self.conversation.messages.clone();
            let _ = self.session_manager.save_current_session();
        }

        Ok(())
    }

    async fn execute_tool_call(&mut self, tool_call: &ToolCall) -> Result<()> {
        let tool_name = &tool_call.function.name;
        let tool_args_str = &tool_call.function.arguments;

        self.formatter.print_tool_call(tool_name, tool_args_str);

        let tool_input: serde_json::Value = if tool_args_str.is_empty() {
            serde_json::Value::Object(Default::default())
        } else {
            serde_json::from_str(tool_args_str)
                .with_context(|| format!("Failed to parse tool arguments: {}", tool_args_str))?
        };

        let ctx = ExecutionContext::new(self.working_directory.clone(), self.config.clone());

        let result = if let Some(tool) = self.tool_registry.get(tool_name) {
            tool.execute(tool_input, &ctx).await?
        } else {
            crate::tools::ToolResult::error(format!("Unknown tool: {}", tool_name))
        };

        self.formatter.print_tool_result(&result.output, result.success);

        let tool_result_content = if result.success {
            result.output
        } else {
            format!("Error: {}", result.error.unwrap_or_default())
        };

        self.conversation.add_tool_result(&tool_call.id, tool_result_content);

        Ok(())
    }
}

fn create_llm_client(config: &AppConfig) -> Result<Box<dyn LlmClient>> {
    let api_key = config.llm.api_key.clone()
        .ok_or_else(|| anyhow!("API key not configured. Set CCODE_API_KEY environment variable."))?;

    let base_url = config.llm.base_url.clone()
        .unwrap_or_else(|| config.llm.provider.default_base_url().to_string());

    let model = if config.llm.model.is_empty() {
        config.llm.provider.default_model().to_string()
    } else {
        config.llm.model.clone()
    };

    let client_config = LlmClientConfig {
        api_key,
        base_url,
        model,
        max_tokens: config.llm.max_tokens,
        temperature: config.llm.temperature,
        timeout_secs: config.llm.timeout_secs,
    };

    let provider_name = match config.llm.provider {
        LlmProvider::Zhipu => "zhipu",
        LlmProvider::DeepSeek => "deepseek",
        LlmProvider::Qwen => "qwen",
        LlmProvider::Moonshot => "moonshot",
        LlmProvider::OpenAI => "openai",
        LlmProvider::Anthropic => "anthropic",
    };

    crate::llm::openai::create_client(client_config, provider_name)
}
