<script lang="ts">
  import { Button, Card, Input } from "$lib/components/ui";
  import { config, loadConfigStore, saveConfigStore } from "$lib/stores/config";
  import {
    providerStatus,
    loadCredentials,
    startOAuthFlow,
    deleteCredential,
    authLoading,
    authError,
    clearAuthError,
  } from "$lib/stores/credentials";
  import OAuthModal from "$lib/components/auth/OAuthModal.svelte";
  import type { ChamberConfig } from "$lib/types/config";
  import { onMount } from "svelte";

  let localConfig = $state<ChamberConfig | null>(null);
  let isSaving = $state(false);
  let saveMessage = $state<{ type: "success" | "error"; text: string } | null>(
    null,
  );

  // API key input states
  let apiKeyInputs = $state<Record<string, string>>({
    anthropic: "",
    gemini: "",
    xai: "",
  });
  let showApiKeyInput = $state<Record<string, boolean>>({
    anthropic: false,
    gemini: false,
    xai: false,
  });

  onMount(async () => {
    await loadConfigStore();
    // Clone the config for local editing
    localConfig = $config ? JSON.parse(JSON.stringify($config)) : null;
    // Load credentials
    await loadCredentials();
  });

  async function handleSave() {
    if (!localConfig) return;
    try {
      isSaving = true;
      saveMessage = null;
      await saveConfigStore(localConfig);
      saveMessage = { type: "success", text: "Settings saved successfully!" };
      setTimeout(() => (saveMessage = null), 3000);
    } catch (error) {
      console.error("Failed to save settings:", error);
      saveMessage = {
        type: "error",
        text:
          error instanceof Error ? error.message : "Failed to save settings",
      };
    } finally {
      isSaving = false;
    }
  }

  async function handleReset() {
    if (
      confirm(
        "Are you sure you want to reset to default settings? This cannot be undone.",
      )
    ) {
      localConfig = $config ? JSON.parse(JSON.stringify($config)) : null;
      saveMessage = {
        type: "success",
        text: "Settings reset to current saved values",
      };
      setTimeout(() => (saveMessage = null), 3000);
    }
  }

  // Authentication handlers
  async function handleConnectOAuth(provider: string) {
    clearAuthError();
    await startOAuthFlow(provider);
  }

  function handleShowApiKeyInput(provider: string) {
    showApiKeyInput[provider] = true;
  }

  async function handleSaveApiKey(provider: string) {
    const key = apiKeyInputs[provider].trim();
    if (!key) return;

    try {
      const { createApiKeyCredential, saveCredential: saveCred } = await import("$lib/stores/credentials");
      const credential = createApiKeyCredential(provider, key);
      await saveCred(credential);
      apiKeyInputs[provider] = "";
      showApiKeyInput[provider] = false;
      saveMessage = { type: "success", text: "API key saved successfully!" };
      setTimeout(() => (saveMessage = null), 3000);
    } catch (error) {
      saveMessage = {
        type: "error",
        text: error instanceof Error ? error.message : "Failed to save API key",
      };
    }
  }

  function handleCancelApiKeyInput(provider: string) {
    apiKeyInputs[provider] = "";
    showApiKeyInput[provider] = false;
  }

  async function handleDisconnect(provider: string) {
    if (confirm(`Are you sure you want to disconnect ${provider}?`)) {
      try {
        await deleteCredential(provider);
        saveMessage = { type: "success", text: "Disconnected successfully!" };
        setTimeout(() => (saveMessage = null), 3000);
      } catch (error) {
        saveMessage = {
          type: "error",
          text: error instanceof Error ? error.message : "Failed to disconnect",
        };
      }
    }
  }

  function getProviderDisplayName(provider: string): string {
    const names: Record<string, string> = {
      anthropic: "Anthropic Claude",
      gemini: "Google Gemini",
      xai: "xAI Grok",
    };
    return names[provider] || provider;
  }
</script>

