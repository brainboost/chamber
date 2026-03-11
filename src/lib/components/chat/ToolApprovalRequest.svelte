<script lang="ts">
  import type { ToolApprovalRequest as ToolApprovalRequestType } from '$lib/types/message';
  import { Card, Button } from '$lib/components/ui';
  import { sessionStore } from '$lib/stores/session';

  let {
    request,
  }: {
    request: ToolApprovalRequestType;
  } = $props();

  let isProcessing = $state(false);
  let isExpanded = $state(true);

  async function handleApprove() {
    isProcessing = true;
    try {
      // Send approval response (will be implemented in session store)
      await sessionStore.approveToolExecution(request.request_id, true);
    } catch (error) {
      console.error('Failed to approve tool:', error);
    } finally {
      isProcessing = false;
    }
  }

  async function handleReject() {
    isProcessing = true;
    try {
      await sessionStore.approveToolExecution(request.request_id, false);
    } catch (error) {
      console.error('Failed to reject tool:', error);
    } finally {
      isProcessing = false;
    }
  }
</script>

<div class="flex justify-start">
  <Card class="max-w-[85%] bg-gradient-to-br from-orange-50 to-yellow-50 border-orange-200 border-2">
    <div class="p-4">
      <!-- Header -->
      <button
        class="w-full flex items-center gap-3 mb-3"
        onclick={() => isExpanded = !isExpanded}
      >
        <div class="w-8 h-8 bg-gradient-to-br from-orange-500 to-red-500 rounded-full flex items-center justify-center animate-pulse">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-white" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="flex-1 text-left">
          <div class="flex items-center gap-2">
            <h3 class="font-semibold text-gray-900">🔧 Tool Approval Required</h3>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-4 w-4 text-gray-500 transition-transform {isExpanded ? 'rotate-180' : ''}"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </div>
          <span class="text-sm text-orange-700 font-medium">{request.tool_name}</span>
        </div>
      </button>

      {#if isExpanded}
        <!-- Reasoning -->
        <div class="mb-4">
          <h4 class="text-xs font-semibold text-gray-600 mb-2">Reasoning:</h4>
          <p class="text-sm text-gray-700 bg-white/50 rounded-md p-3">
            {request.reasoning}
          </p>
        </div>

        <!-- Parameters -->
        <div class="mb-4">
          <h4 class="text-xs font-semibold text-gray-600 mb-2">Parameters:</h4>
          <div class="bg-white/50 rounded-md p-3 font-mono text-xs overflow-x-auto">
            <pre>{JSON.stringify(request.parameters, null, 2)}</pre>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex gap-3">
          <Button
            variant="primary"
            class="flex-1"
            onclick={handleApprove}
            disabled={isProcessing}
          >
            {#if isProcessing}
              <svg class="animate-spin h-4 w-4 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
            {:else}
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-2" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            {/if}
            Approve
          </Button>
          <Button
            variant="danger"
            class="flex-1"
            onclick={handleReject}
            disabled={isProcessing}
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-2" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
            </svg>
            Reject
          </Button>
        </div>
      {/if}
    </div>
  </Card>
</div>
