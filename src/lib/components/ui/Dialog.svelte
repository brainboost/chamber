<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    open = $bindable(false),
    title,
    children,
    class: className = '',
  }: {
    open?: boolean;
    title?: string;
    children: Snippet;
    class?: string;
  } = $props();

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      open = false;
    }
  }
</script>

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/50 z-40 transition-opacity"
    onclick={handleBackdropClick}
    role="presentation"
  >
    <!-- Dialog -->
    <div class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-lg">
      <div class="bg-white rounded-lg shadow-xl max-h-[90vh] flex flex-col {className}">
        {#if title}
          <div class="px-6 py-4 border-b border-gray-200">
            <h2 class="text-xl font-semibold text-gray-900">{title}</h2>
          </div>
        {/if}

        <div class="px-6 py-4 overflow-y-auto">
          {@render children()}
        </div>
      </div>
    </div>
  </div>
{/if}
