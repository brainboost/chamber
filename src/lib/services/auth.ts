import { invoke } from '@tauri-apps/api/core';
import type { Credential } from '$lib/types/config';

/**
 * Start OAuth flow for a provider
 * Returns the authorization URL and callback port
 */
export async function startOAuthFlow(provider: string): Promise<[string, number]> {
  return await invoke<[string, number]>('start_oauth_flow', { provider });
}

/**
 * Open OAuth URL in the default browser
 */
export async function openOAuthUrl(url: string): Promise<void> {
  return await invoke('open_oauth_url', { url });
}

/**
 * Handle OAuth callback
 * Exchanges the authorization code for tokens
 */
export async function handleOAuthCallback(code: string, state: string): Promise<Credential> {
  return await invoke<Credential>('handle_oauth_callback', { code, stateParam: state });
}

/**
 * Save a credential (for API keys or manual token input)
 */
export async function saveCredential(credential: Credential): Promise<void> {
  return await invoke('save_credential', { credential });
}

/**
 * Get a credential for a provider
 */
export async function getCredential(provider: string): Promise<Credential | null> {
  return await invoke<Credential | null>('get_credential', { provider });
}

/**
 * Delete a credential for a provider
 */
export async function deleteCredential(provider: string): Promise<void> {
  return await invoke('delete_credential', { provider });
}

/**
 * List all providers with stored credentials
 */
export async function listCredentials(): Promise<string[]> {
  return await invoke<string[]>('list_credentials');
}

/**
 * Check if a provider has a stored credential
 */
export async function hasCredential(provider: string): Promise<boolean> {
  return await invoke<boolean>('has_credential', { provider });
}

/**
 * Refresh OAuth token if needed
 */
export async function refreshCredential(provider: string): Promise<Credential | null> {
  return await invoke<Credential | null>('refresh_credential', { provider });
}

/**
 * Check if Claude Code CLI credentials are available to import
 */
export async function checkClaudeCodeCredential(): Promise<boolean> {
  return await invoke<boolean>('check_claude_code_credential');
}

/**
 * Push current credentials from keychain to the running sidecar process.
 * Returns true if the sidecar accepted them, false if it wasn't reachable.
 */
export async function pushCredentialsToSidecar(): Promise<boolean> {
  return await invoke<boolean>('push_credentials_to_sidecar');
}

/**
 * Open an in-app OAuth webview for Anthropic.
 * The webview intercepts the redirect to https://claude.ai/oauth/code/callback
 * and exchanges the code for tokens automatically.
 * Result is delivered via the 'oauth-success' / 'oauth-error' Tauri events.
 */
export async function openAnthropicOAuthWebview(): Promise<void> {
  return await invoke('open_anthropic_oauth_webview');
}

/**
 * Import Anthropic credentials from Claude Code CLI (~/.claude/.credentials.json)
 * Returns true if credentials were imported successfully
 */
export async function importClaudeCodeCredential(): Promise<boolean> {
  return await invoke<boolean>('import_claude_code_credential');
}

/**
 * Check if .env file exists and has credentials to migrate
 */
export async function checkEnvFileForMigration(envPath?: string): Promise<boolean> {
  return await invoke<boolean>('check_env_file_for_migration', { envPath });
}

/**
 * Migrate credentials from .env file to keychain storage
 * Returns list of provider names that were migrated
 */
export async function migrateFromEnvFile(envPath?: string): Promise<string[]> {
  return await invoke<string[]>('migrate_from_env_file', { envPath });
}

/**
 * Create an API key credential
 */
export function createApiKeyCredential(provider: string, key: string): Credential {
  return {
    provider,
    auth_type: 'api_key',
    key,
  };
}

/**
 * Detect the credential type from the token string (for Anthropic)
 * Automatically detects between API keys and Bearer tokens
 */
export function detectCredentialType(provider: string, token: string): Credential {
  const trimmed = token.trim();

  // OAuth subscription tokens (sk-ant-oat...) — bearer token
  if (trimmed.startsWith('sk-ant-oat')) {
    return { provider, auth_type: 'bearer_token', token: trimmed };
  }

  // JWT-like tokens (3 parts separated by dots) — bearer token
  if (trimmed.split('.').length >= 3) {
    return { provider, auth_type: 'bearer_token', token: trimmed };
  }

  // Classic API keys (sk-ant-api...)
  if (trimmed.startsWith('sk-ant')) {
    return { provider, auth_type: 'api_key', key: trimmed };
  }

  // Default to API key
  return { provider, auth_type: 'api_key', key: trimmed };
}

/**
 * Create an OAuth token credential
 */
export function createOAuthCredential(
  provider: string,
  accessToken: string,
  refreshToken: string,
  expiresAt?: number,
  scopes?: string[]
): Credential {
  return {
    provider,
    auth_type: 'oauth_token',
    access_token: accessToken,
    refresh_token: refreshToken,
    expires_at: expiresAt,
    scopes: scopes || [],
  };
}
