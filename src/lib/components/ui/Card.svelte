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
  class="bg-white rounded-lg border border-gray-200 shadow-sm {className}"
  {...props}
>
  {@render children()}
</div>
