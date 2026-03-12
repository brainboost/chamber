<script lang="ts">
  import { Button, Textarea } from '$lib/components/ui';
  import { chatStore } from '$lib/stores/chat';
  import { sessionStore } from '$lib/stores/session';
  import type { Message } from '$lib/types/message';
  import MessageBubble from './MessageBubble.svelte';

  let {
    sessionId,
  }: {
    sessionId: string;
  } = $props();

  let messages = $state<Message[]>([]);
  let inputValue = $state('');
  let isLoading = $state(false);
  let messagesContainer: HTMLDivElement;

  // Load messages for this session
  $effect(() => {
    loadMessages();
  });

  async function loadMessages() {
    try {
      messages = await chatStore.getMessages(sessionId);
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  }

  async function handleSend() {
    if (!inputValue.trim() || isLoading) return;

    const userMessage = inputValue.trim();
    inputValue = '';
    isLoading = true;

    try {
      // Add user message to UI immediately
      const newMessage: Message = {
        type: 'UserMessage',
        content: userMessage,
      };
      messages = [...messages, newMessage];

      // Save to store and send to backend
      await chatStore.addMessage(sessionId, newMessage);
      await sessionStore.sendMessage(sessionId, userMessage);

      // Scroll to bottom
      setTimeout(() => {
        messagesContainer?.scrollTo({
          top: messagesContainer.scrollHeight,
          behavior: 'smooth',
        });
      }, 100);
    } catch (error) {
      console.error('Failed to send message:', error);
      const errorMessage: Message = {
        type: 'Error',
        message: error instanceof Error ? error.message : String(error),
      };
      messages = [...messages, errorMessage];
    } finally {
      isLoading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }
</script>

<div class="flex flex-col h-full">
  <!-- Messages Area -->
  <div
    bind:this={messagesContainer}
    class="flex-1 overflow-y-auto px-6 py-6 space-y-4"
  >
    {#if messages.length === 0}
      <div class="flex items-center justify-center h-full">
        <div class="text-center text-gray-500">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mx-auto mb-4 text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
          <p class="text-lg font-medium">No messages yet</p>
          <p class="text-sm mt-1">Start a conversation with the chamber</p>
        </div>
      </div>
    {:else}
      {#each messages as message, i (i)}
        <MessageBubble {message} />
      {/each}

      {#if isLoading}
        <div class="flex items-center gap-2 text-gray-500">
          <div class="flex gap-1">
            <div class="w-2 h-2 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 0ms"></div>
            <div class="w-2 h-2 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 150ms"></div>
            <div class="w-2 h-2 bg-blue-500 rounded-full animate-bounce" style="animation-delay: 300ms"></div>
          </div>
          <span class="text-sm">Chamber is thinking...</span>
        </div>
      {/if}
    {/if}
  </div>

  <!-- Input Area -->
  <div class="border-t border-gray-200 bg-white px-6 py-4">
    <div class="flex gap-3 items-end">
      <div class="flex-1">
        <Textarea
          bind:value={inputValue}
          placeholder="Type your message... (Enter to send, Shift+Enter for new line)"
          rows={3}
          disabled={isLoading}
          onkeydown={handleKeydown}
          class="resize-none"
        />
      </div>
      <Button
        variant="primary"
        onclick={handleSend}
        disabled={!inputValue.trim() || isLoading}
        class="h-[84px]"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
          <path d="M10.894 2.553a1 1 0 00-1.788 0l-7 14a1 1 0 001.169 1.409l5-1.429A1 1 0 009 15.571V11a1 1 0 112 0v4.571a1 1 0 00.725.962l5 1.428a1 1 0 001.17-1.408l-7-14z" />
        </svg>
      </Button>
    </div>
  </div>
</div>
