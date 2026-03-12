<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    children,
    class: className = '',
    ...props
  }: {
    children: Snippet;
    class?: string;
    [key: string]: any;
  } = $props();
</script>

<div
  class="bg-white dark:bg-slate-800 rounded-lg border border-gray-200 dark:border-slate-700 shadow-sm {className}"
  {...props}
>
  {@render children()}
</div>
