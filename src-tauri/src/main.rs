#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chamber::commands::config::AppState;
use chamber::commands::sidecar::SidecarState;
use chamber::commands::auth::AuthState;
use chamber::logging::{init_default_logging, setup_logging, LoggingConfig};
use chamber::services::{SidecarManager, CredentialManager};
use chamber::utils::get_default_workspace_path;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;

fn main() {
    // Initialize logging
    init_default_logging();
    tracing::info!("Starting Chamber application");

    // Initialize workspace path
    let workspace_path = get_default_workspace_path();
    tracing::info!("Workspace path: {:?}", workspace_path);

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

    // Initialize auth state
    let auth_state = AuthState {
        credential_manager: Arc::new(TokioMutex::new(Some(CredentialManager::new()))),
        pending_oauth_flows: Arc::new(TokioMutex::new(std::collections::HashMap::new())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .manage(sidecar_state)
        .manage(auth_state)
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
            // Auth commands
            chamber::commands::auth::start_oauth_flow,
            chamber::commands::auth::handle_oauth_callback,
            chamber::commands::auth::save_credential,
            chamber::commands::auth::get_credential,
            chamber::commands::auth::delete_credential,
            chamber::commands::auth::list_credentials,
            chamber::commands::auth::has_credential,
            chamber::commands::auth::refresh_credential,
        ])
        .setup(|app| {
            tracing::debug!("Setting up application");

            // Initialize sidecar manager on startup
            let config = chamber::models::config::SidecarConfig {
                host: "127.0.0.1".to_string(),
                port: 8765,
                health_check_interval_seconds: 30,
                max_restart_attempts: 3,
            };

            let auth_state_for_sidecar = app.state::<AuthState>();
            let credential_manager_for_sidecar = auth_state_for_sidecar.credential_manager.clone();
            let sidecar_manager = SidecarManager::with_credentials(config, credential_manager_for_sidecar);
            let sidecar_state = app.state::<SidecarState>();

            tauri::async_runtime::block_on(async {
                let mut manager = sidecar_state.manager.lock().await;
                *manager = Some(sidecar_manager);
                tracing::info!("Sidecar manager initialized");
            });

            // Auto-start the sidecar in a separate async block
            let sidecar_manager_arc = sidecar_state.manager.clone();
            tauri::async_runtime::spawn(async move {
                // Small delay to ensure app is fully initialized
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let manager_lock = sidecar_manager_arc.lock().await;
                if let Some(manager) = manager_lock.as_ref() {
                    tracing::info!("Attempting to start sidecar...");
                    match manager.start("chamber-sidecar").await {
                        Ok(_) => {
                            tracing::info!("Sidecar started successfully");
                        }
                        Err(e) => {
                            tracing::warn!("Failed to start sidecar: {}. You may need to start it manually.", e);
                        }
                    }
                }
            });

            // Start background OAuth token refresh task
            let auth_state_for_refresh = app.state::<AuthState>();
            tauri::async_runtime::spawn(async move {
                // Check every 5 minutes for tokens that need refresh
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;

                    let manager_lock = auth_state_for_refresh.credential_manager.lock().await;
                    if let Some(manager) = manager_lock.as_ref() {
                        // Check anthropic
                        if let Err(e) = manager.refresh_token_if_needed("anthropic").await {
                            tracing::warn!("Failed to refresh anthropic token: {}", e);
                        }

                        // Check gemini
                        if let Err(e) = manager.refresh_token_if_needed("gemini").await {
                            tracing::warn!("Failed to refresh gemini token: {}", e);
                        }

                        // Check xai
                        if let Err(e) = manager.refresh_token_if_needed("xai").await {
                            tracing::warn!("Failed to refresh xai token: {}", e);
                        }
                    }
                }
            });

            // Initialize workspace
            let workspace_path = get_default_workspace_path();
            let workspace_manager = chamber::services::WorkspaceManager::new(&workspace_path);
            if let Err(e) = workspace_manager.init_workspace() {
                tracing::error!("Failed to initialize workspace: {}", e);
            } else {
                tracing::info!("Workspace initialized successfully");
            }

            // Load and apply logging configuration
            if let Ok(config) = load_config_from_workspace(std::path::Path::new(&workspace_path)) {
                if let Some(logging_config) = config.get("logging") {
                    if let Ok(log_config) = serde_yaml::from_value::<LoggingConfig>(logging_config.clone()) {
                        tracing::info!("Applying logging configuration from workspace");
                        if let Err(e) = setup_logging(&log_config) {
                            tracing::warn!("Failed to apply logging config: {}", e);
                        }
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_config_from_workspace(workspace_path: &std::path::Path) -> Result<serde_yaml::Mapping, Box<dyn std::error::Error>> {
    let config_path = workspace_path.join("config").join("chamber-config.yaml");
    let contents = std::fs::read_to_string(config_path)?;
    let config: serde_yaml::Value = serde_yaml::from_str(&contents)?;
    Ok(config.as_mapping().cloned().unwrap_or_default())
}
