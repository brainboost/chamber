<script lang="ts">
  import type { Message } from '$lib/types/message';
  import ReasoningStep from './ReasoningStep.svelte';
  import ToolExecution from './ToolExecution.svelte';
  import ToolApprovalRequest from './ToolApprovalRequest.svelte';

  let {
    message,
  }: {
    message: Message;
  } = $props();
</script>

{#if message.type === 'UserMessage'}
  <div class="flex justify-end">
    <div class="bg-blue-600 text-white rounded-lg px-4 py-3 max-w-[70%]">
      <p class="whitespace-pre-wrap">{message.content}</p>
    </div>
  </div>
{:else if message.type === 'AssistantMessage'}
  <div class="flex justify-start">
    <div class="bg-white dark:bg-slate-800 border border-gray-200 dark:border-slate-700 rounded-lg px-4 py-3 max-w-[70%]">
      <div class="flex items-center gap-2 mb-2">
        <div class="w-6 h-6 bg-gradient-to-br from-purple-500 to-blue-500 rounded-full"></div>
        <span class="text-xs font-medium text-gray-600 dark:text-slate-400">{message.model}</span>
      </div>
      <p class="text-gray-900 dark:text-slate-100 whitespace-pre-wrap">{message.content}</p>
    </div>
  </div>
{:else if message.type === 'ReasoningStep'}
  <ReasoningStep reasoning={message} />
{:else if message.type === 'ToolApprovalRequest'}
  <ToolApprovalRequest request={message} />
{:else if message.type === 'ToolExecution'}
  <ToolExecution execution={message} />
{:else if message.type === 'SystemMessage'}
  <div class="flex justify-center">
    <div class="bg-gray-100 dark:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg px-4 py-2 text-sm max-w-[70%]">
      <p>{message.content}</p>
    </div>
  </div>
{:else if message.type === 'Error'}
  <div class="flex justify-center">
    <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-800 dark:text-red-400 rounded-lg px-4 py-3 max-w-[70%]">
      <div class="flex items-center gap-2">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
        </svg>
        <p class="font-medium">Error</p>
      </div>
      <p class="mt-1 text-sm">{message.message}</p>
    </div>
  </div>
{/if}
