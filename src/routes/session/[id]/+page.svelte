<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { Button } from '$lib/components/ui';
  import { sessionStore } from '$lib/stores/session';
  import ChatInterface from '$lib/components/chat/ChatInterface.svelte';
  import type { Session } from '$lib/types/session';

  let sessionId = $derived($page.params.id);
  let session = $state<Session | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    await loadSession();
  });

  async function loadSession() {
    try {
      isLoading = true;
      error = null;
      if (sessionId) {
        session = await sessionStore.getSession(sessionId);
      } else {
        error = "No session ID provided";
      }
    } catch (err) {
      console.error('Failed to load session:', err);
      error = err instanceof Error ? err.message : 'Failed to load session';
    } finally {
      isLoading = false;
    }
  }

  async function handlePause() {
    if (!session) return;
    try {
      await sessionStore.pauseSession(session.id);
      await loadSession(); // Reload to get updated status
    } catch (err) {
      console.error('Failed to pause session:', err);
    }
  }

  async function handleResume() {
    if (!session) return;
    try {
      await sessionStore.resumeSession(session.id);
      await loadSession(); // Reload to get updated status
    } catch (err) {
      console.error('Failed to resume session:', err);
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'active': return 'bg-green-500';
      case 'paused': return 'bg-yellow-500';
      case 'completed': return 'bg-blue-500';
      case 'failed': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  }
</script>

<div class="h-full flex flex-col">
  {#if isLoading}
    <div class="flex items-center justify-center h-full">
      <div class="text-center">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
        <p class="text-gray-600">Loading session...</p>
      </div>
    </div>
  {:else if error}
    <div class="flex items-center justify-center h-full">
      <div class="text-center">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 text-red-500 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="text-lg font-medium text-gray-900 mb-2">Failed to load session</p>
        <p class="text-sm text-gray-600 mb-4">{error}</p>
        <Button variant="primary" onclick={() => window.location.href = '/'}>
          Back to Dashboard
        </Button>
      </div>
    </div>
  {:else if session}
    <!-- Session Header -->
    <div class="bg-white border-b border-gray-200 px-6 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <Button
            variant="ghost"
            size="sm"
            onclick={() => window.location.href = '/'}
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
            </svg>
          </Button>

          <div>
            <h1 class="text-xl font-bold text-gray-900">{session.title}</h1>
            <p class="text-sm text-gray-600">
              Session ID: {session.id.slice(0, 8)}...
            </p>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <!-- Status Badge -->
          <div class="flex items-center gap-2 px-3 py-1.5 bg-gray-100 rounded-full">
            <div class="w-2 h-2 {getStatusColor(session.status)} rounded-full {session.status === 'active' ? 'animate-pulse' : ''}"></div>
            <span class="text-sm text-gray-700 capitalize">{session.status}</span>
          </div>

          <!-- Control Buttons -->
          {#if session.status === 'active'}
            <Button variant="secondary" size="sm" onclick={handlePause}>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
              </svg>
              Pause
            </Button>
          {:else if session.status === 'paused'}
            <Button variant="primary" size="sm" onclick={handleResume}>
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-1" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
              </svg>
              Resume
            </Button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Chat Interface -->
    <div class="flex-1 overflow-hidden">
      <ChatInterface sessionId={sessionId || ''} />
    </div>
  {/if}
</div>
