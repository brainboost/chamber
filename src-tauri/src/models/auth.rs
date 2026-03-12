use serde::{Deserialize, Serialize};

/// Represents a stored credential for an LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Provider name (e.g., "anthropic", "gemini", "xai")
    pub provider: String,
    /// Type of authentication
    #[serde(flatten)]
    pub auth_type: AuthType,
}

/// Type of authentication credential
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthType {
    /// Traditional API key stored in config
    ApiKey {
        /// The API key value
        key: String,
    },
    /// OAuth token with refresh capability
    OAuthToken {
        /// Current access token
        access_token: String,
        /// Refresh token for obtaining new access tokens
        refresh_token: String,
        /// Token expiration timestamp (Unix seconds)
        expires_at: Option<i64>,
        /// OAuth scopes granted
        scopes: Vec<String>,
    },
}

impl Credential {
    /// Get the actual token/key value for API requests
    pub fn get_token(&self) -> Option<String> {
        match &self.auth_type {
            AuthType::ApiKey { key } => Some(key.clone()),
            AuthType::OAuthToken { access_token, .. } => Some(access_token.clone()),
        }
    }

    /// Check if an OAuth token needs refreshing
    pub fn needs_refresh(&self) -> bool {
        match &self.auth_type {
            AuthType::OAuthToken { expires_at, .. } => {
                if let Some(expiry) = expires_at {
                    let now = chrono::Utc::now().timestamp();
                    // Refresh if token expires within 5 minutes
                    now + 300 > *expiry
                } else {
                    false
                }
            }
            AuthType::ApiKey { .. } => false,
        }
    }

    /// Create a new API key credential
    pub fn api_key(provider: String, key: String) -> Self {
        Self {
            provider,
            auth_type: AuthType::ApiKey { key },
        }
    }

    /// Create a new OAuth token credential
    pub fn oauth_token(
        provider: String,
        access_token: String,
        refresh_token: String,
        expires_at: Option<i64>,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            provider,
            auth_type: AuthType::OAuthToken {
                access_token,
                refresh_token,
                expires_at,
                scopes,
            },
        }
    }
}

/// OAuth provider configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// Provider identifier
    pub provider: String,
    /// Authorization endpoint URL
    pub auth_url: String,
    /// Token endpoint URL
    pub token_url: String,
    /// OAuth scopes to request
    pub scopes: Vec<String>,
    /// Client ID (if applicable, some providers don't require it)
    pub client_id: Option<String>,
    /// Redirect URI for OAuth callback
    pub redirect_uri: String,
}

impl OAuthConfig {
    /// Get OAuth configuration for supported providers
    pub fn for_provider(provider: &str) -> Option<Self> {
        match provider {
            "anthropic" => Some(OAuthConfig {
                provider: "anthropic".to_string(),
                auth_url: "https://claude.ai/oauthauthorize".to_string(),
                token_url: "https://claude.ai/api/oauth/token".to_string(),
                scopes: vec!["openid".to_string(), "profile".to_string(), "offline_access".to_string()],
                client_id: None,
                redirect_uri: "chamber://oauth/callback".to_string(),
            }),
            "gemini" => Some(OAuthConfig {
                provider: "gemini".to_string(),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                scopes: vec!["https://www.googleapis.com/auth/generative.language".to_string()],
                client_id: Some(
                    "684799299962-6kgfpt7cdq1q7jcji9tpgs3f9c3j7j9e.apps.googleusercontent.com"
                        .to_string(),
                ),
                redirect_uri: "chamber://oauth/callback".to_string(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_api_key() {
        let cred = Credential::api_key("anthropic".to_string(), "sk-test-key".to_string());
        assert_eq!(cred.get_token().unwrap(), "sk-test-key");
        assert!(!cred.needs_refresh());
    }

    #[test]
    fn test_oauth_token_needs_refresh() {
        let expired_cred = Credential::oauth_token(
            "anthropic".to_string(),
            "access-token".to_string(),
            "refresh-token".to_string(),
            Some(chrono::Utc::now().timestamp() - 100), // Expired
            vec!["scope1".to_string()],
        );
        assert!(expired_cred.needs_refresh());

        let valid_cred = Credential::oauth_token(
            "anthropic".to_string(),
            "access-token".to_string(),
            "refresh-token".to_string(),
            Some(chrono::Utc::now().timestamp() + 1000), // Valid for a while
            vec!["scope1".to_string()],
        );
        assert!(!valid_cred.needs_refresh());
    }
}
