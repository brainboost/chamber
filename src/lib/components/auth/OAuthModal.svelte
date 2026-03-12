<script lang="ts">
  import {
    authModalOpen,
    oauthAuthorizationUrl,
    currentAuthProvider,
    authError,
    closeOAuthModal,
    handleOAuthCallback,
    authLoading,
  } from '$lib/stores/credentials';
  import { Button } from '$lib/components/ui';
  import { onMount, onDestroy } from 'svelte';

  let iframe: HTMLIFrameElement;
  let messageListener: ((event: MessageEvent) => void) | null = null;
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let errorType: 'network' | 'timeout' | 'cancelled' | 'other' | null = null;

  // Set up timeout for OAuth flow (5 minutes)
  function setupTimeout() {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => {
      errorType = 'timeout';
      authError.set('Authentication timed out. Please try again.');
      authLoading.set(false);
    }, 5 * 60 * 1000); // 5 minutes
  }

  function clearOAuthTimeout() {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
  }

  // Handle OAuth callback from iframe
  function onMessage(event: MessageEvent) {
    // Verify origin for security
    const allowedOrigins = ['https://claude.ai', 'https://accounts.google.com'];
    if (!allowedOrigins.some((origin) => event.origin === origin)) {
      console.log('Ignoring message from unauthorized origin:', event.origin);
      return;
    }

    try {
      // Parse OAuth callback
      const url = new URL(event.data);
      const code = url.searchParams.get('code');
      const error = url.searchParams.get('error');
      const state = url.searchParams.get('state');

      if (error) {
        // User denied authorization or OAuth error occurred
        errorType = 'cancelled';
        const errorDescription = url.searchParams.get('error_description') || 'Authorization was denied or cancelled.';
        console.warn('OAuth error:', error, errorDescription);
        authError.set(getUserFriendlyErrorMessage(error, errorDescription));
        authLoading.set(false);
        clearOAuthTimeout();
        return;
      }

      if (code && state) {
        console.log('Received OAuth callback for state:', state);
        authLoading.set(true);
        errorType = null;

        handleOAuthCallback(code, state)
          .then(() => {
            console.log('OAuth callback successful');
            clearOAuthTimeout();
          })
          .catch((error) => {
            console.error('OAuth callback error:', error);
            errorType = 'other';
            clearOAuthTimeout();
          });
      }
    } catch (e) {
      // Not a URL or invalid format, ignore
      console.log('Could not parse OAuth callback:', event.data);
    }
  }

  // Get user-friendly error messages
  function getUserFriendlyErrorMessage(error: string, description: string): string {
    const errorMessages: Record<string, string> = {
      'access_denied': 'You denied authorization. If you want to use Chamber with this provider, please grant access when prompted.',
      'invalid_request': 'The authorization request was invalid. Please try again.',
      'unauthorized_client': 'The application is not authorized to use OAuth with this provider.',
      'redirect_uri_mismatch': 'There is a configuration issue with the OAuth redirect URI.',
      'server_error': 'The OAuth server encountered an error. Please try again.',
      'temporarily_unavailable': 'The OAuth service is temporarily unavailable. Please try again later.'
    };

    return errorMessages[error] || description || 'An unknown error occurred during authentication.';
  }

  // Handle iframe load errors
  function handleIframeError() {
    console.error('OAuth iframe failed to load');
    errorType = 'network';
    authError.set('Failed to load the authorization page. Please check your internet connection and try again.');
    authLoading.set(false);
    clearOAuthTimeout();
  }

  // Listen for modal open/close
  $: if ($authModalOpen && $oauthAuthorizationUrl) {
    console.log('Opening OAuth modal for provider:', $currentAuthProvider);
    errorType = null;
    authError.set(null);
    authLoading.set(false);

    // Add message listener when modal opens
    messageListener = onMessage;
    window.addEventListener('message', messageListener);

    // Set up timeout
    setupTimeout();
  } else {
    // Clean up when modal closes
    clearOAuthTimeout();
    if (messageListener) {
      window.removeEventListener('message', messageListener);
      messageListener = null;
    }
    errorType = null;
  }

  function handleClose() {
    clearOAuthTimeout();
    closeOAuthModal();
  }

  function handleRetry() {
    errorType = null;
    authError.set(null);
    authLoading.set(false);

    // Reload iframe
    if (iframe && $oauthAuthorizationUrl) {
      iframe.src = '';
      // Force a reflow to ensure the iframe reloads
      setTimeout(() => {
        iframe.src = $oauthAuthorizationUrl;
        setupTimeout();
      }, 100);
    }
  }

  // Cleanup on component destroy
  onDestroy(() => {
    clearOAuthTimeout();
    if (messageListener) {
      window.removeEventListener('message', messageListener);
    }
  });
