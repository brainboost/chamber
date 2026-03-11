<script lang="ts">
  let {
    value = $bindable(''),
    placeholder = '',
    disabled = false,
    rows = 3,
    class: className = '',
    ...props
  }: {
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    rows?: number;
    class?: string;
    [key: string]: any;
  } = $props();

  const baseClasses = 'w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed resize-none';
</script>

<textarea
  bind:value
  {placeholder}
  {disabled}
  {rows}
  class="{baseClasses} {className}"
  {...props}
></textarea>
