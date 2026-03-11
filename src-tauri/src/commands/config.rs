use crate::models::config::ChamberConfig;
use crate::services::ConfigManager;
use tauri::State;
use std::sync::Mutex;

pub struct AppState {
    pub config: Mutex<Option<ChamberConfig>>,
    pub workspace_path: Mutex<String>,
}

#[tauri::command]
pub async fn load_config(state: State<'_, AppState>) -> Result<ChamberConfig, String> {
    let workspace_path = state.workspace_path.lock().unwrap().clone();
    let config_manager = ConfigManager::new(&workspace_path);

    let config = config_manager
        .load_config()
        .map_err(|e| e.to_string())?;

    ConfigManager::validate_config(&config)
        .map_err(|e| e.to_string())?;

    *state.config.lock().unwrap() = Some(config.clone());

    Ok(config)
}

#[tauri::command]
pub async fn save_config(
    config: ChamberConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ConfigManager::validate_config(&config)
        .map_err(|e| e.to_string())?;

    let workspace_path = state.workspace_path.lock().unwrap().clone();
    let config_manager = ConfigManager::new(&workspace_path);

    config_manager
        .save_config(&config)
        .map_err(|e| e.to_string())?;

    *state.config.lock().unwrap() = Some(config);

    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<ChamberConfig, String> {
    let config = state.config.lock().unwrap();

    match config.as_ref() {
        Some(c) => Ok(c.clone()),
        None => Err("Config not loaded".to_string()),
    }
}
