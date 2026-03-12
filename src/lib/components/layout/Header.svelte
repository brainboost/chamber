<script lang="ts">
  import { Button } from '$lib/components/ui';
  import { goto } from '$app/navigation';
  import { theme, setTheme } from '$lib/stores/theme';

  function handleSettingsClick() {
    goto('/settings');
  }

  function cycleTheme() {
    if ($theme === 'system') {
      setTheme('light');
    } else if ($theme === 'light') {
      setTheme('dark');
    } else {
      setTheme('system');
    }
  }
</script>

<header class="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 sticky top-0 z-30">
  <div class="flex items-center justify-between px-6 py-4">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2">
        <!-- Chamber Icon -->
        <div class="w-8 h-8 bg-gradient-to-br from-blue-600 to-purple-600 rounded-lg flex items-center justify-center">
          <span class="text-white font-bold text-lg">C</span>
        </div>
        <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Chamber</h1>
      </div>
      <span class="text-sm text-gray-500 dark:text-gray-400 hidden sm:inline">
        Multi-Model AI Collaboration
      </span>
    </div>

    <div class="flex items-center gap-3">
      <!-- Status Indicator -->
      <div class="flex items-center gap-2 px-3 py-1.5 bg-gray-100 dark:bg-gray-800 rounded-full">
        <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
        <span class="text-sm text-gray-700 dark:text-gray-300">Ready</span>
      </div>

      <!-- Theme Toggle Button -->
      <Button variant="ghost" size="sm" onclick={cycleTheme} title={`Theme: ${$theme}`}>
        {#if $theme === 'light'}
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4.22 4.22a1 1 0 011.415 0l.707.707a1 1 0 01-1.414 1.414l-.708-.707a1 1 0 010-1.414zM16 10a1 1 0 011 1h1a1 1 0 110-2h-1a1 1 0 01-1 1zm-4.22 4.22a1 1 0 010 1.415l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 0zM10 16a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zm-4.22-4.22a1 1 0 010-1.415l-.707-.707a1 1 0 01-1.414 1.414l.707.707a1 1 0 011.414 0zM4 10a1 1 0 011-1h1a1 1 0 110 2H3a1 1 0 01-1-1zm4.22-4.22a1 1 0 01-1.415 0l-.707-.707a1 1 0 011.414-1.414l.708.707a1 1 0 010 1.414zM10 5a5 5 0 100 10 5 5 0 000-10z" clip-rule="evenodd" />
          </svg>
        {:else if $theme === 'dark'}
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
          </svg>
        {:else}
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 2a8 8 0 100 16 8 8 0 000-16zM5.12 14.88a6 6 0 018.76-8.76L5.12 14.88z" clip-rule="evenodd" />
          </svg>
        {/if}
      </Button>

      <!-- Settings Button -->
      <Button variant="ghost" size="sm" onclick={handleSettingsClick}>
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd" />
        </svg>
      </Button>
    </div>
  </div>
</header>
