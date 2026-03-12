<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button, Card } from "$lib/components/ui";
  import { config, loadConfigStore } from "$lib/stores/config";
  import { sessionStore } from "$lib/stores/session";
  import type { Session } from "$lib/types/session";
  import { onMount } from "svelte";

  let recentSessions = $state<Session[]>([]);
  let isLoading = $state(true);

  onMount(async () => {
    try {
      await loadConfigStore();
      await loadRecentSessions();
    } catch (error) {
      console.error("Failed to load sessions:", error);
    } finally {
      isLoading = false;
    }
  });

  async function loadRecentSessions() {
    const sessions = await sessionStore.listSessions();
    // Get the 5 most recent sessions
    recentSessions = sessions.slice(0, 5);
  }

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
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
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
  <div class="mb-8">
    <h1 class="text-4xl font-bold text-gray-900 dark:text-slate-100 mb-2">Dashboard</h1>
    <p class="text-gray-600 dark:text-slate-400">
      Multi-Model AI Application with Human-in-the-Loop Workflows
    </p>
  </div>

  <!-- Quick Actions -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
    <Card
      class="p-6 hover:shadow-md transition-shadow cursor-pointer"
      onclick={handleNewSession}
    >
      <div class="flex items-start gap-4">
        <div
          class="w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-500 rounded-lg flex items-center justify-center"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6 text-white"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 4v16m8-8H4"
            />
          </svg>
        </div>
        <div class="flex-1">
          <h3 class="font-semibold text-gray-900 dark:text-slate-100 mb-1">New Session</h3>
          <p class="text-sm text-gray-600 dark:text-slate-400">Start a new multi-model chamber</p>
        </div>
      </div>
    </Card>

    <Card
      class="p-6 hover:shadow-md transition-shadow cursor-pointer"
      onclick={() => goto("/sessions")}
    >
      <div class="flex items-start gap-4">
        <div
          class="w-12 h-12 bg-gradient-to-br from-purple-500 to-pink-500 rounded-lg flex items-center justify-center"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6 text-white"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z"
            />
          </svg>
        </div>
        <div class="flex-1">
          <h3 class="font-semibold text-gray-900 dark:text-slate-100 mb-1">Browse Sessions</h3>
          <p class="text-sm text-gray-600 dark:text-slate-400">View all chamber sessions</p>
        </div>
      </div>
    </Card>

    <Card
      class="p-6 hover:shadow-md transition-shadow cursor-pointer"
      onclick={() => goto("/settings")}
    >
      <div class="flex items-start gap-4">
        <div
          class="w-12 h-12 bg-gradient-to-br from-orange-500 to-red-500 rounded-lg flex items-center justify-center"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6 text-white"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
        </div>
        <div class="flex-1">
          <h3 class="font-semibold text-gray-900 dark:text-slate-100 mb-1">Settings</h3>
          <p class="text-sm text-gray-600 dark:text-slate-400">Configure models and tools</p>
        </div>
      </div>
    </Card>
  </div>

  <!-- Recent Sessions -->
  <div>
    <h2 class="text-2xl font-bold text-gray-900 dark:text-slate-100 mb-4">Recent Sessions</h2>

    {#if isLoading}
      <div class="flex items-center justify-center h-40">
        <div
          class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
        ></div>
      </div>
    {:else if recentSessions.length === 0}
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
          <p class="text-lg font-medium mb-2 dark:text-slate-200">No sessions yet</p>
          <p class="text-sm mb-4 dark:text-slate-400">
            Create your first chamber session to get started
          </p>
          <Button variant="primary" onclick={handleNewSession}>
            Create Session
          </Button>
        </div>
      </Card>
    {:else}
      <div class="space-y-3">
        {#each recentSessions as session}
          <Card
            class="p-4 hover:shadow-md transition-shadow cursor-pointer"
            onclick={() => (window.location.href = `/session/${session.id}`)}
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-4">
                <div
                  class="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-500 rounded-lg flex items-center justify-center"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5 text-white"
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
                <div>
                  <h3 class="font-semibold text-gray-900 dark:text-slate-100">{session.title}</h3>
                  <p class="text-sm text-gray-600 dark:text-slate-400">
                    {formatDate(session.updated_at)}
                  </p>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <div class="flex items-center gap-2">
                  <div
                    class="w-2 h-2 {getStatusColor(
                      session.status,
                    )} rounded-full"
                  ></div>
                  <span class="text-sm text-gray-600 dark:text-slate-400 capitalize"
                    >{session.status}</span
                  >
                </div>
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="h-5 w-5 text-gray-400 dark:text-slate-500"
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
</div>
