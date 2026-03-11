<script lang="ts">
  import type { ReasoningStep as ReasoningStepType } from '$lib/types/message';
  import { Card } from '$lib/components/ui';

  let {
    reasoning,
  }: {
    reasoning: ReasoningStepType;
  } = $props();

  const stepColors: Record<string, string> = {
    orchestrator_plan: 'from-purple-500 to-blue-500',
    parallel_reasoning: 'from-blue-500 to-cyan-500',
    orchestrator_synthesis: 'from-purple-500 to-pink-500',
    tool_approval: 'from-orange-500 to-red-500',
    finalize: 'from-green-500 to-emerald-500',
  };

  const stepLabels: Record<string, string> = {
    orchestrator_plan: 'Planning',
    parallel_reasoning: 'Reasoning',
    orchestrator_synthesis: 'Synthesis',
    tool_approval: 'Tool Approval',
    finalize: 'Finalizing',
  };

  const gradientClass = stepColors[reasoning.step] || 'from-gray-500 to-gray-600';
  const stepLabel = stepLabels[reasoning.step] || reasoning.step;
</script>

<div class="flex justify-start">
  <Card class="max-w-[85%] border-l-4 border-l-blue-500">
    <div class="p-4">
      <!-- Header -->
      <div class="flex items-center gap-3 mb-3">
        <div class="w-8 h-8 bg-gradient-to-br {gradientClass} rounded-full flex items-center justify-center">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-white" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clip-rule="evenodd" />
          </svg>
        </div>
        <div class="flex-1">
          <div class="flex items-center gap-2">
            <h3 class="font-semibold text-gray-900">{stepLabel}</h3>
            <span class="text-xs px-2 py-0.5 bg-blue-100 text-blue-700 rounded-full">
              {reasoning.model}
            </span>
          </div>
        </div>
      </div>

      <!-- Content -->
      <div class="text-gray-700 text-sm leading-relaxed whitespace-pre-wrap">
        {reasoning.content}
      </div>
    </div>
  </Card>
</div>
