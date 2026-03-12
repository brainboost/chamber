use crate::models::auth::Credential;
use crate::services::CredentialManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Shared state for OAuth flows
#[derive(Default)]
pub struct AuthState {
    pub credential_manager: Arc<Mutex<Option<CredentialManager>>>,
    pub pending_oauth_flows: Arc<Mutex<std::collections::HashMap<String, OAuthFlow>>>,
}

/// Represents a pending OAuth flow
pub struct OAuthFlow {
    pub provider: String,
    pub code_verifier: String,
}

/// Start OAuth flow for a provider
///
/// Returns the authorization URL that the user should navigate to
#[tauri::command]
pub async fn start_oauth_flow(
    provider: String,
    state: tauri::State<'_, AuthState>,
) -> Result<String, String> {
    tracing::info!("Starting OAuth flow for provider: {}", provider);

    let manager = state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    // Generate PKCE verifier and challenge
    let (code_verifier, code_challenge) = CredentialManager::generate_pkce_verifier()
        .map_err(|e| format!("Failed to generate PKCE verifier: {}", e))?;

    // Generate state parameter for security
    let state_param = Uuid::new_v4().to_string();

    // Get authorization URL
    let auth_url = CredentialManager::get_authorization_url(&provider, &code_challenge, &state_param)
        .map_err(|e| format!("Failed to get authorization URL: {}", e))?;

    // Store the flow data
    let mut flows = state.pending_oauth_flows.lock().await;
    flows.insert(
        state_param.clone(),
        OAuthFlow {
            provider: provider.clone(),
            code_verifier,
        },
    );

    tracing::info!("OAuth authorization URL generated for provider: {}", provider);
    Ok(auth_url)
}

/// Handle OAuth callback
///
/// Called when the OAuth provider redirects back to the app
#[tauri::command]
pub async fn handle_oauth_callback(
    code: String,
    state_param: String,
    auth_state: tauri::State<'_, AuthState>,
) -> Result<Credential, String> {
    tracing::info!("Handling OAuth callback for state: {}", state_param);

    // Retrieve the flow data
    let mut flows = auth_state.pending_oauth_flows.lock().await;
    let flow = flows
        .remove(&state_param)
        .ok_or_else(|| "Invalid or expired OAuth flow".to_string())?;

    // Get credential manager
    let manager = auth_state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    // Exchange code for tokens
    let credential = manager
        .exchange_code_for_token(&flow.provider, &code, &flow.code_verifier)
        .await
        .map_err(|e| format!("Failed to exchange code for token: {}", e))?;

    // Store the credential
    manager
        .store_credential(credential.clone())
        .await
        .map_err(|e| format!("Failed to store credential: {}", e))?;

    tracing::info!("OAuth flow completed successfully for provider: {}", flow.provider);
    Ok(credential)
}

/// Save a credential (for API keys or manual token input)
#[tauri::command]
pub async fn save_credential(
    credential: Credential,
    state: tauri::State<'_, AuthState>,
) -> Result<(), String> {
    tracing::info!("Saving credential for provider: {}", credential.provider);

    let manager = state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    manager
        .store_credential(credential)
        .await
        .map_err(|e| format!("Failed to save credential: {}", e))?;

    tracing::info!("Credential saved successfully");
    Ok(())
}

/// Get a credential for a provider
#[tauri::command]
pub async fn get_credential(
    provider: String,
    state: tauri::State<'_, AuthState>,
) -> Result<Option<Credential>, String> {
    tracing::debug!("Getting credential for provider: {}", provider);

    let manager = state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let credential = manager
        .get_credential(&provider)
        .await
        .map_err(|e| format!("Failed to get credential: {}", e))?;

    Ok(credential)
}

/// Delete a credential for a provider
#[tauri::command]
pub async fn delete_credential(
    provider: String,
    auth_state: tauri::State<'_, AuthState>,
) -> Result<(), String> {
    tracing::info!("Deleting credential for provider: {}", provider);

    let manager = auth_state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    manager
        .delete_credential(&provider)
        .await
        .map_err(|e| format!("Failed to delete credential: {}", e))?;

    tracing::info!("Credential deleted successfully");
    Ok(())
}

/// List all providers with stored credentials
#[tauri::command]
pub async fn list_credentials(
    state: tauri::State<'_, AuthState>,
) -> Result<Vec<String>, String> {
    let manager = state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let providers = manager
        .list_providers()
        .await
        .map_err(|e| format!("Failed to list credentials: {}", e))?;

    Ok(providers)
}

/// Check if a provider has a stored credential
#[tauri::command]
pub async fn has_credential(
    provider: String,
    state: tauri::State<'_, AuthState>,
) -> Result<bool, String> {
    let manager = state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let credential = manager
        .get_credential(&provider)
        .await
        .map_err(|e| format!("Failed to check credential: {}", e))?;

    Ok(credential.is_some())
}

/// Refresh OAuth token if needed
#[tauri::command]
pub async fn refresh_credential(
    provider: String,
    state: tauri::State<'_, AuthState>,
) -> Result<Option<Credential>, String> {
    tracing::info!("Refreshing credential for provider: {}", provider);

    let manager = state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let refreshed = manager
        .refresh_token_if_needed(&provider)
        .await
        .map_err(|e| format!("Failed to refresh credential: {}", e))?;

    Ok(refreshed)
}
