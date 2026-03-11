use crate::commands::config::AppState;
use crate::commands::sidecar::SidecarState;
use crate::models::message::{Message, SidecarRequest};
use crate::models::session::{Session, SessionStatus};
use crate::services::WorkspaceManager;
use tauri::State;

#[tauri::command]
pub async fn create_session(
    title: String,
    app_state: State<'_, AppState>,
) -> Result<Session, String> {
    let workspace_path = app_state.workspace_path.lock().unwrap().clone();
    let session = Session::new(title, workspace_path.clone());

    let manager = WorkspaceManager::new(&workspace_path);
    manager
        .create_session_dir(&session.id)
        .map_err(|e| e.to_string())?;

    manager
        .save_session_metadata(&session)
        .map_err(|e| e.to_string())?;

    Ok(session)
}

#[tauri::command]
pub async fn send_message(
    session_id: String,
    content: String,
    sidecar_state: State<'_, SidecarState>,
) -> Result<(), String> {
    let manager_lock = sidecar_state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        let request = SidecarRequest {
            session_id,
            message: Message::UserMessage { content },
        };

        let response = manager
            .send_message(request)
            .await
            .map_err(|e| e.to_string())?;

        if !response.success {
            return Err(response.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(())
    } else {
        Err("Sidecar manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn pause_session(
    session_id: String,
    sidecar_state: State<'_, SidecarState>,
) -> Result<(), String> {
    let manager_lock = sidecar_state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        let response = manager
            .pause_session(&session_id)
            .await
            .map_err(|e| e.to_string())?;

        if !response.success {
            return Err(response.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(())
    } else {
        Err("Sidecar manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn resume_session(
    session_id: String,
    sidecar_state: State<'_, SidecarState>,
) -> Result<(), String> {
    let manager_lock = sidecar_state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        let response = manager
            .resume_session(&session_id)
            .await
            .map_err(|e| e.to_string())?;

        if !response.success {
            return Err(response.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        Ok(())
    } else {
        Err("Sidecar manager not initialized".to_string())
    }
}
