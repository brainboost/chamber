<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    variant = 'default',
    size = 'default',
    disabled = false,
    onclick,
    children,
    class: className = '',
    ...props
  }: {
    variant?: 'default' | 'primary' | 'secondary' | 'danger' | 'ghost';
    size?: 'sm' | 'default' | 'lg';
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
    class?: string;
    [key: string]: any;
  } = $props();

  const variants = {
    default: 'bg-gray-200 text-gray-900 hover:bg-gray-300',
    primary: 'bg-blue-600 text-white hover:bg-blue-700',
    secondary: 'bg-gray-600 text-white hover:bg-gray-700',
    danger: 'bg-red-600 text-white hover:bg-red-700',
    ghost: 'bg-transparent hover:bg-gray-100 text-gray-900',
  };

  const sizes = {
    sm: 'px-3 py-1.5 text-sm',
    default: 'px-4 py-2',
    lg: 'px-6 py-3 text-lg',
  };

  const baseClasses = 'inline-flex items-center justify-center rounded-md font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none';
  const classes = `${baseClasses} ${variants[variant]} ${sizes[size]} ${className}`;
</script>

<button
  class={classes}
  {disabled}
  {onclick}
  {...props}
>
  {@render children()}
</button>
