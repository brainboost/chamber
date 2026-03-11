<script lang="ts">
  import { Button } from '$lib/components/ui';
  import { goto } from '$app/navigation';

  function handleSettingsClick() {
    goto('/settings');
  }
</script>

<header class="bg-white border-b border-gray-200 sticky top-0 z-30">
  <div class="flex items-center justify-between px-6 py-4">
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2">
        <!-- Chamber Icon -->
        <div class="w-8 h-8 bg-gradient-to-br from-blue-600 to-purple-600 rounded-lg flex items-center justify-center">
          <span class="text-white font-bold text-lg">C</span>
        </div>
        <h1 class="text-2xl font-bold text-gray-900">Chamber</h1>
      </div>
      <span class="text-sm text-gray-500 hidden sm:inline">
        Multi-Model AI Collaboration
      </span>
    </div>

    <div class="flex items-center gap-3">
      <!-- Status Indicator -->
      <div class="flex items-center gap-2 px-3 py-1.5 bg-gray-100 rounded-full">
        <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
        <span class="text-sm text-gray-700">Ready</span>
      </div>

      <!-- Settings Button -->
      <Button variant="ghost" size="sm" onclick={handleSettingsClick}>
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd" />
        </svg>
      </Button>
    </div>
  </div>
</header>
