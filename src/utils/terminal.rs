use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
};
use std::io::{self, Stdout};

pub struct TerminalGuard {
    stdout: Stdout,
}

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout();
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self { stdout })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub fn get_terminal_size() -> (u16, u16) {
    crossterm::terminal::size().unwrap_or((80, 24))
}

pub fn clear_screen() {
    let _ = crossterm::execute!(io::stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All));
}
