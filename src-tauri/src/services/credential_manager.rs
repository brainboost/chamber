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
        if let Some(credential) = self.get_credential(provider).await? {
            if credential.needs_refresh() {
                tracing::info!("Refreshing OAuth token for provider: {}", provider);
                let refreshed = self.refresh_oauth_token(&credential).await?;
                self.store_credential(refreshed.clone()).await?;
                return Ok(Some(refreshed));
            }
        }
        Ok(None)
    }

    /// Refresh an OAuth token
    async fn refresh_oauth_token(&self, credential: &Credential) -> Result<Credential> {
        let config = OAuthConfig::for_provider(&credential.provider)
            .ok_or_else(|| anyhow!("No OAuth config for provider: {}", credential.provider))?;

        let refresh_token = match &credential.auth_type {
            AuthType::OAuthToken { refresh_token, .. } => refresh_token.clone(),
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
        }

        let response = client
            .post(&config.token_url)
            .form(&params)
            .send()
            .await
            .context("Failed to send refresh request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Token refresh failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        let expires_at = token_response.expires_in.map(|duration| {
            chrono::Utc::now().timestamp() + duration as i64
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
        // Generate random code verifier (43-128 characters)
        let mut rng = rand::thread_rng();
        let verifier: String = (0..64)
            .map(|_| {
                let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
                chars[rng.gen_range(0..chars.len())] as char
            })
            .collect();

        // Create code challenge by hashing verifier with SHA256
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();

        // Base64 URL encode without padding
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(hash);

        Ok((verifier, challenge))
    }

    /// Get OAuth authorization URL
    pub fn get_authorization_url(
        provider: &str,
        code_challenge: &str,
        state: &str,
    ) -> Result<String> {
        let config = OAuthConfig::for_provider(provider)
            .ok_or_else(|| anyhow!("No OAuth config for provider: {}", provider))?;

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
            }
        }

        Ok(url.to_string())
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code_for_token(
        provider: &str,
        code: &str,
        code_verifier: &str,
    ) -> Result<Credential> {
        let config = OAuthConfig::for_provider(provider)
            .ok_or_else(|| anyhow!("No OAuth config for provider: {}", provider))?;

        let client = reqwest::Client::new();
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("code_verifier", code_verifier);
        params.insert("redirect_uri", &config.redirect_uri);

        // Add client_id for Google
        if let Some(client_id) = &config.client_id {
            params.insert("client_id", client_id);
        }

        let response = client
            .post(&config.token_url)
            .form(&params)
            .send()
            .await
            .context("Failed to send token request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Token exchange failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        let expires_at = token_response.expires_in.map(|duration| {
            chrono::Utc::now().timestamp() + duration as i64
        });

        let refresh_token = token_response
            .refresh_token
            .ok_or_else(|| anyhow!("No refresh token in response"))?;

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

    #[test]
    fn test_generate_pkce_verifier() {
        let (verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();
        assert!(!verifier.is_empty());
        assert!(!challenge.is_empty());
        assert!(verifier.len() >= 43);
        assert_ne!(verifier, challenge); // Should be different due to hashing
    }

    #[test]
    fn test_authorization_url_generation() {
        let (_verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();
        let url = CredentialManager::get_authorization_url("anthropic", &challenge, "test-state").unwrap();
        assert!(url.contains("code_challenge="));
        assert!(url.contains("state=test-state"));
    }
}
