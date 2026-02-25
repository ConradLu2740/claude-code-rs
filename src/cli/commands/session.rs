use crate::core::SessionManager;
use crate::cli::args::SessionAction;
use anyhow::Result;
use std::path::PathBuf;

pub fn run_session(action: SessionAction, storage_dir: PathBuf) -> Result<()> {
    let mut manager = SessionManager::new(storage_dir)?;

    match action {
        SessionAction::List => {
            let sessions = manager.list_sessions()?;
            
            if sessions.is_empty() {
                println!("No sessions found.");
                return Ok(());
            }

            println!("Found {} sessions:\n", sessions.len());
            println!("{:<12} {:<8} {:<10} {}", "ID", "Messages", "Updated", "Name");
            println!("{}", "-".repeat(60));

            for info in sessions {
                println!(
                    "{:<12} {:<8} {:<10} {}",
                    &info.id.to_string()[..8],
                    info.message_count,
                    info.updated_at.format("%m/%d %H:%M"),
                    info.name.unwrap_or_default()
                );
            }
        }
        SessionAction::Show { id } => {
            let uuid = uuid::Uuid::parse_str(&id)?;
            
            if let Some(session) = manager.load_session(uuid)? {
                println!("Session: {}", session.id);
                println!("Created: {}", session.created_at.format("%Y-%m-%d %H:%M:%S"));
                println!("Updated: {}", session.updated_at.format("%Y-%m-%d %H:%M:%S"));
                println!("Messages: {}", session.messages.len());
                println!("\nMessages:");
                println!("{}", "-".repeat(60));

                for (i, msg) in session.messages.iter().enumerate() {
                    println!("\n[{}] {:?}", i + 1, msg.role);
                    match &msg.content {
                        crate::llm::MessageContent::Text(text) => {
                            println!("{}", text);
                        }
                        crate::llm::MessageContent::Parts(parts) => {
                            for part in parts {
                                if let Some(text) = &part.text {
                                    println!("{}", text);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("Session not found: {}", id);
            }
        }
        SessionAction::Delete { id } => {
            let uuid = uuid::Uuid::parse_str(&id)?;
            
            if manager.delete_session(uuid)? {
                println!("Session deleted: {}", id);
            } else {
                println!("Session not found: {}", id);
            }
        }
        SessionAction::Export { id, format } => {
            let uuid = uuid::Uuid::parse_str(&id)?;
            
            if let Some(session) = manager.load_session(uuid)? {
                let output = match format.as_str() {
                    "json" => serde_json::to_string_pretty(&session)?,
                    "markdown" | "md" => {
                        let mut md = String::new();
                        md.push_str(&format!("# Session: {}\n\n", session.id));
                        md.push_str(&format!("Created: {}\n\n", session.created_at));
                        
                        for msg in &session.messages {
                            md.push_str(&format!("## {:?}\n\n", msg.role));
                            match &msg.content {
                                crate::llm::MessageContent::Text(text) => {
                                    md.push_str(text);
                                    md.push_str("\n\n");
                                }
                                crate::llm::MessageContent::Parts(parts) => {
                                    for part in parts {
                                        if let Some(text) = &part.text {
                                            md.push_str(text);
                                            md.push_str("\n");
                                        }
                                    }
                                    md.push_str("\n");
                                }
                            }
                        }
                        md
                    }
                    _ => {
                        println!("Unknown format: {}. Use 'json' or 'markdown'.", format);
                        return Ok(());
                    }
                };

                println!("{}", output);
            } else {
                println!("Session not found: {}", id);
            }
        }
    }

    Ok(())
}
