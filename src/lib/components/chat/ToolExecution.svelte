<script lang="ts">
  import type { ToolExecution as ToolExecutionType } from '$lib/types/message';
  import { Card } from '$lib/components/ui';

  let {
    execution,
  }: {
    execution: ToolExecutionType;
  } = $props();

  let isExpanded = $state(false);

  const toolIcons: Record<string, string> = {
    web_search: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z',
    calculator: 'M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z',
    read_file: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
    write_file: 'M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z',
  };

  const toolIcon = toolIcons[execution.tool_name] || 'M13 10V3L4 14h7v7l9-11h-7z';
</script>

<div class="flex justify-start">
  <Card class="max-w-[85%] bg-gradient-to-br from-green-50 to-emerald-50 border-green-200">
    <div class="p-4">
      <!-- Header -->
      <button
        class="w-full flex items-center gap-3 mb-2"
        onclick={() => isExpanded = !isExpanded}
      >
        <div class="w-8 h-8 bg-gradient-to-br from-green-500 to-emerald-500 rounded-full flex items-center justify-center">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d={toolIcon} />
          </svg>
        </div>
        <div class="flex-1 text-left">
          <div class="flex items-center gap-2">
            <h3 class="font-semibold text-gray-900">Tool: {execution.tool_name}</h3>
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
          <span class="text-xs text-green-700">Executed successfully</span>
        </div>
      </button>

      {#if isExpanded}
        <!-- Parameters -->
        <div class="mt-3 mb-3">
          <h4 class="text-xs font-semibold text-gray-600 mb-2">Parameters:</h4>
          <div class="bg-white/50 rounded-md p-3 font-mono text-xs overflow-x-auto">
            <pre>{JSON.stringify(execution.parameters, null, 2)}</pre>
          </div>
        </div>

        <!-- Result -->
        <div>
          <h4 class="text-xs font-semibold text-gray-600 mb-2">Result:</h4>
          <div class="bg-white/50 rounded-md p-3 font-mono text-xs overflow-x-auto">
            <pre>{JSON.stringify(execution.result, null, 2)}</pre>
          </div>
        </div>
      {/if}
    </div>
  </Card>
</div>
