<script lang="ts">
  import '../app.css';
  import Header from '$lib/components/layout/Header.svelte';
  import Sidebar from '$lib/components/layout/Sidebar.svelte';
</script>

<div class="flex flex-col h-screen bg-gray-50">
  <Header />

  <div class="flex flex-1 overflow-hidden">
    <Sidebar />

    <main class="flex-1 overflow-y-auto">
      <slot />
    </main>
  </div>
</div>
