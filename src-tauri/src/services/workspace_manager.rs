use crate::models::session::Session;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct WorkspaceManager {
    workspace_path: PathBuf,
}

impl WorkspaceManager {
    pub fn new(workspace_path: &str) -> Self {
        Self {
            workspace_path: PathBuf::from(workspace_path),
        }
    }

    pub fn init_workspace(&self) -> Result<()> {
        fs::create_dir_all(&self.workspace_path)
            .context("Failed to create workspace directory")?;

        let sessions_dir = self.workspace_path.join("sessions");
        fs::create_dir_all(&sessions_dir)
            .context("Failed to create sessions directory")?;

        let config_dir = self.workspace_path.join("config");
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        Ok(())
    }

    pub fn create_session_dir(&self, session_id: &str) -> Result<PathBuf> {
        let session_dir = self.workspace_path.join("sessions").join(session_id);
        fs::create_dir_all(&session_dir)
            .context("Failed to create session directory")?;

        Ok(session_dir)
    }

    pub fn save_session_metadata(&self, session: &Session) -> Result<()> {
        let session_dir = self.create_session_dir(&session.id)?;
        let metadata_path = session_dir.join("session.md");

        let metadata = format!(
            r#"---
id: {}
title: {}
created_at: {}
updated_at: {}
status: {:?}
---

# {}

**Status**: {:?}
**Created**: {}
**Updated**: {}
"#,
            session.id,
            session.title,
            session.created_at,
            session.updated_at,
            session.status,
            session.title,
            session.status,
            chrono::DateTime::from_timestamp(session.created_at, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S"),
            chrono::DateTime::from_timestamp(session.updated_at, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S"),
        );

        fs::write(metadata_path, metadata)
            .context("Failed to write session metadata")?;

        Ok(())
    }

    pub fn append_to_history(&self, session_id: &str, content: &str) -> Result<()> {
        let session_dir = self.workspace_path.join("sessions").join(session_id);
        let history_path = session_dir.join("history.md");

        let mut history = if history_path.exists() {
            fs::read_to_string(&history_path)
                .context("Failed to read history file")?
        } else {
            String::from("# Session History\n\n")
        };

        history.push_str(content);
        history.push_str("\n\n---\n\n");

        fs::write(history_path, history)
            .context("Failed to write history file")?;

        Ok(())
    }

    pub fn save_plan(&self, session_id: &str, plan: &str) -> Result<()> {
        let session_dir = self.workspace_path.join("sessions").join(session_id);
        let plan_path = session_dir.join("plan.md");

        let content = format!("# Current Plan\n\n{}", plan);

        fs::write(plan_path, content)
            .context("Failed to write plan file")?;

        Ok(())
    }

    pub fn load_session_history(&self, session_id: &str) -> Result<String> {
        let history_path = self.workspace_path
            .join("sessions")
            .join(session_id)
            .join("history.md");

        if !history_path.exists() {
            return Ok(String::new());
        }

        fs::read_to_string(history_path)
            .context("Failed to read history file")
    }

    pub fn list_sessions(&self) -> Result<Vec<String>> {
        let sessions_dir = self.workspace_path.join("sessions");

        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();

        for entry in fs::read_dir(sessions_dir)
            .context("Failed to read sessions directory")?
        {
            let entry = entry.context("Failed to read directory entry")?;
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    sessions.push(name.to_string());
                }
            }
        }

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::session::SessionStatus;
    use tempfile::TempDir;

    #[test]
    fn test_init_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path().to_str().unwrap());

        manager.init_workspace().unwrap();

        assert!(temp_dir.path().join("sessions").exists());
        assert!(temp_dir.path().join("config").exists());
    }

    #[test]
    fn test_save_session_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path().to_str().unwrap());
        manager.init_workspace().unwrap();

        let session = Session::new(
            "Test Session".to_string(),
            temp_dir.path().to_str().unwrap().to_string(),
        );

        manager.save_session_metadata(&session).unwrap();

        let metadata_path = temp_dir
            .path()
            .join("sessions")
            .join(&session.id)
            .join("session.md");

        assert!(metadata_path.exists());
    }

    #[test]
    fn test_append_to_history() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path().to_str().unwrap());
        manager.init_workspace().unwrap();

        let session_id = "test-session";
        manager.create_session_dir(session_id).unwrap();

        manager.append_to_history(session_id, "Test message").unwrap();
        manager.append_to_history(session_id, "Another message").unwrap();

        let history = manager.load_session_history(session_id).unwrap();
        assert!(history.contains("Test message"));
        assert!(history.contains("Another message"));
    }
}
