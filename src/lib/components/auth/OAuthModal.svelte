<script lang="ts">
  import {
    authModalOpen,
    oauthAuthorizationUrl,
    currentAuthProvider,
    authError,
    closeOAuthModal,
    authLoading,
  } from '$lib/stores/credentials';
  import { Button } from '$lib/components/ui';

  function handleClose() {
    closeOAuthModal();
  }
</script>

{#if $authModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-lg w-full mx-4 overflow-hidden">
      <!-- Header -->
      <div class="bg-gray-50 dark:bg-gray-700 px-6 py-4 border-b border-gray-200 dark:border-gray-600 flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
            Connect to {$currentAuthProvider || 'Provider'}
          </h2>
          <p class="text-sm text-gray-600 dark:text-gray-300 mt-1">
            Complete the authorization in your browser
          </p>
        </div>
        <button
          on:click={handleClose}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
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
          <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6 text-center">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-red-500 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>

            <h3 class="text-lg font-semibold text-red-800 dark:text-red-400 mb-2">Authentication Failed</h3>
            <p class="text-red-600 dark:text-red-500 mb-4">{$authError}</p>

            <div class="flex gap-3 justify-center">
              <Button variant="secondary" onclick={handleClose}>Close</Button>
            </div>
          </div>
        {:else if $authLoading}
          <!-- Loading State -->
          <div class="flex flex-col items-center py-8">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"></div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Processing Authentication</h3>
            <p class="text-sm text-gray-600 dark:text-gray-400 text-center">
              Completing OAuth flow... This window will close automatically when successful.
            </p>
          </div>
        {:else}
          <!-- Waiting State -->
          <div class="flex flex-col items-center py-8">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 text-blue-600 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 15l-2 5L9 9l11 4-5 2zm0 0l5 5M7.188 2.239l.777 2.897M5.136 7.965l-2.898-.777M13.95 4.05l-2.122 2.122m-5.657 5.656l-2.12 2.122" />
            </svg>

            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-3">
              Browser Opened
            </h3>

            <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 max-w-md">
              <p class="text-sm text-blue-800 dark:text-blue-300 text-center mb-4">
                We've opened your browser to complete the OAuth authorization with {$currentAuthProvider}.
              </p>
              <div class="text-left text-sm text-blue-700 dark:text-blue-400 space-y-2">
                <p class="flex items-start gap-2">
                  <svg class="h-5 w-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                  </svg>
                  <span>Sign in to your {$currentAuthProvider} account in the browser</span>
                </p>
                <p class="flex items-start gap-2">
                  <svg class="h-5 w-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                  </svg>
                  <span>Authorize Chamber to access your account</span>
                </p>
                <p class="flex items-start gap-2">
                  <svg class="h-5 w-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                  </svg>
                  <span>This window will close automatically when complete</span>
                </p>
              </div>
            </div>

            {#if $oauthAuthorizationUrl}
              <p class="text-sm text-gray-600 dark:text-gray-400 mt-4 max-w-md text-center">
                {$oauthAuthorizationUrl}
              </p>
            {/if}

            <div class="mt-6 flex gap-3">
              <Button variant="secondary" onclick={handleClose}>Cancel</Button>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="bg-gray-50 dark:bg-gray-700 px-6 py-4 border-t border-gray-200 dark:border-gray-600">
        <p class="text-sm text-gray-600 dark:text-gray-400 text-center">
          Your credentials are stored securely using your system's keychain.
        </p>
      </div>
    </div>
  </div>
{/if}
