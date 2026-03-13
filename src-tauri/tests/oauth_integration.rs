//! Integration tests for OAuth authentication flow
//!
//! These tests verify the complete OAuth flow from authorization through token storage.
//! They test the integration between the credential manager, OAuth config, and credential models.

use chamber::models::auth::{AuthType, Credential, OAuthConfig};
use chamber::services::CredentialManager;
use std::sync::Arc;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Helper to get current timestamp for testing
    fn get_test_timestamp() -> i64 {
        chrono::Utc::now().timestamp()
    }

    /// Helper to create a future timestamp for testing
    fn get_future_timestamp(seconds: i64) -> i64 {
        get_test_timestamp() + seconds
    }

    #[tokio::test]
    async fn test_oauth_flow_complete_anthropic() {
        // This test verifies the complete OAuth flow for Anthropic
        // Note: In a real test, we would use the mock server above
        // For now, we test the components that don't require network calls

        let manager = CredentialManager::new();

        // Step 1: Generate PKCE verifier
        let (verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();
        assert!(!verifier.is_empty());
        assert!(!challenge.is_empty());

        // Step 2: Generate authorization URL
        let state = "test-state-123";
        let auth_url = CredentialManager::get_authorization_url("anthropic", &challenge, state).unwrap();
        assert!(auth_url.contains("claude.ai"));
        assert!(auth_url.contains(&challenge));
        assert!(auth_url.contains(state));

        // Step 3: Simulate storing the credential (skip actual token exchange in test)
        // In real flow, user would authorize and we'd get a code
        let _mock_code = "test_authorization_code";
        let mock_access_token = "test_access_token";
        let mock_refresh_token = "test_refresh_token";
        let expires_at = chrono::Utc::now().timestamp() + 3600;

        let credential = Credential::oauth_token(
            "anthropic".to_string(),
            mock_access_token.to_string(),
            mock_refresh_token.to_string(),
            Some(expires_at),
            vec!["openid".to_string(), "profile".to_string()],
        );

        // Step 4: Store credential
        manager.store_credential(credential.clone()).await.unwrap();

        // Step 5: Retrieve credential
        let retrieved = manager.get_credential("anthropic").await.unwrap().unwrap();
        assert_eq!(retrieved.provider, "anthropic");
        assert_eq!(retrieved.get_token().unwrap(), mock_access_token);

        // Step 6: Verify token is not expired
        assert!(!retrieved.needs_refresh());
    }

    #[tokio::test]
    async fn test_oauth_flow_complete_gemini() {
        let manager = CredentialManager::new();

        // PKCE generation
        let (_verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();

        // Authorization URL for Gemini
        let auth_url = CredentialManager::get_authorization_url("gemini", &challenge, "test-state").unwrap();
        assert!(auth_url.contains("accounts.google.com"));
        assert!(auth_url.contains("client_id"));

        // Create and store credential
        let credential = Credential::oauth_token(
            "gemini".to_string(),
            "test_access_token_gemini".to_string(),
            "test_refresh_token_gemini".to_string(),
            Some(get_future_timestamp(3600)),
            vec!["https://www.googleapis.com/auth/generative.language".to_string()],
        );

        manager.store_credential(credential).await.unwrap();

        // Verify retrieval
        let retrieved = manager.get_credential("gemini").await.unwrap().unwrap();
        assert_eq!(retrieved.provider, "gemini");
    }

    #[tokio::test]
    async fn test_oauth_flow_token_expiry() {
        let manager = CredentialManager::new();

        // Create a credential that will expire soon
        let expires_soon = get_future_timestamp(200); // 200 seconds

        let credential = Credential::oauth_token(
            "anthropic".to_string(),
            "test_token".to_string(),
            "test_refresh".to_string(),
            Some(expires_soon),
            vec!["openid".to_string()],
        );

        manager.store_credential(credential).await.unwrap();

        // Retrieve and check if it needs refresh
        let retrieved = manager.get_credential("anthropic").await.unwrap().unwrap();
        assert!(retrieved.needs_refresh());
    }

    #[tokio::test]
    async fn test_oauth_flow_multiple_providers() {
        let manager = CredentialManager::new();

        // Store credentials for multiple providers
        let anthropic_cred = Credential::oauth_token(
            "anthropic".to_string(),
            "anthropic_token".to_string(),
            "anthropic_refresh".to_string(),
            Some(get_future_timestamp(3600)),
            vec!["openid".to_string()],
        );

        let gemini_cred = Credential::oauth_token(
            "gemini".to_string(),
            "gemini_token".to_string(),
            "gemini_refresh".to_string(),
            Some(get_future_timestamp(3600)),
            vec!["scope".to_string()],
        );

        let xai_cred = Credential::api_key("xai".to_string(), "xai_key".to_string());

        manager.store_credential(anthropic_cred).await.unwrap();
        manager.store_credential(gemini_cred).await.unwrap();
        manager.store_credential(xai_cred).await.unwrap();

        // Verify all are stored
        let providers = manager.list_providers().await.unwrap();
        assert_eq!(providers.len(), 3);
        assert!(providers.contains(&"anthropic".to_string()));
        assert!(providers.contains(&"gemini".to_string()));
        assert!(providers.contains(&"xai".to_string()));

        // Verify credentials as env vars
        let env_vars = manager.get_credentials_as_env().await.unwrap();
        assert_eq!(env_vars.get("ANTHROPIC_API_KEY"), Some(&"anthropic_token".to_string()));
        assert_eq!(env_vars.get("GOOGLE_API_KEY"), Some(&"gemini_token".to_string()));
        assert_eq!(env_vars.get("XAI_API_KEY"), Some(&"xai_key".to_string()));
    }

    #[tokio::test]
    async fn test_oauth_flow_error_handling() {
        let manager = CredentialManager::new();

        // Test getting non-existent credential
        let result = manager.get_credential("nonexistent").await.unwrap();
        assert!(result.is_none());

        // Test deleting non-existent credential (should not error)
        let result = manager.delete_credential("nonexistent").await;
        assert!(result.is_ok()); // Should succeed silently
    }

    #[tokio::test]
    async fn test_oauth_flow_credential_update() {
        let manager = CredentialManager::new();

        // Store initial credential
        let cred1 = Credential::api_key("anthropic".to_string(), "old_key".to_string());
        manager.store_credential(cred1).await.unwrap();

        // Verify it's there
        let retrieved = manager.get_credential("anthropic").await.unwrap().unwrap();
        assert_eq!(retrieved.get_token().unwrap(), "old_key");

        // Update with new credential
        let cred2 = Credential::api_key("anthropic".to_string(), "new_key".to_string());
        manager.store_credential(cred2).await.unwrap();

        // Verify it's updated
        let retrieved = manager.get_credential("anthropic").await.unwrap().unwrap();
        assert_eq!(retrieved.get_token().unwrap(), "new_key");
    }

    #[tokio::test]
    async fn test_oauth_flow_provider_configurations() {
        // Test that all provider configurations are valid
        let providers = vec!["anthropic", "gemini"];

        for provider in providers {
            let config = OAuthConfig::for_provider(provider);
            assert!(config.is_some(), "Provider {} should have OAuth config", provider);

            let config = config.unwrap();
            assert!(!config.auth_url.is_empty());
            assert!(!config.token_url.is_empty());
            assert!(!config.scopes.is_empty());
            assert!(!config.redirect_uri.is_empty());
            assert_eq!(config.provider, provider);
        }
    }

    #[tokio::test]
    async fn test_oauth_flow_invalid_provider() {
        // Test that invalid provider returns None
        let config = OAuthConfig::for_provider("invalid_provider");
        assert!(config.is_none());

        // Test authorization URL with invalid provider
        let (_verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();
        let result = CredentialManager::get_authorization_url("invalid_provider", &challenge, "state");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_oauth_flow_serialization_roundtrip() {
        // Create a complex OAuth credential
        let original = Credential::oauth_token(
            "test_provider".to_string(),
            "access123".to_string(),
            "refresh456".to_string(),
            Some(1234567890),
            vec!["scope1".to_string(), "scope2".to_string(), "scope3".to_string()],
        );

        // Serialize
        let serialized = serde_json::to_string(&original).unwrap();

        // Deserialize
        let deserialized: Credential = serde_json::from_str(&serialized).unwrap();

        // Verify all fields match
        assert_eq!(original.provider, deserialized.provider);
        assert_eq!(original.get_token(), deserialized.get_token());

        match (original.auth_type, deserialized.auth_type) {
            (AuthType::OAuthToken { access_token: at1, refresh_token: rt1, expires_at: ea1, scopes: s1 },
             AuthType::OAuthToken { access_token: at2, refresh_token: rt2, expires_at: ea2, scopes: s2 }) => {
                assert_eq!(at1, at2);
                assert_eq!(rt1, rt2);
                assert_eq!(ea1, ea2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Auth types don't match after serialization roundtrip"),
        }
    }

    #[tokio::test]
    async fn test_oauth_flow_cache_consistency() {
        let manager = CredentialManager::new();

        // Store a credential
        let credential = Credential::api_key("test_provider".to_string(), "test_key".to_string());
        manager.store_credential(credential.clone()).await.unwrap();

        // Retrieve multiple times
        let retrieved1 = manager.get_credential("test_provider").await.unwrap().unwrap();
        let retrieved2 = manager.get_credential("test_provider").await.unwrap().unwrap();

        // Verify consistency
        assert_eq!(retrieved1.provider, retrieved2.provider);
        assert_eq!(retrieved1.get_token(), retrieved2.get_token());

        // Verify cache is being used (second retrieval should be faster)
        // In a real test, we'd measure timing, but for now we just verify correctness
    }

    #[tokio::test]
    async fn test_oauth_flow_delete_and_retrieve() {
        let manager = CredentialManager::new();

        // Store credential
        let credential = Credential::api_key("test_provider".to_string(), "test_key".to_string());
        manager.store_credential(credential).await.unwrap();

        // Verify it exists
        assert!(manager.get_credential("test_provider").await.unwrap().is_some());

        // Delete it
        manager.delete_credential("test_provider").await.unwrap();

        // Verify it's gone
        assert!(manager.get_credential("test_provider").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_oauth_flow_list_providers_empty() {
        let manager = CredentialManager::new();

        // List providers when none are stored
        let providers = manager.list_providers().await.unwrap();
        assert!(providers.is_empty());
    }

    #[tokio::test]
    async fn test_oauth_flow_get_credentials_as_env_empty() {
        let manager = CredentialManager::new();

        // Get env vars when no credentials stored
        let env_vars = manager.get_credentials_as_env().await.unwrap();
        assert!(env_vars.is_empty());
    }

    #[tokio::test]
    async fn test_oauth_flow_pkce_uniqueness() {
        // Generate multiple PKCE verifiers
        let (v1, c1) = CredentialManager::generate_pkce_verifier().unwrap();
        let (v2, c2) = CredentialManager::generate_pkce_verifier().unwrap();
        let (v3, c3) = CredentialManager::generate_pkce_verifier().unwrap();

        // All should be different (random)
        assert_ne!(v1, v2);
        assert_ne!(v2, v3);
        assert_ne!(v1, v3);
        assert_ne!(c1, c2);
        assert_ne!(c2, c3);
        assert_ne!(c1, c3);
    }

    #[tokio::test]
    async fn test_oauth_flow_state_parameter_validation() {
        // Test that state parameter is properly handled
        let state1 = "valid-state-123";
        let state2 = "another-valid-state-456";

        let (_verifier, challenge) = CredentialManager::generate_pkce_verifier().unwrap();

        let url1 = CredentialManager::get_authorization_url("anthropic", &challenge, state1).unwrap();
        let url2 = CredentialManager::get_authorization_url("anthropic", &challenge, state2).unwrap();

        // State parameters should be different
        assert!(url1.contains(&format!("state={}", state1)));
        assert!(url2.contains(&format!("state={}", state2)));
        assert!(!url1.contains(state2));
        assert!(!url2.contains(state1));
    }

    #[tokio::test]
    async fn test_oauth_flow_token_type_validation() {
        // Test that only OAuth tokens have refresh capabilities
        let api_key_cred = Credential::api_key("test".to_string(), "key".to_string());
        let oauth_cred = Credential::oauth_token(
            "test".to_string(),
            "access".to_string(),
            "refresh".to_string(),
            None,
            vec![],
        );

        // API key should never need refresh
        assert!(!api_key_cred.needs_refresh());

        // OAuth token without expiry also doesn't need refresh
        assert!(!oauth_cred.needs_refresh());

        // OAuth token with past expiry should need refresh
        let expired_oauth = Credential::oauth_token(
            "test".to_string(),
            "access".to_string(),
            "refresh".to_string(),
            Some(chrono::Utc::now().timestamp() - 100), // Expired 100s ago
            vec![],
        );
        assert!(expired_oauth.needs_refresh());
    }

    #[tokio::test]
    async fn test_oauth_flow_concurrent_operations() {
        // Test that the manager handles concurrent operations correctly
        let manager = Arc::new(CredentialManager::new());
        let mut handles = vec![];

        // Spawn multiple concurrent store operations
        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let cred = Credential::api_key(
                    format!("provider_{}", i),
                    format!("key_{}", i),
                );
                manager_clone.store_credential(cred).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            assert!(handle.await.is_ok());
        }

        // Verify all were stored
        let providers: Vec<String> = manager.list_providers().await.unwrap();
        assert_eq!(providers.len(), 5);
    }
}