<div class="p-8 max-w-4xl mx-auto">
  <!-- Header -->
  <div class="mb-8">
    <h1 class="text-4xl font-bold text-gray-900 dark:text-slate-100 mb-2">Settings</h1>
    <p class="text-gray-600 dark:text-slate-400">Configure your chamber models and workspace</p>
  </div>

  {#if !localConfig}
    <div class="flex items-center justify-center h-40">
      <div
        class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
      ></div>
    </div>
  {:else}
    <!-- Save Message -->
    {#if saveMessage}
      <div
        class="mb-6 p-4 rounded-lg {saveMessage.type === 'success'
          ? 'bg-green-50 dark:bg-green-900/20 text-green-800 dark:text-green-400 border border-green-200 dark:border-green-800'
          : 'bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-400 border border-red-200 dark:border-red-800'}"
      >
        <div class="flex items-center gap-2">
          {#if saveMessage.type === "success"}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                clip-rule="evenodd"
              />
            </svg>
          {:else}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                clip-rule="evenodd"
              />
            </svg>
          {/if}
          <span>{saveMessage.text}</span>
        </div>
      </div>
    {/if}

    <!-- Orchestrator Section -->
    <Card class="p-6 mb-6">
      <h2 class="text-xl font-bold text-gray-900 dark:text-slate-100 mb-4">Orchestrator Model</h2>
      <p class="text-sm text-gray-600 dark:text-slate-400 mb-4">
        The orchestrator coordinates the multi-model chamber and synthesizes
        responses.
      </p>

      <div class="space-y-4">
        <div>
          <label for="orchestrator-provider" class="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2"
            >Provider</label
          >
          <Input
            id="orchestrator-provider"
            bind:value={localConfig.orchestrator.provider}
            placeholder="e.g., anthropic, google, ollama"
          />
        </div>
        <div>
          <label for="orchestrator-model" class="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2"
            >Model</label
          >
          <Input
            id="orchestrator-model"
            bind:value={localConfig.orchestrator.model}
            placeholder="e.g., claude-3-5-sonnet-20241022"
          />
        </div>
      </div>
    </Card>

    <!-- Reasoning Models Section -->
    <Card class="p-6 mb-6">
      <h2 class="text-xl font-bold text-gray-900 dark:text-slate-100 mb-4">Reasoning Models</h2>
      <p class="text-sm text-gray-600 dark:text-slate-400 mb-4">
        Multiple models that work together to reason about your requests.
      </p>

      <div class="space-y-6">
        {#each localConfig.reasoning_models as model, i}
          <div class="border-l-4 border-blue-500 pl-4">
            <div class="flex items-center justify-between mb-3">
              <h3 class="font-semibold text-gray-900 dark:text-slate-100">Model {i + 1}</h3>
              <label for="model-enabled-{i}" class="flex items-center gap-2 cursor-pointer">
                <input
                  id="model-enabled-{i}"
                  type="checkbox"
                  bind:checked={model.enabled}
                  class="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
                />
                <span class="text-sm text-gray-700 dark:text-slate-300">Enabled</span>
              </label>
            </div>
            <div class="space-y-3">
              <div>
                <label for="reasoning-provider-{i}" class="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-1"
                  >Provider</label
                >
                <Input id="reasoning-provider-{i}" bind:value={model.provider} />
              </div>
              <div>
                <label for="reasoning-model-{i}" class="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-1"
                  >Model</label
                >
                <Input id="reasoning-model-{i}" bind:value={model.model} />
              </div>
            </div>
          </div>
        {/each}
      </div>
    </Card>

    <!-- Workspace Section -->
    <Card class="p-6 mb-6">
      <h2 class="text-xl font-bold text-gray-900 dark:text-slate-100 mb-4">Workspace</h2>
      <p class="text-sm text-gray-600 dark:text-slate-400 mb-4">
        Configure where your session files and history are stored.
      </p>

      <div class="space-y-4">
        <div>
          <label for="workspace-path" class="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2"
            >Workspace Path</label
          >
          <Input
            id="workspace-path"
            bind:value={localConfig.workspace.path}
            placeholder="./workspace"
          />
        </div>
        <div>
          <label for="sessions-dir" class="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2"
            >Sessions Directory</label
          >
          <Input
            id="sessions-dir"
            bind:value={localConfig.workspace.sessions_dir}
            placeholder="sessions"
          />
        </div>
        <div>
          <label for="config-dir" class="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2"
            >Config Directory</label
          >
          <Input
            id="config-dir"
            bind:value={localConfig.workspace.config_dir}
            placeholder=".config"
          />
        </div>
      </div>
    </Card>

    <!-- Tool Approval Section -->
    <Card class="p-6 mb-6">
      <h2 class="text-xl font-bold text-gray-900 dark:text-slate-100 mb-4">Tool Approval</h2>
      <p class="text-sm text-gray-600 dark:text-slate-400 mb-4">
        Configure which tools require human approval before execution.
      </p>

      <label for="tools-approval" class="flex items-center gap-2 cursor-pointer">
        <input
          id="tools-approval"
          type="checkbox"
          bind:checked={localConfig.tools.approval_required}
          class="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
        />
        <span class="text-sm text-gray-700 dark:text-slate-300">Require approval for all tools</span
        >
      </label>
    </Card>

    <!-- Authentication Section -->
    <Card class="p-6 mb-6">
      <h2 class="text-xl font-bold text-gray-900 mb-4">Authentication</h2>
      <p class="text-sm text-gray-600 mb-4">
        Connect your LLM provider accounts. Credentials are stored securely in your system keychain.
      </p>

      {#if $authError}
        <div class="mb-4 p-4 bg-red-50 text-red-800 border border-red-200 rounded-lg">
          {$authError}
        </div>
      {/if}

      <div class="space-y-4">
        <!-- Anthropic -->
        {@const anthropicStatus = $providerStatus.anthropic}
        <div class="border border-gray-200 rounded-lg p-4">
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 bg-orange-500 rounded-lg flex items-center justify-center">
                <span class="text-white font-bold text-lg">C</span>
              </div>
              <div>
                <h3 class="font-semibold text-gray-900">Anthropic Claude</h3>
                {#if anthropicStatus?.has_credential}
                  <p class="text-sm text-green-600">
                    Connected via {anthropicStatus.auth_type === 'oauth_token' ? 'OAuth' : 'API Key'}
                  </p>
                {:else}
                  <p class="text-sm text-gray-500">Not connected</p>
                {/if}
              </div>
            </div>
            <div class="flex gap-2">
              {#if anthropicStatus?.has_credential}
                <Button variant="secondary" onclick={() => handleDisconnect('anthropic')}>
                  Disconnect
                </Button>
              {:else}
                <Button variant="primary" onclick={() => handleConnectOAuth('anthropic')} disabled={$authLoading}>
                  Connect with OAuth
                </Button>
                {#if !showApiKeyInput.anthropic}
                  <Button variant="secondary" onclick={() => handleShowApiKeyInput('anthropic')} disabled={$authLoading}>
                    Use API Key
                  </Button>
                {/if}
              {/if}
            </div>
          </div>
          {#if !anthropicStatus?.has_credential && showApiKeyInput.anthropic}
            <div class="mt-3 pt-3 border-t border-gray-200">
              <div class="flex gap-2">
                <Input
                  type="password"
                  placeholder="sk-ant-api03-..."
                  bind:value={apiKeyInputs.anthropic}
                  class="flex-1"
                />
                <Button onclick={() => handleSaveApiKey('anthropic')} disabled={$authLoading}>
                  Save
                </Button>
                <Button variant="secondary" onclick={() => handleCancelApiKeyInput('anthropic')}>
                  Cancel
                </Button>
              </div>
            </div>
          {/if}
        </div>

        <!-- Gemini -->
        {@const geminiStatus = $providerStatus.gemini}
        <div class="border border-gray-200 rounded-lg p-4">
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 bg-blue-500 rounded-lg flex items-center justify-center">
                <span class="text-white font-bold text-lg">G</span>
              </div>
              <div>
                <h3 class="font-semibold text-gray-900">Google Gemini</h3>
                {#if geminiStatus?.has_credential}
                  <p class="text-sm text-green-600">
                    Connected via {geminiStatus.auth_type === 'oauth_token' ? 'OAuth' : 'API Key'}
                  </p>
                {:else}
                  <p class="text-sm text-gray-500">Not connected</p>
                {/if}
              </div>
            </div>
            <div class="flex gap-2">
              {#if geminiStatus?.has_credential}
                <Button variant="secondary" onclick={() => handleDisconnect('gemini')}>
                  Disconnect
                </Button>
              {:else}
                <Button variant="primary" onclick={() => handleConnectOAuth('gemini')} disabled={$authLoading}>
                  Connect with OAuth
                </Button>
                {#if !showApiKeyInput.gemini}
                  <Button variant="secondary" onclick={() => handleShowApiKeyInput('gemini')} disabled={$authLoading}>
                    Use API Key
                  </Button>
                {/if}
              </div>
            </div>
          </div>
          {#if !geminiStatus?.has_credential && showApiKeyInput.gemini}
            <div class="mt-3 pt-3 border-t border-gray-200">
              <div class="flex gap-2">
                <Input
                  type="password"
                  placeholder="AIzaSy..."
                  bind:value={apiKeyInputs.gemini}
                  class="flex-1"
                />
                <Button onclick={() => handleSaveApiKey('gemini')} disabled={$authLoading}>
                  Save
                </Button>
                <Button variant="secondary" onclick={() => handleCancelApiKeyInput('gemini')}>
                  Cancel
                </Button>
              </div>
            </div>
          {/if}
        </div>

        <!-- xAI -->
        {@const xaiStatus = $providerStatus.xai}
        <div class="border border-gray-200 rounded-lg p-4">
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 bg-gray-800 rounded-lg flex items-center justify-center">
                <span class="text-white font-bold text-lg">X</span>
              </div>
              <div>
                <h3 class="font-semibold text-gray-900">xAI Grok</h3>
                {#if xaiStatus?.has_credential}
                  <p class="text-sm text-green-600">Connected via API Key</p>
                {:else}
                  <p class="text-sm text-gray-500">Not connected</p>
                {/if}
              </div>
            </div>
            <div class="flex gap-2">
              {#if xaiStatus?.has_credential}
                <Button variant="secondary" onclick={() => handleDisconnect('xai')}>
                  Disconnect
                </Button>
              {:else}
                {#if !showApiKeyInput.xai}
                  <Button variant="secondary" onclick={() => handleShowApiKeyInput('xai')} disabled={$authLoading}>
                    Add API Key
                  </Button>
                {/if}
              {/if}
            </div>
          </div>
          {#if !xaiStatus?.has_credential && showApiKeyInput.xai}
            <div class="mt-3 pt-3 border-t border-gray-200">
              <div class="flex gap-2">
                <Input
                  type="password"
                  placeholder="xai-..."
                  bind:value={apiKeyInputs.xai}
                  class="flex-1"
                />
                <Button onclick={() => handleSaveApiKey('xai')} disabled={$authLoading}>
                  Save
                </Button>
                <Button variant="secondary" onclick={() => handleCancelApiKeyInput('xai')}>
                  Cancel
                </Button>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </Card>

    <!-- OAuth Modal -->
    <OAuthModal />

    <!-- Actions -->
    <div class="flex gap-3">
      <Button
        variant="primary"
        onclick={handleSave}
        disabled={isSaving}
        class="flex-1"
      >
        {#if isSaving}
          <svg
            class="animate-spin h-4 w-4 mr-2"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              class="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="4"
            ></circle>
            <path
              class="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            ></path>
          </svg>
        {/if}
        Save Settings
      </Button>
      <Button variant="secondary" onclick={handleReset}
        >Reset to Defaults</Button
      >
    </div>
  {/if}
</div>
