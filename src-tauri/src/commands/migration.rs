use crate::commands::auth::AuthState;
use crate::models::auth::Credential;
use std::collections::HashMap;
use std::path::PathBuf;

/// Migrate API keys from .env file to keychain storage
///
/// This function reads a .env file and migrates found API keys to secure keychain storage.
#[tauri::command]
pub async fn migrate_from_env_file(
    env_path: Option<String>,
    state: tauri::State<'_, AuthState>,
) -> Result<Vec<String>, String> {
    tracing::info!("Starting credential migration from .env file");

    // Determine .env file path
    let env_file_path = if let Some(path) = env_path {
        PathBuf::from(path)
    } else {
        // Default to python-sidecar/.env
        let mut path = PathBuf::from(".");
        path.push("python-sidecar");
        path.push(".env");
        path
    };

    tracing::debug!("Reading .env file from: {:?}", env_file_path);

    // Read .env file
    let env_content = std::fs::read_to_string(&env_file_path)
        .map_err(|e| format!("Failed to read .env file: {}", e))?;

    // Parse .env file
    let env_vars = parse_env_file(&env_content);

    // Get credential manager
    let manager = state
        .credential_manager
        .lock()
        .await;
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Credential manager not initialized".to_string())?;

    let mut migrated_providers = Vec::new();

    // Migrate ANTHROPIC_API_KEY
    if let Some(anthropic_key) = env_vars.get("ANTHROPIC_API_KEY") {
        if !anthropic_key.is_empty() {
            tracing::info!("Found ANTHROPIC_API_KEY, migrating to keychain");

            let credential = Credential::api_key("anthropic".to_string(), anthropic_key.clone());

            // Check if already exists
            if manager.get_credential("anthropic").await.is_ok() {
                tracing::info!("Credential for anthropic already exists in keychain, skipping");
            } else {
                manager
                    .store_credential(credential)
                    .await
                    .map_err(|e| format!("Failed to store anthropic credential: {}", e))?;

                migrated_providers.push("anthropic".to_string());
                tracing::info!("Successfully migrated anthropic credential");
            }
        }
    }

    // Migrate GOOGLE_API_KEY
    if let Some(google_key) = env_vars.get("GOOGLE_API_KEY") {
        if !google_key.is_empty() {
            tracing::info!("Found GOOGLE_API_KEY, migrating to keychain");

            let credential = Credential::api_key("gemini".to_string(), google_key.clone());

            // Check if already exists
            if manager.get_credential("gemini").await.is_ok() {
                tracing::info!("Credential for gemini already exists in keychain, skipping");
            } else {
                manager
                    .store_credential(credential)
                    .await
                    .map_err(|e| format!("Failed to store gemini credential: {}", e))?;

                migrated_providers.push("gemini".to_string());
                tracing::info!("Successfully migrated gemini credential");
            }
        }
    }

    // Migrate XAI_API_KEY
    if let Some(xai_key) = env_vars.get("XAI_API_KEY") {
        if !xai_key.is_empty() {
            tracing::info!("Found XAI_API_KEY, migrating to keychain");

            let credential = Credential::api_key("xai".to_string(), xai_key.clone());

            // Check if already exists
            if manager.get_credential("xai").await.is_ok() {
                tracing::info!("Credential for xai already exists in keychain, skipping");
            } else {
                manager
                    .store_credential(credential)
                    .await
                    .map_err(|e| format!("Failed to store xai credential: {}", e))?;

                migrated_providers.push("xai".to_string());
                tracing::info!("Successfully migrated xai credential");
            }
        }
    }

    if migrated_providers.is_empty() {
        tracing::info!("No new credentials to migrate");
    } else {
        tracing::info!("Migration complete: {} providers migrated", migrated_providers.len());
    }

    Ok(migrated_providers)
}

/// Check if .env file exists and has credentials to migrate
#[tauri::command]
pub async fn check_env_file_for_migration(env_path: Option<String>) -> Result<bool, String> {
    let env_file_path = if let Some(path) = env_path {
        PathBuf::from(path)
    } else {
        let mut path = PathBuf::from(".");
        path.push("python-sidecar");
        path.push(".env");
        path
    };

    // Check if file exists
    if !env_file_path.exists() {
        return Ok(false);
    }

    // Read and check for API keys
    let env_content = std::fs::read_to_string(&env_file_path)
        .map_err(|e| format!("Failed to read .env file: {}", e))?;

    let env_vars = parse_env_file(&env_content);

    // Check if any API keys are present
    let has_credentials = env_vars.keys().any(|key| {
        matches!(
            key.as_str(),
            "ANTHROPIC_API_KEY" | "GOOGLE_API_KEY" | "XAI_API_KEY"
        )
    });

    Ok(has_credentials)
}

/// Parse .env file content into a HashMap
fn parse_env_file(content: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse KEY=VALUE format
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();

            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value[1..value.len() - 1].to_string()
            } else {
                value
            };

            vars.insert(key, value);
        }
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_file() {
        let content = r#"
# This is a comment
ANTHROPIC_API_KEY=sk-ant-test123
GOOGLE_API_KEY="AIza-test456"
EMPTY_KEY=
XAI_API_KEY='xai-test789'
"#;

        let vars = parse_env_file(content);

        assert_eq!(vars.get("ANTHROPIC_API_KEY"), Some(&"sk-ant-test123".to_string()));
        assert_eq!(vars.get("GOOGLE_API_KEY"), Some(&"AIza-test456".to_string()));
        assert_eq!(vars.get("EMPTY_KEY"), Some(&"".to_string()));
        assert_eq!(vars.get("XAI_API_KEY"), Some(&"xai-test789".to_string()));
    }
}
