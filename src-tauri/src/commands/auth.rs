use crate::models::auth::Credential;
use crate::services::CredentialManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use tauri::{AppHandle, Emitter};

/// Open the OAuth authorization URL in the user's default browser
#[tauri::command]
pub async fn open_oauth_url(url: String) -> Result<(), String> {
    tracing::info!("Opening OAuth URL in browser: {}", url);

    // Open the URL in the default browser
    open::that(url.clone())
        .map_err(|e| format!("Failed to open OAuth URL in browser: {}", e))?;

    tracing::info!("Successfully opened OAuth URL in browser");
    Ok(())
}

/// Start a local OAuth callback server
///
/// This starts a local HTTP server on a random port to receive the OAuth callback.
/// Returns the port number so the redirect URI can be updated dynamically.
#[tauri::command]
pub async fn start_oauth_callback_server(
    app: AppHandle,
    state: tauri::State<'_, AuthState>,
) -> Result<u16, String> {
    use tokio::net::TcpListener;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    tracing::info!("Starting OAuth callback server");

    // Bind to port 0 to get a random available port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| format!("Failed to bind OAuth callback server: {}", e))?;

    let port = listener.local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();

    tracing::info!("OAuth callback server listening on port {}", port);

    // Spawn a task to handle the callback
    let auth_state = state.credential_manager.clone();
    let pending_flows = state.pending_oauth_flows.clone();
    let app_handle = app.clone();

    tokio::spawn(async move {
        tracing::debug!("OAuth callback server task started");

        // Accept the connection (OAuth provider will redirect here)
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                tracing::info!("Received OAuth callback from: {}", addr);

                // Read the HTTP request
                let mut buffer = [0; 4096];
                let bytes_read = match socket.read(&mut buffer).await {
                    Ok(n) => n,
                    Err(e) => {
                        tracing::error!("Failed to read from socket: {}", e);
                        return;
                    }
                };

                let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                tracing::debug!("Received request:\n{}", request);

                // Extract the code and state from the HTTP request
                let (code, state_param) = if let Some(line) = request.lines().next() {
                    if line.starts_with("GET /") {
                        let path = line.split_whitespace().nth(1).unwrap_or("/");
                        if let Some(query) = path.split('?').nth(1) {
                            let params: std::collections::HashMap<String, String> = query
                                .split('&')
                                .filter_map(|p| {
                                    let mut parts = p.split('=');
                                    Some((
                                        parts.next()?.to_string(),
                                        parts.next()?.to_string(),
                                    ))
                                })
                                .collect();

                            let code = params.get("code").cloned();
                            let state = params.get("state").cloned();

                            (code, state)
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                // Send HTTP response
                let response = "HTTP/1.1 200 OK\r\n\
                    Content-Type: text/html\r\n\
                    Content-Length: 200\r\n\
                    Connection: close\r\n\
                    \r\n\
                    <!DOCTYPE html><html><head><title>Authentication Successful</title></head>\
                    <body><h1>Authentication Successful!</h1>\
                    <p>You can close this window and return to the application.</p>\
                    <script>window.close();</script></body></html>";

                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;

                // Process the OAuth callback if we got code and state
                if let (Some(code), Some(state_param)) = (code, state_param) {
                    tracing::info!("Processing OAuth callback with state: {}", state_param);

                    // Retrieve the flow data
                    let mut flows = pending_flows.lock().await;
                    if let Some(flow) = flows.remove(&state_param) {
                        drop(flows);

                        // Exchange code for tokens using the same redirect URI
                        match CredentialManager::exchange_code_for_token_with_redirect(
                            &flow.provider,
                            &code,
                            &flow.code_verifier,
                            Some(&flow.redirect_uri),
                        ).await {
                            Ok(credential) => {
                                // Get credential manager and store the credential
                                let mgr = auth_state.lock().await;
                                if let Some(manager) = mgr.as_ref() {
                                    if let Err(e) = manager.store_credential(credential.clone()).await {
                                        tracing::error!("Failed to store credential: {}", e);
                                        let _ = app_handle.emit_to("main", "oauth-error", e.to_string());
                                    } else {
                                        tracing::info!("OAuth flow completed successfully for provider: {}", flow.provider);
                                        let _ = app_handle.emit_to("main", "oauth-success", flow.provider);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to exchange code for token: {}", e);
                                let _ = app_handle.emit_to("main", "oauth-error", e.to_string());
                            }
                        }
                    } else {
                        tracing::error!("Invalid or expired OAuth flow: {}", state_param);
                        let _ = app_handle.emit_to("main", "oauth-error", "Invalid or expired OAuth flow".to_string());
                    }
                } else {
                    tracing::error!("OAuth callback missing code or state parameter");
                    let _ = app_handle.emit_to("main", "oauth-error", "Invalid OAuth callback".to_string());
                }
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {}", e);
            }
        }
    });

    Ok(port)
}

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
    pub redirect_uri: String,
}

/// Start OAuth flow for a provider
///
/// Returns the authorization URL that the user should navigate to
#[tauri::command]
pub async fn start_oauth_flow(
    provider: String,
    app: AppHandle,
    state: tauri::State<'_, AuthState>,
) -> Result<(String, u16), String> {
    tracing::info!("Starting OAuth flow for provider: {}", provider);

    let _manager = state
        .credential_manager
        .lock()
        .await;
    let _manager = _manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    // Start the OAuth callback server and get the port
    let port = start_oauth_callback_server(app.clone(), state.clone()).await?;

    // Generate PKCE verifier and challenge
    let (code_verifier, code_challenge) = CredentialManager::generate_pkce_verifier()
        .map_err(|e| format!("Failed to generate PKCE verifier: {}", e))?;

    // Generate state parameter for security
    let state_param = Uuid::new_v4().to_string();

    // Use localhost redirect URI for the browser flow
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    // Get authorization URL with the custom redirect URI
    let auth_url = CredentialManager::get_authorization_url_with_redirect(
        &provider,
        &code_challenge,
        &state_param,
        Some(&redirect_uri),
    )
        .map_err(|e| format!("Failed to get authorization URL: {}", e))?;

    // Store the flow data with the redirect URI
    let mut flows = state.pending_oauth_flows.lock().await;
    flows.insert(
        state_param.clone(),
        OAuthFlow {
            provider: provider.clone(),
            code_verifier,
            redirect_uri: redirect_uri.clone(),
        },
    );

    tracing::info!("OAuth authorization URL generated for provider: {} with redirect URI: {}",
        provider, redirect_uri);
    Ok((auth_url, port))
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
    let credential = CredentialManager::exchange_code_for_token(&flow.provider, &code, &flow.code_verifier)
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

/// Import Anthropic credentials from Claude Code CLI's credentials file
///
/// Reads ~/.claude/.credentials.json and imports the OAuth token into Chamber's keychain.
/// Returns true if credentials were found and imported, false if the file doesn't exist.
#[tauri::command]
pub async fn import_claude_code_credential(
    state: tauri::State<'_, AuthState>,
) -> Result<bool, String> {
    let credentials_path = dirs::home_dir()
        .map(|h| h.join(".claude").join(".credentials.json"))
        .ok_or("Could not determine home directory")?;

    if !credentials_path.exists() {
        tracing::debug!("Claude Code credentials not found at {:?}", credentials_path);
        return Ok(false);
    }

    let content = std::fs::read_to_string(&credentials_path)
        .map_err(|e| format!("Failed to read Claude credentials: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse Claude credentials: {}", e))?;

    let oauth = json
        .get("claudeAiOauth")
        .ok_or("No Claude AI OAuth credentials found in credentials file")?;

    let access_token = oauth
        .get("accessToken")
        .and_then(|v| v.as_str())
        .ok_or("Missing accessToken in Claude credentials")?
        .to_string();

    let refresh_token = oauth
        .get("refreshToken")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // expiresAt is in milliseconds in Claude Code's format
    let expires_at = oauth
        .get("expiresAt")
        .and_then(|v| v.as_i64())
        .map(|ms| ms / 1000);

    let scopes: Vec<String> = oauth
        .get("scopes")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let credential = Credential::oauth_token(
        "anthropic".to_string(),
        access_token,
        refresh_token,
        expires_at,
        scopes,
    );

    let manager = state.credential_manager.lock().await;
    let manager = manager
        .as_ref()
        .ok_or("Credential manager not initialized")?;

    manager
        .store_credential(credential)
        .await
        .map_err(|e| format!("Failed to store imported credential: {}", e))?;

    tracing::info!("Successfully imported Anthropic credential from Claude Code CLI");
    Ok(true)
}

/// Check if Claude Code CLI credentials are available to import
#[tauri::command]
pub async fn check_claude_code_credential() -> Result<bool, String> {
    let path = dirs::home_dir()
        .map(|h| h.join(".claude").join(".credentials.json"))
        .ok_or("Could not determine home directory")?;

    if !path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read credentials file: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|_| "Failed to parse credentials file")?;

    Ok(json.get("claudeAiOauth")
        .and_then(|o| o.get("accessToken"))
        .and_then(|v| v.as_str())
        .map(|t| !t.is_empty())
        .unwrap_or(false))
}

/// Push current credentials to the running sidecar process
///
/// Called after saving or importing credentials so the sidecar picks them up
/// without needing a restart.
#[tauri::command]
pub async fn push_credentials_to_sidecar(
    state: tauri::State<'_, AuthState>,
) -> Result<bool, String> {
    let manager = state.credential_manager.lock().await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let env_vars = manager
        .get_credentials_as_env()
        .await
        .map_err(|e| format!("Failed to read credentials: {}", e))?;

    if env_vars.is_empty() {
        return Ok(false);
    }

    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8765/api/credentials")
        .json(&serde_json::json!({ "env_vars": env_vars }))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!("Pushed credentials to sidecar");
            Ok(true)
        }
        Ok(resp) => {
            tracing::warn!("Sidecar credential push failed: {}", resp.status());
            Ok(false)
        }
        Err(e) => {
            // Sidecar not running — not an error, just a no-op
            tracing::debug!("Sidecar not reachable for credential push: {}", e);
            Ok(false)
        }
    }
}

