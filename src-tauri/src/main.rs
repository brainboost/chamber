#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chamber::commands::config::{get_config, load_config, save_config, AppState};
use chamber::commands::session::{create_session, pause_session, resume_session, send_message};
use chamber::commands::sidecar::{
    get_websocket_url, health_check_sidecar, is_sidecar_running, restart_sidecar, start_sidecar,
    stop_sidecar, SidecarState,
};
use chamber::commands::workspace::{
    append_to_history, init_workspace, list_sessions, load_session_history, save_plan,
};
use chamber::services::SidecarManager;
use chamber::utils::get_default_workspace_path;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;

fn main() {
    // Initialize workspace path
    let workspace_path = get_default_workspace_path();

    // Initialize app state
    let app_state = AppState {
        config: Mutex::new(None),
        workspace_path: Mutex::new(workspace_path.clone()),
    };

    // Initialize sidecar state (sidecar path will be resolved at runtime)
    let sidecar_state = SidecarState {
        manager: Arc::new(TokioMutex::new(None)),
        sidecar_path: "chamber-sidecar".to_string(), // Will be resolved from bundle
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .manage(sidecar_state)
        .invoke_handler(tauri::generate_handler![
            // Config commands
            chamber::commands::config::load_config,
            chamber::commands::config::save_config,
            chamber::commands::config::get_config,
            // Workspace commands
            chamber::commands::workspace::init_workspace,
            chamber::commands::workspace::list_sessions,
            chamber::commands::workspace::load_session_history,
            chamber::commands::workspace::save_plan,
            chamber::commands::workspace::append_to_history,
            // Sidecar commands
            chamber::commands::sidecar::start_sidecar,
            chamber::commands::sidecar::stop_sidecar,
            chamber::commands::sidecar::restart_sidecar,
            chamber::commands::sidecar::health_check_sidecar,
            chamber::commands::sidecar::get_websocket_url,
            chamber::commands::sidecar::is_sidecar_running,
            // Session commands
            chamber::commands::session::create_session,
            chamber::commands::session::send_message,
            chamber::commands::session::pause_session,
            chamber::commands::session::resume_session,
        ])
        .setup(|app| {
            // Initialize sidecar manager on startup
            let config = chamber::models::config::SidecarConfig {
                host: "127.0.0.1".to_string(),
                port: 8765,
                health_check_interval_seconds: 30,
                max_restart_attempts: 3,
            };

            let sidecar_manager = SidecarManager::new(config);
            let sidecar_state = app.state::<SidecarState>();

            tauri::async_runtime::block_on(async {
                let mut manager = sidecar_state.manager.lock().await;
                *manager = Some(sidecar_manager);
            });

            // Initialize workspace
            let workspace_path = get_default_workspace_path();
            let workspace_manager = chamber::services::WorkspaceManager::new(&workspace_path);
            if let Err(e) = workspace_manager.init_workspace() {
                eprintln!("Failed to initialize workspace: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
