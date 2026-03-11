use crate::commands::config::AppState;
use crate::services::WorkspaceManager;
use tauri::State;

#[tauri::command]
pub async fn init_workspace(state: State<'_, AppState>) -> Result<(), String> {
    let workspace_path = state.workspace_path.lock().unwrap().clone();
    let manager = WorkspaceManager::new(&workspace_path);

    manager.init_workspace().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let workspace_path = state.workspace_path.lock().unwrap().clone();
    let manager = WorkspaceManager::new(&workspace_path);

    manager.list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_session_history(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let workspace_path = state.workspace_path.lock().unwrap().clone();
    let manager = WorkspaceManager::new(&workspace_path);

    manager
        .load_session_history(&session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_plan(
    session_id: String,
    plan: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let workspace_path = state.workspace_path.lock().unwrap().clone();
    let manager = WorkspaceManager::new(&workspace_path);

    manager
        .save_plan(&session_id, &plan)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn append_to_history(
    session_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let workspace_path = state.workspace_path.lock().unwrap().clone();
    let manager = WorkspaceManager::new(&workspace_path);

    manager
        .append_to_history(&session_id, &content)
        .map_err(|e| e.to_string())
}
