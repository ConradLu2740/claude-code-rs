use crate::llm::Message;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
    pub metadata: SessionMetadata,
    pub working_directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetadata {
    pub total_tokens: usize,
    pub message_count: usize,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub tags: Vec<String>,
}

impl Session {
    pub fn new(working_directory: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
            metadata: SessionMetadata::default(),
            working_directory,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn add_message(&mut self, message: Message) {
        self.updated_at = Utc::now();
        self.messages.push(message);
        self.metadata.message_count = self.messages.len();
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

pub struct SessionManager {
    storage_dir: PathBuf,
    current_session: Option<Session>,
}

impl SessionManager {
    pub fn new(storage_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&storage_dir)
            .with_context(|| "Failed to create session storage directory")?;
        
        Ok(Self {
            storage_dir,
            current_session: None,
        })
    }

    pub fn create_session(&mut self, working_directory: PathBuf) -> Result<&Session> {
        let session = Session::new(working_directory);
        self.current_session = Some(session);
        self.save_current_session()?;
        Ok(self.current_session.as_ref().unwrap())
    }

    pub fn load_session(&mut self, id: Uuid) -> Result<Option<Session>> {
        let session_file = self.storage_dir.join(format!("{}.json", id));
        
        if !session_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&session_file)
            .with_context(|| format!("Failed to read session file: {:?}", session_file))?;
        
        let session: Session = serde_json::from_str(&content)
            .with_context(|| "Failed to parse session file")?;
        
        Ok(Some(session))
    }

    pub fn save_session(&self, session: &Session) -> Result<()> {
        let session_file = self.storage_dir.join(format!("{}.json", session.id));
        
        let content = serde_json::to_string_pretty(session)
            .with_context(|| "Failed to serialize session")?;
        
        std::fs::write(&session_file, content)
            .with_context(|| format!("Failed to write session file: {:?}", session_file))?;
        
        Ok(())
    }

    pub fn save_current_session(&self) -> Result<()> {
        if let Some(ref session) = self.current_session {
            self.save_session(session)?;
        }
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let mut sessions = Vec::new();
        
        for entry in std::fs::read_dir(&self.storage_dir)
            .with_context(|| "Failed to read session directory")?
        {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        sessions.push(SessionInfo {
                            id: session.id,
                            name: session.name,
                            created_at: session.created_at,
                            updated_at: session.updated_at,
                            message_count: session.messages.len(),
                        });
                    }
                }
            }
        }

        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(sessions)
    }

    pub fn delete_session(&self, id: Uuid) -> Result<bool> {
        let session_file = self.storage_dir.join(format!("{}.json", id));
        
        if session_file.exists() {
            std::fs::remove_file(&session_file)
                .with_context(|| "Failed to delete session file")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn current_session(&self) -> Option<&Session> {
        self.current_session.as_ref()
    }

    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        self.current_session.as_mut()
    }

    pub fn set_current_session(&mut self, session: Session) {
        self.current_session = Some(session);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}