</script>

{#if $authModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
    <div class="bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 overflow-hidden">
      <!-- Header -->
      <div class="bg-gray-50 px-6 py-4 border-b border-gray-200 flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold text-gray-900">
            Connect to {$currentAuthProvider || 'Provider'}
          </h2>
          <p class="text-sm text-gray-600 mt-1">
            Sign in to authorize Chamber to access your account
          </p>
        </div>
        <button
          on:click={handleClose}
          class="text-gray-400 hover:text-gray-600 transition-colors"
          aria-label="Close"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6">
        {#if $authError}
          <!-- Error State -->
          <div class="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-red-500 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>

            {#if errorType === 'timeout'}
              <h3 class="text-lg font-semibold text-red-800 mb-2">Authentication Timed Out</h3>
              <p class="text-red-600 mb-4">{$authError}</p>
              <p class="text-sm text-red-500 mb-4">The authorization page took too long to respond. Please check your connection and try again.</p>
            {:else if errorType === 'network'}
              <h3 class="text-lg font-semibold text-red-800 mb-2">Network Error</h3>
              <p class="text-red-600 mb-4">{$authError}</p>
              <p class="text-sm text-red-500 mb-4">Please check your internet connection and ensure you can access the provider's website.</p>
            {:else if errorType === 'cancelled'}
              <h3 class="text-lg font-semibold text-red-800 mb-2">Authorization Cancelled</h3>
              <p class="text-red-600 mb-4">{$authError}</p>
            {:else}
              <h3 class="text-lg font-semibold text-red-800 mb-2">Authentication Failed</h3>
              <p class="text-red-600 mb-4">{$authError}</p>
            {/if}

            <div class="flex gap-3 justify-center">
              <Button variant="primary" onclick={handleRetry}>Try Again</Button>
              <Button variant="secondary" onclick={handleClose}>Cancel</Button>
            </div>

            <!-- Additional help for common errors -->
            {#if errorType !== 'cancelled' && errorType !== 'timeout'}
              <div class="mt-4 pt-4 border-t border-red-200">
                <details class="text-left">
                  <summary class="text-sm text-red-700 cursor-pointer hover:text-red-900">
                    Need help troubleshooting?
                  </summary>
                  <div class="mt-2 text-sm text-red-600 space-y-1">
                    <p>• Make sure you're not blocking popups from this application</p>
                    <p>• Check that your browser allows third-party cookies</p>
                    <p>• Try using a different browser if the problem persists</p>
                    <p>• If using a VPN, try disabling it temporarily</p>
                  </div>
                </details>
              </div>
            {/if}
          </div>
        {:else if $oauthAuthorizationUrl}
          <!-- OAuth iframe -->
          <div class="relative" style="height: 600px;">
            {#if $authLoading}
              <div class="absolute inset-0 flex items-center justify-center bg-white bg-opacity-90 z-10">
                <div class="flex flex-col items-center">
                  <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-3"></div>
                  <p class="text-gray-600">Processing authentication...</p>
                </div>
              </div>
            {/if}
            <iframe
              bind:this={iframe}
              src={$oauthAuthorizationUrl}
              class="w-full h-full border-0"
              title="OAuth Authorization"
            ></iframe>
          </div>
        {:else}
          <!-- Loading State -->
          <div class="flex items-center justify-center h-96">
            <div class="flex flex-col items-center">
              <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-3"></div>
              <p class="text-gray-600">Preparing authorization...</p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="bg-gray-50 px-6 py-4 border-t border-gray-200">
        <p class="text-sm text-gray-600 text-center">
          Your credentials are stored securely using your system's keychain.
        </p>
      </div>
    </div>
  </div>
{/if}
