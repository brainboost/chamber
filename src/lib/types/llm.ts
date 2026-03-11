export type LLMProvider = 'anthropic' | 'gemini' | 'ollama' | 'xai';

export interface LLMModel {
  provider: LLMProvider;
  name: string;
  display_name: string;
  description?: string;
}

export const AVAILABLE_MODELS: Record<LLMProvider, LLMModel[]> = {
  anthropic: [
    {
      provider: 'anthropic',
      name: 'claude-opus-4-5',
      display_name: 'Claude Opus 4.5',
      description: 'Most capable model',
    },
    {
      provider: 'anthropic',
      name: 'claude-sonnet-4-5',
      display_name: 'Claude Sonnet 4.5',
      description: 'Balanced performance',
    },
  ],
  gemini: [
    {
      provider: 'gemini',
      name: 'gemini-2.0-flash-thinking-exp',
      display_name: 'Gemini 2.0 Flash Thinking',
      description: 'Experimental thinking model',
    },
  ],
  ollama: [
    {
      provider: 'ollama',
      name: 'llama2',
      display_name: 'Llama 2',
      description: 'Local model',
    },
  ],
  xai: [
    {
      provider: 'xai',
      name: 'grok-beta',
      display_name: 'Grok Beta',
      description: 'x.ai model',
    },
  ],
};
