use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ccode")]
#[command(author = "Developer")]
#[command(version = "0.1.0")]
#[command(about = "A Claude Code style AI programming assistant CLI tool", long_about = None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(short, long, default_value = ".")]
    pub workdir: PathBuf,

    #[arg(long, env = "CCODE_API_KEY")]
    pub api_key: Option<String>,

    #[arg(long, env = "CCODE_PROVIDER", default_value = "zhipu")]
    pub provider: String,

    #[arg(long, env = "CCODE_MODEL")]
    pub model: Option<String>,

    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub no_stream: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Start an interactive chat session")]
    Chat {
        #[arg(short, long)]
        session: Option<String>,
        
        #[arg(long)]
        system: Option<String>,
    },

    #[command(about = "Send a single message and get a response")]
    Ask {
        #[arg(required = true)]
        message: Vec<String>,
        
        #[arg(short, long)]
        json: bool,
    },

    #[command(about = "Manage sessions")]
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    #[command(about = "Index the codebase for semantic search")]
    Index {
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        #[arg(long)]
        force: bool,
    },

    #[command(about = "Search the indexed codebase")]
    Search {
        #[arg(required = true)]
        query: Vec<String>,
        
        #[arg(short, long, default_value = "5")]
        top_k: usize,
    },

    #[command(about = "Execute a tool directly")]
    Tool {
        #[arg(required = true)]
        name: String,
        
        #[arg(short, long)]
        input: Option<String>,
    },

    #[command(about = "Show configuration")]
    Config {
        #[arg(long)]
        generate: bool,
    },

    #[command(about = "List available tools")]
    Tools,
}

#[derive(Debug, Subcommand)]
pub enum SessionAction {
    #[command(about = "List all sessions")]
    List,

    #[command(about = "Show a specific session")]
    Show {
        #[arg(required = true)]
        id: String,
    },

    #[command(about = "Delete a session")]
    Delete {
        #[arg(required = true)]
        id: String,
    },

    #[command(about = "Export a session")]
    Export {
        #[arg(required = true)]
        id: String,
        
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

impl CliArgs {
    pub fn message_to_string(&self) -> Option<String> {
        if let Some(Commands::Ask { message, .. }) = &self.command {
            Some(message.join(" "))
        } else {
            None
        }
    }
}
