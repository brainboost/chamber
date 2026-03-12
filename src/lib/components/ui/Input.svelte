<script lang="ts">
  let {
    value = $bindable(''),
    type = 'text',
    placeholder = '',
    disabled = false,
    class: className = '',
    ...props
  }: {
    value?: string;
    type?: string;
    placeholder?: string;
    disabled?: boolean;
    class?: string;
    [key: string]: any;
  } = $props();

  const baseClasses = 'w-full px-3 py-2 border border-gray-300 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 dark:disabled:bg-slate-800 disabled:cursor-not-allowed placeholder-gray-400 dark:placeholder-slate-500';
</script>

<input
  bind:value
  {type}
  {placeholder}
  {disabled}
  class="{baseClasses} {className}"
  {...props}
/>
