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
#[serde(tag = "auth_type", rename_all = "snake_case")]
pub enum AuthType {
    /// Traditional API key (e.g., sk-ant-...)
    ApiKey {
        /// The API key value
        key: String,
    },
    /// Bearer token from subscription/setup (e.g., JWT-like tokens)
    BearerToken {
        /// The bearer token value
        token: String,
    },
    /// OAuth token with refresh capability (for providers like Gemini)
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
            AuthType::BearerToken { token } => Some(token.clone()),
            AuthType::OAuthToken { access_token, .. } => Some(access_token.clone()),
        }
    }

    /// Detect the authentication type from a token string (for Anthropic)
    pub fn detect_anthropic_auth_type(token: &str) -> AuthType {
        let trimmed = token.trim();

        // OAuth access tokens (sk-ant-oat...) — use Bearer header
        if trimmed.starts_with("sk-ant-oat") {
            return AuthType::BearerToken {
                token: trimmed.to_string(),
            };
        }

        // JWT-like tokens (3 parts separated by dots) are Bearer tokens
        if trimmed.matches('.').count() >= 2 {
            return AuthType::BearerToken {
                token: trimmed.to_string(),
            };
        }

        // Classic API keys (sk-ant-api...) use x-api-key header
        if trimmed.starts_with("sk-ant") {
            return AuthType::ApiKey {
                key: trimmed.to_string(),
            };
        }

        // Default to API key for backward compatibility
        AuthType::ApiKey {
            key: trimmed.to_string(),
        }
    }

    /// Get the HTTP header name and value for API requests
    pub fn get_auth_header(&self) -> (String, String) {
        match &self.auth_type {
            AuthType::ApiKey { .. } => ("x-api-key".to_string(), self.get_token().unwrap()),
            AuthType::BearerToken { .. } => ("authorization".to_string(), format!("Bearer {}", self.get_token().unwrap())),
            AuthType::OAuthToken { .. } => ("authorization".to_string(), format!("Bearer {}", self.get_token().unwrap())),
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
            AuthType::BearerToken { .. } => false, // Bearer tokens don't expire in the same way
        }
    }

    /// Create a new API key credential
    pub fn api_key(provider: String, key: String) -> Self {
        Self {
            provider,
            auth_type: AuthType::ApiKey { key },
        }
    }

    /// Create a new bearer token credential
    pub fn bearer_token(provider: String, token: String) -> Self {
        Self {
            provider,
            auth_type: AuthType::BearerToken { token },
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
                auth_url: "https://claude.ai/oauth/authorize".to_string(),
                // Token endpoint discovered from Claude Code CLI binary
                token_url: "https://platform.claude.com/v1/oauth/token".to_string(),
                // Scopes used by Claude Code CLI for subscription access
                scopes: vec![
                    "user:profile".to_string(),
                    "user:inference".to_string(),
                    "user:sessions:claude_code".to_string(),
                    "user:mcp_servers".to_string(),
                ],
                // Claude Code CLI's own client_id (public PKCE, no secret needed).
                // Can be overridden with ANTHROPIC_OAUTH_CLIENT_ID env var.
                client_id: Some(
                    std::env::var("ANTHROPIC_OAUTH_CLIENT_ID")
                        .unwrap_or_else(|_| "9d1c250a-e61b-44d9-88ed-5944d1962f5e".to_string())
                ),
                redirect_uri: "chamber://oauth/callback".to_string(),
            }),
            // Gemini uses API keys only — no public OAuth client available
            // "gemini" => ...
            _ => None,
        }
    }

    /// Get OAuth configuration with a custom redirect URI
    pub fn for_provider_with_redirect(provider: &str, redirect_uri: String) -> Option<Self> {
        let mut config = Self::for_provider(provider)?;
        config.redirect_uri = redirect_uri;
        Some(config)
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
