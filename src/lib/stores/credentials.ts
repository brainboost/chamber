import { writable, type Writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { Credential, ProviderAuthStatus } from '$lib/types/config';
import * as auth from '$lib/services/auth';
import { pushCredentialsToSidecar } from '$lib/services/auth';

// Map of provider -> Credential
export const credentials: Writable<Record<string, Credential>> = writable({});

// OAuth modal state
export const authModalOpen: Writable<boolean> = writable(false);

// Current provider being authenticated
export const currentAuthProvider: Writable<string | null> = writable(null);

// OAuth authorization URL (for the modal to display)
export const oauthAuthorizationUrl: Writable<string | null> = writable(null);

// Loading state for auth operations
export const authLoading: Writable<boolean> = writable(false);

// Error state for auth operations
export const authError: Writable<string | null> = writable(null);

// Provider authentication status
export const providerStatus: Writable<Record<string, ProviderAuthStatus>> = writable({});

/**
 * Load all credentials from secure storage
 */
export async function loadCredentials(): Promise<void> {
  try {
    authLoading.set(true);
    authError.set(null);

    const providers = await auth.listCredentials();
    const creds: Record<string, Credential> = {};
    const statuses: Record<string, ProviderAuthStatus> = {};

    for (const provider of providers) {
      const cred = await auth.getCredential(provider);
      if (cred) {
        creds[provider] = cred;
        statuses[provider] = {
          provider,
          has_credential: true,
          auth_type: cred.auth_type,
          needs_refresh: cred.auth_type === 'oauth_token' &&
            cred.expires_at ? cred.expires_at < Date.now() + 300000 : false, // 5 min buffer
        };
      }
    }

    credentials.set(creds);
    providerStatus.set(statuses);
  } catch (error) {
    console.error('Failed to load credentials:', error);
    authError.set(error instanceof Error ? error.message : 'Failed to load credentials');
    throw error;
  } finally {
    authLoading.set(false);
  }
}

/**
 * Save a credential securely
 */
export async function saveCredential(credential: Credential): Promise<void> {
  try {
    authLoading.set(true);
    authError.set(null);

    await auth.saveCredential(credential);

    // Update local store
    credentials.update((creds) => ({
      ...creds,
      [credential.provider]: credential,
    }));

    // Update status
    providerStatus.update((statuses) => ({
      ...statuses,
      [credential.provider]: {
        provider: credential.provider,
        has_credential: true,
        auth_type: credential.auth_type,
      },
    }));
  } catch (error) {
    console.error('Failed to save credential:', error);
    authError.set(error instanceof Error ? error.message : 'Failed to save credential');
    throw error;
  } finally {
    authLoading.set(false);
  }
}

/**
 * Delete a credential
 */
export async function deleteCredential(provider: string): Promise<void> {
  try {
    authLoading.set(true);
    authError.set(null);

    await auth.deleteCredential(provider);

    // Update local store
    credentials.update((creds) => {
      const newCreds = { ...creds };
      delete newCreds[provider];
      return newCreds;
    });

    // Update status
    providerStatus.update((statuses) => ({
      ...statuses,
      [provider]: {
        provider,
        has_credential: false,
      },
    }));
  } catch (error) {
    console.error('Failed to delete credential:', error);
    authError.set(error instanceof Error ? error.message : 'Failed to delete credential');
    throw error;
  } finally {
    authLoading.set(false);
  }
}

/**
 * Start OAuth flow for a provider
 */
export async function startOAuthFlow(provider: string): Promise<void> {
  try {
    authLoading.set(true);
    authError.set(null);
    currentAuthProvider.set(provider);
    authModalOpen.set(true);

    // Start OAuth flow and get auth URL + callback port
    const [authUrl, port] = await auth.startOAuthFlow(provider);

    // Open the authorization URL in the default browser
    await auth.openOAuthUrl(authUrl);

    // Listen for OAuth success/error events from Tauri
    await setupOAuthListeners();

    // Show a message in the modal
    oauthAuthorizationUrl.set(`Browser opened. Please complete the authorization in your browser and return here.`);
  } catch (error) {
    console.error('Failed to start OAuth flow:', error);
    authError.set(error instanceof Error ? error.message : 'Failed to start OAuth flow');
    authModalOpen.set(false);
    currentAuthProvider.set(null);
    throw error;
  } finally {
    authLoading.set(false);
  }
}

/**
 * Set up event listeners for OAuth callbacks (idempotent).
 * Call before opening any OAuth flow so events are wired up.
 */
export async function setupOAuthListeners(): Promise<void> {
  if (typeof window === 'undefined') return;
  if ((window as any).__oauthListenersSetup) return;
  (window as any).__oauthListenersSetup = true;

  const unlistenSuccess = await listen<string>('oauth-success', (event) => {
    console.log('OAuth successful for provider:', event.payload);
    handleOAuthSuccess(event.payload);
  });

  const unlistenError = await listen<string>('oauth-error', (event) => {
    console.error('OAuth error:', event.payload);
    handleOAuthError(event.payload);
  });

  (window as any).__oauthUnlisten = [unlistenSuccess, unlistenError];
}

/**
 * Handle successful OAuth authentication
 */
async function handleOAuthSuccess(provider: string) {
  try {
    authLoading.set(true);

    // Reload credentials to get the newly stored one
    await loadCredentials();

    // Push new token to sidecar so it can be used immediately
    await pushCredentialsToSidecar().catch(() => {/* sidecar not running — no-op */});

    // Close modal
    authModalOpen.set(false);
    currentAuthProvider.set(null);
    oauthAuthorizationUrl.set(null);
    authError.set(null);
  } catch (error) {
    console.error('Failed to handle OAuth success:', error);
    authError.set(error instanceof Error ? error.message : 'Failed to complete authentication');
  } finally {
    authLoading.set(false);
  }
}

/**
 * Handle OAuth error
 */
function handleOAuthError(error: string) {
  authLoading.set(false);
  authError.set(error);
  // Don't close modal - let user see the error and retry
}

/**
 * Handle OAuth callback (called from the modal)
 */
export async function handleOAuthCallback(code: string, state: string): Promise<void> {
  try {
    authLoading.set(true);
    authError.set(null);

    const credential = await auth.handleOAuthCallback(code, state);

    // Update local store
    credentials.update((creds) => ({
      ...creds,
      [credential.provider]: credential,
    }));

    // Update status
    providerStatus.update((statuses) => ({
      ...statuses,
      [credential.provider]: {
        provider: credential.provider,
        has_credential: true,
        auth_type: credential.auth_type,
      },
    }));

    // Close modal
    authModalOpen.set(false);
    currentAuthProvider.set(null);
    oauthAuthorizationUrl.set(null);
  } catch (error) {
    console.error('Failed to handle OAuth callback:', error);
    authError.set(error instanceof Error ? error.message : 'Failed to complete OAuth flow');
    throw error;
  } finally {
    authLoading.set(false);
  }
}

/**
 * Refresh a credential if it's an OAuth token
 */
export async function refreshCredential(provider: string): Promise<void> {
  try {
    authLoading.set(true);
    authError.set(null);

    const credential = await auth.refreshCredential(provider);
    if (credential) {
      // Update local store
      credentials.update((creds) => ({
        ...creds,
        [provider]: credential,
      }));

      // Update status
      providerStatus.update((statuses) => ({
        ...statuses,
        [provider]: {
          provider,
          has_credential: true,
          auth_type: credential.auth_type,
          needs_refresh: false,
        },
      }));
    }
  } catch (error) {
    console.error('Failed to refresh credential:', error);
    authError.set(error instanceof Error ? error.message : 'Failed to refresh credential');
    throw error;
  } finally {
    authLoading.set(false);
  }
}

/**
 * Close OAuth modal
 */
export function closeOAuthModal(): void {
  authModalOpen.set(false);
  currentAuthProvider.set(null);
  oauthAuthorizationUrl.set(null);
  authError.set(null);
}

/**
 * Clear auth error
 */
export function clearAuthError(): void {
  authError.set(null);
}