/// Open an in-app OAuth webview for Anthropic (PKCE flow, intercepts redirect)
///
/// Instead of redirecting to localhost (which the client_id doesn't support),
/// we open a Tauri WebviewWindow and intercept navigation to the allowed
/// redirect URI `https://claude.ai/oauth/code/callback`.
#[tauri::command]
pub async fn open_anthropic_oauth_webview(
    app: AppHandle,
    state: tauri::State<'_, AuthState>,
) -> Result<(), String> {
    use std::sync::{Arc, Mutex as StdMutex};
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
    use tokio::sync::oneshot;

    let (code_verifier, code_challenge) = CredentialManager::generate_pkce_verifier()
        .map_err(|e| format!("Failed to generate PKCE verifier: {}", e))?;

    let state_param = Uuid::new_v4().to_string();

    // Use the redirect URI that is actually registered with Anthropic's OAuth client
    let redirect_uri = "https://claude.ai/oauth/code/callback".to_string();

    let auth_url_str = CredentialManager::get_authorization_url_with_redirect(
        "anthropic",
        &code_challenge,
        &state_param,
        Some(&redirect_uri),
    )
    .map_err(|e| format!("Failed to build auth URL: {}", e))?;

    let auth_url: url::Url = auth_url_str
        .parse()
        .map_err(|e| format!("Invalid auth URL: {}", e))?;

    // Stash the flow so the spawned task can retrieve it
    {
        let mut flows = state.pending_oauth_flows.lock().await;
        flows.insert(
            state_param.clone(),
            OAuthFlow {
                provider: "anthropic".to_string(),
                code_verifier,
                redirect_uri: redirect_uri.clone(),
            },
        );
    }

    // Oneshot channel: navigation callback → async task
    let (tx, rx) = oneshot::channel::<Result<String, String>>();
    let tx = Arc::new(StdMutex::new(Some(tx)));

    let tx_nav = tx.clone();
    let app_for_nav = app.clone();

    // Close any existing OAuth window before opening a new one
    if let Some(existing) = app.get_webview_window("anthropic-oauth") {
        let _ = existing.close();
        // Small delay to let it close cleanly
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let window = WebviewWindowBuilder::new(
        &app,
        "anthropic-oauth",
        WebviewUrl::External(auth_url),
    )
    .title("Sign in with Anthropic")
    .inner_size(480.0, 720.0)
    .center()
    .on_navigation(move |url| {
        // Google OAuth blocks all embedded WebViews at the browser engine level.
        // When the user clicks "Authorize with Google", redirect the window to a
        // static in-app page explaining they must use email/password instead.
        if url.host_str().map_or(false, |h| h.contains("accounts.google") || h == "google.com") {
            if let Some(win) = app_for_nav.get_webview_window("anthropic-oauth") {
                let _ = win.navigate(
                    "data:text/html,<html><body style='font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;background:#f8fafc'><div style='max-width:360px;text-align:center;padding:32px'><div style='font-size:48px;margin-bottom:16px'>🔒</div><h2 style='margin:0 0 12px;color:#1e293b'>Google sign-in unavailable</h2><p style='color:#64748b;line-height:1.6;margin:0 0 24px'>Google blocks sign-in inside embedded browsers.<br><br>Please go back and sign in with your <strong>email and password</strong> instead.</p><button onclick='history.back()' style='background:#2563eb;color:#fff;border:none;padding:10px 24px;border-radius:8px;cursor:pointer;font-size:15px'>← Go back</button></div></body></html>".parse().expect("valid data URL")
                );
            }
            return false;
        }

        // Intercept the OAuth redirect and extract the code
        if url.as_str().starts_with("https://claude.ai/oauth/code/callback") {
            let result = url
                .query_pairs()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| Ok(v.into_owned()))
                .unwrap_or_else(|| {
                    let err = url
                        .query_pairs()
                        .find(|(k, _)| k == "error")
                        .map(|(_, v)| format!("OAuth error: {}", v))
                        .unwrap_or_else(|| "OAuth callback missing code".to_string());
                    Err(err)
                });

            if let Ok(mut guard) = tx_nav.lock() {
                if let Some(sender) = guard.take() {
                    let _ = sender.send(result);
                }
            }
            // Block the webview from actually navigating to the callback URL
            return false;
        }
        true
    })
    .build()
    .map_err(|e| format!("Failed to open OAuth window: {}", e))?;

    // Async task: wait for code → close window → exchange for tokens → emit event
    let app_clone = app.clone();
    let credential_manager = state.credential_manager.clone();
    let pending_flows = state.pending_oauth_flows.clone();
    let state_param_clone = state_param.clone();

    tokio::spawn(async move {
        let result = tokio::time::timeout(tokio::time::Duration::from_secs(300), rx).await;

        // Always close the OAuth window
        let _ = window.close();

        let code = match result {
            Ok(Ok(Ok(code))) => code,
            Ok(Ok(Err(err))) => {
                let _ = app_clone.emit_to("main", "oauth-error", err);
                return;
            }
            _ => {
                // Timeout or channel closed (user closed window)
                let mut flows = pending_flows.lock().await;
                flows.remove(&state_param_clone);
                // No error event — user deliberately cancelled
                return;
            }
        };

        let flow = {
            let mut flows = pending_flows.lock().await;
            flows.remove(&state_param_clone)
        };

        let Some(flow) = flow else {
            let _ = app_clone.emit_to("main", "oauth-error", "OAuth flow expired".to_string());
            return;
        };

        match CredentialManager::exchange_code_for_token_with_redirect(
            &flow.provider,
            &code,
            &flow.code_verifier,
            Some(&flow.redirect_uri),
        )
        .await
        {
            Ok(credential) => {
                let mgr = credential_manager.lock().await;
                if let Some(manager) = mgr.as_ref() {
                    match manager.store_credential(credential).await {
                        Ok(_) => {
                            tracing::info!("OAuth webview flow completed for anthropic");
                            let _ = app_clone.emit_to("main", "oauth-success", "anthropic".to_string());
                        }
                        Err(e) => {
                            let _ = app_clone.emit_to("main", "oauth-error", e.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Token exchange failed: {}", e);
                let _ = app_clone.emit_to("main", "oauth-error", e.to_string());
            }
        }
    });

    Ok(())
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
