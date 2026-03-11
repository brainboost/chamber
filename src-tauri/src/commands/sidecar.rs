use crate::services::SidecarManager;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SidecarState {
    pub manager: Arc<Mutex<Option<SidecarManager>>>,
    pub sidecar_path: String,
}

#[tauri::command]
pub async fn start_sidecar(state: State<'_, SidecarState>) -> Result<(), String> {
    let manager_lock = state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        manager
            .start(&state.sidecar_path)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Sidecar manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn stop_sidecar(state: State<'_, SidecarState>) -> Result<(), String> {
    let manager_lock = state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        manager.stop().await.map_err(|e| e.to_string())
    } else {
        Err("Sidecar manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn restart_sidecar(state: State<'_, SidecarState>) -> Result<(), String> {
    let manager_lock = state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        manager
            .restart(&state.sidecar_path)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Sidecar manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn health_check_sidecar(state: State<'_, SidecarState>) -> Result<bool, String> {
    let manager_lock = state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        manager.health_check().await.map_err(|e| e.to_string())
    } else {
        Err("Sidecar manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_websocket_url(state: State<'_, SidecarState>) -> Result<String, String> {
    let manager_lock = state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        Ok(manager.get_websocket_url())
    } else {
        Err("Sidecar manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn is_sidecar_running(state: State<'_, SidecarState>) -> Result<bool, String> {
    let manager_lock = state.manager.lock().await;

    if let Some(manager) = manager_lock.as_ref() {
        Ok(manager.is_running().await)
    } else {
        Ok(false)
    }
}
