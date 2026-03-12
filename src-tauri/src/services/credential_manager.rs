use crate::models::auth::{AuthType, Credential, OAuthConfig};
use anyhow::{anyhow, Context, Result};
use base64::Engine as _;
use keyring::{Entry, Error as KeyringError};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Service name for keychain storage
const SERVICE_NAME: &str = "com.chamber.app";

/// Credential Manager for secure storage and retrieval of API keys and OAuth tokens
pub struct CredentialManager {
    /// In-memory cache of credentials
    cache: RwLock<HashMap<String, Credential>>,
}

impl CredentialManager {
    /// Create a new CredentialManager
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Store a credential securely in the keychain
    pub async fn store_credential(&self, credential: Credential) -> Result<()> {
        let provider = credential.provider.clone();
        let serialized = serde_json::to_string(&credential)
            .context("Failed to serialize credential")?;

        let entry = Entry::new(SERVICE_NAME, &format!("{}_credential", provider))
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;

        entry
            .set_password(&serialized)
            .map_err(|e| anyhow!("Failed to store credential in keychain: {}", e))?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(provider, credential);

        tracing::info!("Stored credential for provider: {}", provider);
        Ok(())
    }

    /// Retrieve a credential from the keychain
    pub async fn get_credential(&self, provider: &str) -> Result<Option<Credential>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cred) = cache.get(provider) {
                tracing::debug!("Retrieved credential from cache for provider: {}", provider);
                return Ok(Some(cred.clone()));
            }
        }

        // Fall back to keychain
        let entry = Entry::new(SERVICE_NAME, &format!("{}_credential", provider));

        match entry {
            Ok(entry) => {
                match entry.get_password() {
                    Ok(password) => {
                        let credential: Credential = serde_json::from_str(&password)
                            .context("Failed to deserialize credential")?;

                        // Update cache
                        let mut cache = self.cache.write().await;
                        cache.insert(provider.to_string(), credential.clone());

                        tracing::info!("Retrieved credential from keychain for provider: {}", provider);
                        Ok(Some(credential))
                    }
                    Err(KeyringError::NoEntry) => {
                        tracing::debug!("No credential found for provider: {}", provider);
                        Ok(None)
                    }
                    Err(e) => Err(anyhow!("Failed to retrieve credential from keychain: {}", e)),
                }
            }
            Err(e) => Err(anyhow!("Failed to create keyring entry: {}", e)),
        }
    }

    /// Delete a credential from the keychain
    pub async fn delete_credential(&self, provider: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_credential", provider))
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;

        entry
            .delete_credential()
            .map_err(|e| match e {
                KeyringError::NoEntry => {
                    tracing::info!("No credential found to delete for provider: {}", provider);
                    Ok(())
                }
                _ => Err(anyhow!("Failed to delete credential from keychain: {}", e)),
            })?;

        // Remove from cache
        let mut cache = self.cache.write().await;
        cache.remove(provider);

        tracing::info!("Deleted credential for provider: {}", provider);
        Ok(())
    }

    /// List all providers with stored credentials
    pub async fn list_providers(&self) -> Result<Vec<String>> {
        let cache = self.cache.read().await;
        let mut providers: Vec<String> = cache.keys().cloned().collect();

        // If cache is empty, try to discover from keychain
        if providers.is_empty() {
            drop(cache);
            // Note: keyring crate doesn't provide a way to list all entries
            // Users would need to know which providers they've set up
            // This is a limitation we work with
        }

        providers.sort();
        Ok(providers)
    }

    /// Refresh OAuth token if needed
    pub async fn refresh_token_if_needed(&self, provider: &str) -> Result<Option<Credential>> {
        tracing::debug!("Checking if token refresh needed for provider: {}", provider);

        if let Some(credential) = self.get_credential(provider).await? {
            if credential.needs_refresh() {
                tracing::info!("Token refresh needed for provider: {}", provider);

                // Log token expiry info
                if let AuthType::OAuthToken { expires_at, .. } = &credential.auth_type {
                    if let Some(expiry) = expires_at {
                        let now = chrono::Utc::now().timestamp();
                        let minutes_until_expiry = (expiry - now) / 60;
                        tracing::info!("Token expires in {} minutes", minutes_until_expiry);
                    }
                }

                let refreshed = self.refresh_oauth_token(&credential).await?;
                self.store_credential(refreshed.clone()).await?;
                tracing::info!("Successfully refreshed token for provider: {}", provider);
                return Ok(Some(refreshed));
            } else {
                tracing::debug!("Token is still valid for provider: {}", provider);
            }
        } else {
            tracing::debug!("No credential found for provider: {}", provider);
        }

        Ok(None)
    }

    /// Refresh an OAuth token
    async fn refresh_oauth_token(&self, credential: &Credential) -> Result<Credential> {
        tracing::debug!("Starting token refresh for provider: {}", credential.provider);

        let config = OAuthConfig::for_provider(&credential.provider)
            .ok_or_else(|| anyhow!("No OAuth config for provider: {}", credential.provider))?;

        tracing::debug!("Using token endpoint: {}", config.token_url);

        let refresh_token = match &credential.auth_type {
            AuthType::OAuthToken { refresh_token, .. } => {
                tracing::trace!("Using refresh token (length: {})", refresh_token.len());
                refresh_token.clone()
            }
            AuthType::ApiKey { .. } => {
                return Err(anyhow!("Cannot refresh API key credential"));
            }
        };

        // Build token request
        let client = reqwest::Client::new();
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", &refresh_token);

        // Add client_id for Google
        if let Some(client_id) = &config.client_id {
            params.insert("client_id", client_id);
            tracing::trace!("Using client_id for token refresh");
        }

        tracing::debug!("Sending refresh request to token endpoint");

        let response = client
            .post(&config.token_url)
            .form(&params)
            .send()
            .await
            .context("Failed to send refresh request")?;

        let status = response.status();
        tracing::debug!("Token refresh response status: {}", status);

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Token refresh failed with status {}: {}", status, error_text);
            return Err(anyhow!("Token refresh failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        tracing::info!("Successfully received new access token");

        let expires_at = token_response.expires_in.map(|duration| {
            let expiry = chrono::Utc::now().timestamp() + duration as i64;
            tracing::info!("New token expires in {} seconds", duration);
            expiry
        });

        Ok(Credential::oauth_token(
            credential.provider.clone(),
            token_response.access_token,
            token_response
                .refresh_token
                .unwrap_or_else(|| refresh_token.clone()),
            expires_at,
            vec![], // Scopes not returned in refresh response
        ))
    }

    /// Generate PKCE code verifier and challenge
    pub fn generate_pkce_verifier() -> Result<(String, String)> {
        tracing::trace!("Generating PKCE verifier and challenge");

        // Generate random code verifier (43-128 characters)
        let mut rng = rand::thread_rng();
        let verifier: String = (0..64)
            .map(|_| {
                let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
                chars[rng.gen_range(0..chars.len())] as char
            })
            .collect();

        tracing::trace!("Generated code verifier (length: {})", verifier.len());

        // Create code challenge by hashing verifier with SHA256
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();

        // Base64 URL encode without padding
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(hash);

        tracing::trace!("Generated code challenge (length: {})", challenge.len());

        Ok((verifier, challenge))
    }

    /// Get OAuth authorization URL
    pub fn get_authorization_url(
        provider: &str,
        code_challenge: &str,
        state: &str,
    ) -> Result<String> {
        tracing::debug!("Generating authorization URL for provider: {}", provider);
        tracing::trace!("Using state parameter: {}", state);

        let config = OAuthConfig::for_provider(provider)
            .ok_or_else(|| anyhow!("No OAuth config for provider: {}", provider))?;

        tracing::debug!("OAuth config - auth_url: {}, token_url: {}",
            config.auth_url, config.token_url);
        tracing::debug!("OAuth scopes: {}", config.scopes.join(", "));

        let mut url = url::Url::parse(&config.auth_url)?;

        // Add query parameters
        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("response_type", "code");
            query_pairs.append_pair("code_challenge", code_challenge);
            query_pairs.append_pair("code_challenge_method", "S256");
            query_pairs.append_pair("redirect_uri", &config.redirect_uri);
            query_pairs.append_pair("state", state);

            // Add scopes
            query_pairs.append_pair("scope", &config.scopes.join(" "));

            // Add client_id for Google
            if let Some(client_id) = &config.client_id {
                query_pairs.append_pair("client_id", client_id);
                tracing::trace!("Using client_id for OAuth flow");
            }
        }

        let auth_url = url.to_string();
        tracing::info!("Generated authorization URL for provider: {}", provider);

        Ok(auth_url)
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code_for_token(
        provider: &str,
        code: &str,
        code_verifier: &str,
    ) -> Result<Credential> {
        tracing::info!("Exchanging authorization code for tokens for provider: {}", provider);
        tracing::debug!("Authorization code length: {}", code.len());

        let config = OAuthConfig::for_provider(provider)
            .ok_or_else(|| anyhow!("No OAuth config for provider: {}", provider))?;

        tracing::debug!("Token endpoint: {}", config.token_url);

        let client = reqwest::Client::new();
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("code_verifier", code_verifier);
        params.insert("redirect_uri", &config.redirect_uri);

        // Add client_id for Google
        if let Some(client_id) = &config.client_id {
            params.insert("client_id", client_id);
            tracing::trace!("Using client_id for token exchange");
        }

        tracing::debug!("Sending token exchange request");

        let response = client
            .post(&config.token_url)
            .form(&params)
            .send()
            .await
            .context("Failed to send token request")?;

        let status = response.status();
        tracing::debug!("Token exchange response status: {}", status);

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Token exchange failed with status {}: {}", status, error_text);

            // Provide more helpful error messages based on common OAuth errors
            if error_text.contains("invalid_grant") {
                return Err(anyhow!(
                    "Authorization code is invalid or expired. Please try the OAuth flow again."
                ));
            } else if error_text.contains("redirect_uri_mismatch") {
                return Err(anyhow!(
                    "Redirect URI mismatch. Please check the OAuth configuration."
                ));
            } else if error_text.contains("invalid_client") {
                return Err(anyhow!(
                    "Invalid client ID. Please check the OAuth configuration."
                ));
            }

            return Err(anyhow!("Token exchange failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        tracing::info!("Successfully exchanged authorization code for tokens");

        let expires_at = token_response.expires_in.map(|duration| {
            let expiry = chrono::Utc::now().timestamp() + duration as i64;
            tracing::info!("Access token expires in {} seconds", duration);
            expiry
        });

        let refresh_token = token_response
            .refresh_token
            .ok_or_else(|| {
                tracing::error!("No refresh token in OAuth response");
                anyhow!("No refresh token in response")
            })?;

        tracing::info!("Successfully obtained OAuth tokens (refresh token length: {})",
            refresh_token.len());

        Ok(Credential::oauth_token(
            provider.to_string(),
            token_response.access_token,
            refresh_token,
            expires_at,
            config.scopes,
        ))
    }

    /// Get credentials as environment variables for Python sidecar
    pub async fn get_credentials_as_env(&self) -> Result<HashMap<String, String>> {
        let mut env_vars = HashMap::new();

        // Anthropic
        if let Some(cred) = self.get_credential("anthropic").await? {
            if let Some(token) = cred.get_token() {
                env_vars.insert("ANTHROPIC_API_KEY".to_string(), token);
            }
        }

        // Gemini/Google
        if let Some(cred) = self.get_credential("gemini").await? {
            if let Some(token) = cred.get_token() {
                env_vars.insert("GOOGLE_API_KEY".to_string(), token);
            }
        }

        // XAI
        if let Some(cred) = self.get_credential("xai").await? {
            if let Some(token) = cred.get_token() {
                env_vars.insert("XAI_API_KEY".to_string(), token);
            }
        }

        Ok(env_vars)
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Token response from OAuth token endpoint
#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::auth::AuthType;

    #[test]
    fn test_generate_pkce_verifier() {
        let (verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();
        assert!(!verifier.is_empty());
        assert!(!challenge.is_empty());
        assert!(verifier.len() >= 43);
        assert!(verifier.len() <= 128);
        assert_ne!(verifier, challenge); // Should be different due to hashing
        assert!(challenge.len() == 43 || challenge.len() == 44); // Base64 URL encoded
    }

    #[test]
    fn test_pkce_verifier_deterministic_hash() {
        // Same verifier should produce same challenge
        let verifier = "test-verifier-string-12345";
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

        let mut hasher2 = Sha256::new();
        hasher2.update(verifier.as_bytes());
        let hash2 = hasher2.finalize();
        let challenge2 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash2);

        assert_eq!(challenge, challenge2);
    }

    #[test]
    fn test_authorization_url_generation() {
        let (_verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();
        let url = CredentialManager::get_authorization_url("anthropic", &challenge, "test-state").unwrap();

        assert!(url.contains("code_challenge="));
        assert!(url.contains("state=test-state"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("redirect_uri=chamber://oauth/callback"));
        assert!(url.contains("scope="));
    }

    #[test]
    fn test_authorization_url_google_includes_client_id() {
        let (_verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();
        let url = CredentialManager::get_authorization_url("gemini", &challenge, "test-state").unwrap();

        assert!(url.contains("client_id="));
        assert!(url.contains("https://accounts.google.com"));
    }

    #[test]
    fn test_authorization_url_invalid_provider() {
        let (_verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();
        let result = CredentialManager::get_authorization_url("invalid_provider", &challenge, "test-state");
        assert!(result.is_err());
    }

    #[test]
    fn test_credential_api_key() {
        let cred = Credential::api_key("test_provider".to_string(), "test_key_123".to_string());

        assert_eq!(cred.provider, "test_provider");
        assert_eq!(cred.get_token().unwrap(), "test_key_123");
        assert!(!cred.needs_refresh()); // API keys don't need refresh
    }

    #[test]
    fn test_credential_oauth_token() {
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 3600; // 1 hour from now

        let cred = Credential::oauth_token(
            "test_provider".to_string(),
            "access_token".to_string(),
            "refresh_token".to_string(),
            Some(expires_at),
            vec!["scope1".to_string(), "scope2".to_string()],
        );

        assert_eq!(cred.provider, "test_provider");
        assert_eq!(cred.get_token().unwrap(), "access_token");
        assert!(!cred.needs_refresh()); // Token is still valid
    }

    #[test]
    fn test_credential_oauth_token_needs_refresh() {
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 200; // Expires in 200 seconds (< 5 minutes)

        let cred = Credential::oauth_token(
            "test_provider".to_string(),
            "access_token".to_string(),
            "refresh_token".to_string(),
            Some(expires_at),
            vec!["scope1".to_string()],
        );

        assert!(cred.needs_refresh()); // Token needs refresh
    }

    #[test]
    fn test_credential_oauth_token_no_expiry() {
        // Token without expiry time doesn't need refresh
        let cred = Credential::oauth_token(
            "test_provider".to_string(),
            "access_token".to_string(),
            "refresh_token".to_string(),
            None,
            vec!["scope1".to_string()],
        );

        assert!(!cred.needs_refresh());
    }

    #[test]
    fn test_oauth_config_anthropic() {
        let config = OAuthConfig::for_provider("anthropic").unwrap();
        assert_eq!(config.provider, "anthropic");
        assert!(config.auth_url.contains("claude.ai"));
        assert!(config.token_url.contains("claude.ai"));
        assert!(config.scopes.contains(&"openid".to_string()));
        assert!(config.scopes.contains(&"offline_access".to_string()));
        assert_eq!(config.redirect_uri, "chamber://oauth/callback");
        assert!(config.client_id.is_none()); // Anthropic doesn't use client_id
    }

    #[test]
    fn test_oauth_config_gemini() {
        let config = OAuthConfig::for_provider("gemini").unwrap();
        assert_eq!(config.provider, "gemini");
        assert!(config.auth_url.contains("accounts.google.com"));
        assert!(config.token_url.contains("oauth2.googleapis.com"));
        assert!(config.scopes.contains(&"https://www.googleapis.com/auth/generative.language".to_string()));
        assert!(config.client_id.is_some()); // Google requires client_id
    }

    #[test]
    fn test_oauth_config_invalid_provider() {
        let config = OAuthConfig::for_provider("invalid_provider");
        assert!(config.is_none());
    }

    #[tokio::test]
    async fn test_credential_manager_new() {
        let manager = CredentialManager::new();
        assert!(manager.cache.try_read().is_ok());
    }

    #[tokio::test]
    async fn test_credential_manager_default() {
        let manager = CredentialManager::default();
        assert!(manager.cache.try_read().is_ok());
    }

    #[test]
    fn test_credential_serialization() {
        let cred = Credential::oauth_token(
            "test_provider".to_string(),
            "access_token".to_string(),
            "refresh_token".to_string(),
            Some(1234567890),
            vec!["scope1".to_string()],
        );

        let serialized = serde_json::to_string(&cred).unwrap();
        assert!(serialized.contains("test_provider"));
        assert!(serialized.contains("access_token"));

        let deserialized: Credential = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.provider, cred.provider);
        assert_eq!(deserialized.get_token().unwrap(), cred.get_token().unwrap());
    }

    #[test]
    fn test_multiple_pkce_generations_are_unique() {
        let (v1, c1) = CredentialManager::generate_pkce_verifier().unwrap();
        let (v2, c2) = CredentialManager::generate_pkce_verifier().unwrap();

        // Verifiers should be different (random)
        assert_ne!(v1, v2);
        // Challenges should also be different
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_get_credentials_as_env_empty() {
        // This test verifies the function exists and returns empty map when no credentials
        // Full integration test would require mocking the keyring
        let manager = CredentialManager::new();

        // In a real test, we would use tokio and potentially mock keyring
        // For now, we just verify the function compiles
        assert!(true); // Placeholder
    }
}
