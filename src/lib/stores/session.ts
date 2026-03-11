import { writable, derived, type Writable, type Readable } from 'svelte/store';
import type { Session } from '$lib/types/session';
import * as tauri from '$lib/services/tauri';
import { indexedDB } from '$lib/services/indexeddb';

export const currentSession: Writable<Session | null> = writable(null);
export const sessions: Writable<Session[]> = writable([]);

export const isSessionActive: Readable<boolean> = derived(
  currentSession,
  ($currentSession) => $currentSession?.status === 'active'
);

export const isSessionPaused: Readable<boolean> = derived(
  currentSession,
  ($currentSession) => $currentSession?.status === 'paused'
);

export function setCurrentSession(session: Session): void {
  currentSession.set(session);
}

export function updateSessionStatus(status: Session['status']): void {
  currentSession.update((session) => {
    if (session) {
      return { ...session, status, updated_at: Date.now() };
    }
    return session;
  });
}

export function clearCurrentSession(): void {
  currentSession.set(null);
}

// Extended API for UI components
export const sessionStore = {
  async createSession(params: { title: string; workspace_path: string }): Promise<Session> {
    try {
      // Create session via Tauri
      const session = await tauri.createSession(params.title);

      // Save to IndexedDB
      await indexedDB.saveSession(session);

      // Update store
      currentSession.set(session);

      return session;
    } catch (error) {
      console.error('Failed to create session:', error);
      throw error;
    }
  },

  async getSession(sessionId: string): Promise<Session> {
    try {
      // Try to get from IndexedDB first
      const session = await indexedDB.getSession(sessionId);
      if (session) {
        return session;
      }

      // Fallback to creating a mock session (replace with actual Tauri call when implemented)
      throw new Error('Session not found');
    } catch (error) {
      console.error('Failed to get session:', error);
      throw error;
    }
  },

  async listSessions(): Promise<Session[]> {
    try {
      // Get all sessions from IndexedDB
      const allSessions = await indexedDB.getAllSessions();

      // Sort by updated_at descending (most recent first)
      const sorted = allSessions.sort((a, b) => b.updated_at - a.updated_at);

      // Update store
      sessions.set(sorted);

      return sorted;
    } catch (error) {
      console.error('Failed to list sessions:', error);
      return [];
    }
  },

  async sendMessage(sessionId: string, content: string): Promise<void> {
    try {
      // Send message to sidecar via Tauri
      await tauri.sendMessage(sessionId, content);

      // Save to history file
      await tauri.appendToHistory(sessionId, `## User\n${content}`);
    } catch (error) {
      console.error('Failed to send message:', error);
      throw error;
    }
  },

  async pauseSession(sessionId: string): Promise<void> {
    try {
      // Pause via sidecar
      await tauri.pauseSession(sessionId);

      // Update status in IndexedDB
      const session = await indexedDB.getSession(sessionId);
      if (session) {
        await indexedDB.saveSession({ ...session, status: 'paused', updated_at: Date.now() });
      }

      // Update store if this is the current session
      currentSession.update((current) => {
        if (current?.id === sessionId) {
          return { ...current, status: 'paused', updated_at: Date.now() };
        }
        return current;
      });
    } catch (error) {
      console.error('Failed to pause session:', error);
      throw error;
    }
  },

  async resumeSession(sessionId: string): Promise<void> {
    try {
      // Resume via sidecar
      await tauri.resumeSession(sessionId);

      // Update status in IndexedDB
      const session = await indexedDB.getSession(sessionId);
      if (session) {
        await indexedDB.saveSession({ ...session, status: 'active', updated_at: Date.now() });
      }

      // Update store if this is the current session
      currentSession.update((current) => {
        if (current?.id === sessionId) {
          return { ...current, status: 'active', updated_at: Date.now() };
        }
        return current;
      });
    } catch (error) {
      console.error('Failed to resume session:', error);
      throw error;
    }
  },

  async approveToolExecution(requestId: string, approved: boolean): Promise<void> {
    try {
      // Send approval response via Tauri
      // This will be implemented in the Tauri backend
      console.log(`Tool approval ${approved ? 'approved' : 'rejected'} for request ${requestId}`);

      // TODO: Implement actual Tauri command when backend is ready
      // await tauri.approveToolExecution(requestId, approved);
    } catch (error) {
      console.error('Failed to approve tool execution:', error);
      throw error;
    }
  },
};
