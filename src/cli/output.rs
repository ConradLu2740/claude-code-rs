use console::Style;
use std::io::{self, Write};
use termimad::MadSkin;

pub struct OutputFormatter {
    skin: MadSkin,
    pub user_style: Style,
    assistant_style: Style,
    system_style: Style,
    error_style: Style,
    success_style: Style,
    tool_style: Style,
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter {
    pub fn new() -> Self {
        Self {
            skin: MadSkin::default_dark(),
            user_style: Style::new().cyan().bold(),
            assistant_style: Style::new().green(),
            system_style: Style::new().yellow(),
            error_style: Style::new().red().bold(),
            success_style: Style::new().green().bold(),
            tool_style: Style::new().blue(),
        }
    }

    pub fn print_user(&self, message: &str) {
        println!("{}", self.user_style.apply_to(format!("You: {}", message)));
    }

    pub fn print_assistant(&self, message: &str) {
        println!();
        self.skin.print_text(message);
        println!();
    }

    pub fn print_assistant_stream(&self, delta: &str) {
        print!("{}", delta);
        io::stdout().flush().ok();
    }

    pub fn print_system(&self, message: &str) {
        println!("{}", self.system_style.apply_to(format!("⚙️  {}", message)));
    }

    pub fn print_error(&self, message: &str) {
        eprintln!("{}", self.error_style.apply_to(format!("❌ Error: {}", message)));
    }

    pub fn print_success(&self, message: &str) {
        println!("{}", self.success_style.apply_to(format!("✅ {}", message)));
    }

    pub fn print_tool_call(&self, name: &str, input: &str) {
        println!();
        println!(
            "{}",
            self.tool_style.apply_to(format!("🔧 Tool: {}", name))
        );
        if !input.is_empty() {
            println!("   Input: {}", input);
        }
    }

    pub fn print_tool_result(&self, result: &str, success: bool) {
        if success {
            println!("{}", self.success_style.apply_to("   Result:"));
        } else {
            println!("{}", self.error_style.apply_to("   Error:"));
        }
        
        for line in result.lines().take(20) {
            println!("     {}", line);
        }
        
        if result.lines().count() > 20 {
            println!("     ... (truncated)");
        }
        println!();
    }

    pub fn print_divider(&self) {
        println!("{}", "─".repeat(60));
    }

    pub fn print_welcome(&self) {
        println!();
        println!("{}", self.assistant_style.apply_to("╔════════════════════════════════════════════════════════════╗"));
        println!("{}", self.assistant_style.apply_to("║                                                            ║"));
        println!("{}", self.assistant_style.apply_to("║   🤖 Claude Code RS - AI Programming Assistant             ║"));
        println!("{}", self.assistant_style.apply_to("║                                                            ║"));
        println!("{}", self.assistant_style.apply_to("║   Type your message and press Enter to chat                ║"));
        println!("{}", self.assistant_style.apply_to("║   /help - Show available commands                          ║"));
        println!("{}", self.assistant_style.apply_to("║   /exit - Exit the session                                 ║"));
        println!("{}", self.assistant_style.apply_to("║                                                            ║"));
        println!("{}", self.assistant_style.apply_to("╚════════════════════════════════════════════════════════════╝"));
        println!();
    }

    pub fn print_token_usage(&self, prompt_tokens: usize, completion_tokens: usize) {
        let total = prompt_tokens + completion_tokens;
        println!(
            "{}",
            self.system_style.apply_to(format!(
                "📊 Tokens: {} prompt + {} completion = {} total",
                prompt_tokens, completion_tokens, total
            ))
        );
    }

    pub fn print_session_info(&self, id: &str, message_count: usize) {
        println!(
            "{}",
            self.system_style.apply_to(format!(
                "📁 Session: {} ({} messages)",
                id, message_count
            ))
        );
    }
}
