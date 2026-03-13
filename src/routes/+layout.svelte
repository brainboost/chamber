<script lang="ts">
  import '../app.css';
  import Header from '$lib/components/layout/Header.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import { onMount, onDestroy } from 'svelte';
  import { initTheme } from '$lib/stores/theme';
  import { loadConfigStore } from '$lib/stores/config';
  import { pushCredentialsToSidecar } from '$lib/services/auth';
  import { setupOAuthListeners } from '$lib/stores/credentials';
  import { listen } from '@tauri-apps/api/event';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { isSidecarRunning, startSidecar } from '$lib/services/tauri';

  type SidecarStatus = 'unknown' | 'starting' | 'ready' | 'failed';

  let sidecarStatus = $state<SidecarStatus>('unknown');
  let sidecarError = $state<string | null>(null);
  let retrying = $state(false);
  let unlistenSidecar: UnlistenFn | null = null;

  onMount(async () => {
    try {
      await loadConfigStore();
    } catch (err) {
      console.error('Failed to load initial config:', err);
    }
    initTheme();

    // Wire up OAuth event listeners early so callbacks are never missed
    setupOAuthListeners().catch(() => {});

    // Listen for sidecar lifecycle events emitted by the Rust backend
    unlistenSidecar = await listen<{ status: string; error?: string }>(
      'sidecar://status',
      (event) => {
        sidecarStatus = event.payload.status as SidecarStatus;
        sidecarError = event.payload.error ?? null;

        // Push credentials as soon as the sidecar confirms it's ready.
        // This is the reliable moment — the sidecar process is accepting requests.
        if (event.payload.status === 'ready') {
          pushCredentialsToSidecar().catch(() => {});
        }
      }
    );

    // Check if the sidecar was already running before this listener was registered
    // (e.g. app restarted but sidecar was left running). Push credentials in that case too.
    try {
      const running = await isSidecarRunning();
      if (running && sidecarStatus === 'unknown') {
        sidecarStatus = 'ready';
        pushCredentialsToSidecar().catch(() => {});
      }
    } catch {
      // ignore — sidecar status event will arrive shortly
    }
  });

  onDestroy(() => {
    unlistenSidecar?.();
  });

  async function handleRetry() {
    retrying = true;
    sidecarError = null;
    try {
      await startSidecar();
      sidecarStatus = 'ready';
    } catch (e) {
      sidecarStatus = 'failed';
      sidecarError = String(e);
    } finally {
      retrying = false;
    }
  }
</script>

<div class="flex flex-col h-screen bg-gray-50 dark:bg-slate-900 text-gray-900 dark:text-slate-100">
  <Header />

  {#if sidecarStatus === 'starting'}
    <div class="flex items-center gap-2 border-b border-blue-200 bg-blue-50 px-4 py-2 dark:border-blue-800 dark:bg-blue-900/20">
      <div class="h-2 w-2 animate-pulse rounded-full bg-blue-500"></div>
      <span class="text-sm text-blue-700 dark:text-blue-400">Starting AI backend…</span>
    </div>
  {:else if sidecarStatus === 'failed'}
    <div class="flex items-center gap-3 border-b border-red-200 bg-red-50 px-4 py-2 dark:border-red-800 dark:bg-red-900/20">
      <span class="flex-1 text-sm text-red-600 dark:text-red-400">
        AI backend failed to start.{sidecarError ? ` ${sidecarError.split('\n')[0]}` : ''}
      </span>
      <button
        onclick={handleRetry}
        disabled={retrying}
        class="rounded bg-red-100 px-3 py-1 text-xs text-red-700 hover:bg-red-200 disabled:opacity-50 dark:bg-red-800 dark:text-red-300 dark:hover:bg-red-700"
      >
        {retrying ? 'Retrying…' : 'Retry'}
      </button>
    </div>
  {/if}

  <div class="flex flex-1 overflow-hidden">
    <Sidebar />

    <main class="flex-1 overflow-y-auto">
      <slot />
    </main>
  </div>
</div>
