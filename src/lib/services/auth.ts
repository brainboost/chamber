import { invoke } from '@tauri-apps/api/core';
import type { Credential } from '$lib/types/config';

/**
 * Start OAuth flow for a provider
 * Returns the authorization URL to open
 */
export async function startOAuthFlow(provider: string): Promise<string> {
  return await invoke<string>('start_oauth_flow', { provider });
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
