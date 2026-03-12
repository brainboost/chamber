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

  let iframe: HTMLIFrameElement;
  let messageListener: ((event: MessageEvent) => void) | null = null;

  // Handle OAuth callback from iframe
  function onMessage(event: MessageEvent) {
    // Verify origin for security
    const allowedOrigins = ['https://claude.ai', 'https://accounts.google.com'];
    if (!allowedOrigins.some((origin) => event.origin === origin)) {
      return;
    }

    try {
      // Parse OAuth callback
      const url = new URL(event.data);
      const code = url.searchParams.get('code');
      const state = url.searchParams.get('state');

      if (code && state) {
        handleOAuthCallback(code, state)
          .then(() => {
            // Success - modal will close automatically
          })
          .catch((error) => {
            console.error('OAuth callback error:', error);
          });
      }
    } catch (e) {
      // Not a URL, ignore
    }
  }

  // Listen for modal open/close
  $: if ($authModalOpen && $oauthAuthorizationUrl) {
    // Add message listener when modal opens
    messageListener = onMessage;
    window.addEventListener('message', messageListener);
  } else {
    // Remove listener when modal closes
    if (messageListener) {
      window.removeEventListener('message', messageListener);
      messageListener = null;
    }
  }

  function handleClose() {
    closeOAuthModal();
  }

  function handleRetry() {
    // Reload iframe
    if (iframe) {
      iframe.src = $oauthAuthorizationUrl || '';
    }
  }
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
            <h3 class="text-lg font-semibold text-red-800 mb-2">Authentication Failed</h3>
            <p class="text-red-600 mb-4">{$authError}</p>
            <div class="flex gap-3 justify-center">
              <Button variant="secondary" onclick={handleRetry}>Try Again</Button>
              <Button variant="secondary" onclick={handleClose}>Cancel</Button>
            </div>
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
