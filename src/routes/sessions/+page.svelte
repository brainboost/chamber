<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button, Card, Input } from "$lib/components/ui";
  import { config, loadConfigStore } from "$lib/stores/config";
  import { sessionStore } from "$lib/stores/session";
  import type { Session } from "$lib/types/session";
  import { onMount } from "svelte";

  let sessions = $state<Session[]>([]);
  let filteredSessions = $state<Session[]>([]);
  let searchQuery = $state("");
  let isLoading = $state(true);

  onMount(async () => {
    await loadConfigStore();
    await loadSessions();
  });

  async function loadSessions() {
    try {
      isLoading = true;
      sessions = await sessionStore.listSessions();
      filteredSessions = sessions;
    } catch (error) {
      console.error("Failed to load sessions:", error);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (searchQuery.trim() === "") {
      filteredSessions = sessions;
    } else {
      const query = searchQuery.toLowerCase();
      filteredSessions = sessions.filter(
        (s) =>
          s.title.toLowerCase().includes(query) ||
          s.id.toLowerCase().includes(query),
      );
    }
  });

  async function handleNewSession() {
    try {
      const currentConfig = $config;
      if (!currentConfig) {
        console.error("Config not loaded");
        return;
      }
      const session = await sessionStore.createSession({
        title: `Session ${new Date().toLocaleString()}`,
        workspace_path: currentConfig.workspace.path,
      });
      goto(`/session/${session.id}`);
    } catch (error) {
      console.error("Failed to create session:", error);
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp).toLocaleString();
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case "active":
        return "bg-green-500";
      case "paused":
        return "bg-yellow-500";
      case "completed":
        return "bg-blue-500";
      case "failed":
        return "bg-red-500";
      default:
        return "bg-gray-500";
    }
  }
</script>

<div class="p-8 max-w-7xl mx-auto">
  <!-- Header -->
  <div class="flex items-center justify-between mb-8">
    <div>
      <h1 class="text-4xl font-bold text-gray-900 mb-2">Sessions</h1>
      <p class="text-gray-600">View and manage your chamber sessions</p>
    </div>
    <Button variant="primary" onclick={handleNewSession}>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-5 w-5 mr-2"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"
          clip-rule="evenodd"
        />
      </svg>
      New Session
    </Button>
  </div>

  <!-- Search -->
  <div class="mb-6">
    <Input
      bind:value={searchQuery}
      type="search"
      placeholder="Search sessions by title or ID..."
      class="max-w-md"
    />
  </div>

  <!-- Sessions List -->
  {#if isLoading}
    <div class="flex items-center justify-center h-40">
      <div
        class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
      ></div>
    </div>
  {:else if filteredSessions.length === 0}
    <Card class="p-12">
      <div class="text-center text-gray-500">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-16 w-16 mx-auto mb-4 text-gray-300"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z"
          />
        </svg>
        <p class="text-lg font-medium mb-2">
          {searchQuery ? "No sessions found" : "No sessions yet"}
        </p>
        <p class="text-sm mb-4">
          {searchQuery
            ? "Try a different search query"
            : "Create your first chamber session to get started"}
        </p>
        {#if !searchQuery}
          <Button variant="primary" onclick={handleNewSession}>
            Create Session
          </Button>
        {/if}
      </div>
    </Card>
  {:else}
    <div class="grid gap-4">
      {#each filteredSessions as session}
        <Card
          class="p-5 hover:shadow-md transition-shadow cursor-pointer"
          onclick={() => goto(`/session/${session.id}`)}
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-4 flex-1">
              <div
                class="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-500 rounded-lg flex items-center justify-center flex-shrink-0"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-6 w-6 text-white"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M18 10c0 3.866-3.582 7-8 7a8.841 8.841 0 01-4.083-.98L2 17l1.338-3.123C2.493 12.767 2 11.434 2 10c0-3.866 3.582-7 8-7s8 3.134 8 7zM7 9H5v2h2V9zm8 0h-2v2h2V9zM9 9h2v2H9V9z"
                    clip-rule="evenodd"
                  />
                </svg>
              </div>
              <div class="flex-1 min-w-0">
                <h3 class="font-semibold text-gray-900 truncate">
                  {session.title}
                </h3>
                <div class="flex items-center gap-4 mt-1">
                  <p class="text-sm text-gray-600">
                    Created: {formatDate(session.created_at)}
                  </p>
                  <p class="text-sm text-gray-600">
                    Updated: {formatDate(session.updated_at)}
                  </p>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <div class="flex items-center gap-2">
                <div
                  class="w-2 h-2 {getStatusColor(session.status)} rounded-full"
                ></div>
                <span class="text-sm text-gray-600 capitalize"
                  >{session.status}</span
                >
              </div>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-5 w-5 text-gray-400"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fill-rule="evenodd"
                  d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                  clip-rule="evenodd"
                />
              </svg>
            </div>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</div>
