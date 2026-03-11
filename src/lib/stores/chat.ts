import { writable, type Writable } from 'svelte/store';
import type { Message } from '$lib/types/message';
import { indexedDB } from '$lib/services/indexeddb';

export const messages: Writable<Message[]> = writable([]);

export function addMessage(message: Message): void {
  messages.update((msgs) => [...msgs, message]);
}

export function clearMessages(): void {
  messages.set([]);
}

export function setMessages(newMessages: Message[]): void {
  messages.set(newMessages);
}

// Extended API for UI components
export const chatStore = {
  async getMessages(sessionId: string): Promise<Message[]> {
    try {
      const msgs = await indexedDB.getMessages(sessionId);
      setMessages(msgs);
      return msgs;
    } catch (error) {
      console.error('Failed to get messages:', error);
      return [];
    }
  },

  async addMessage(sessionId: string, message: Message): Promise<void> {
    try {
      // Add to store
      addMessage(message);

      // Save to IndexedDB
      await indexedDB.saveMessage(sessionId, message);
    } catch (error) {
      console.error('Failed to add message:', error);
      throw error;
    }
  },
};
