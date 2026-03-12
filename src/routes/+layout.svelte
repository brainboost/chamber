<script lang="ts">
  import '../app.css';
  import Header from '$lib/components/layout/Header.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
  import { onMount } from 'svelte';
  import { initTheme } from '$lib/stores/theme';
  import { loadConfigStore } from '$lib/stores/config';

  onMount(async () => {
    try {
      await loadConfigStore();
    } catch (err) {
      console.error('Failed to load initial config:', err);
    }
    initTheme();
  });
</script>

<div class="flex flex-col h-screen bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-100">
  <Header />

  <div class="flex flex-1 overflow-hidden">
    <Sidebar />

    <main class="flex-1 overflow-y-auto">
      <slot />
    </main>
  </div>
</div>
