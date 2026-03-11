import type { Session } from '$lib/types/session';
import type { Message } from '$lib/types/message';
import * as tauri from '$lib/services/tauri';
import { indexedDB } from '$lib/services/indexeddb';
import { currentSession, updateSessionStatus } from '$lib/stores/session';
import { addMessage, setMessages } from '$lib/stores/chat';
import { setLoading, setError } from '$lib/stores/ui';

export class SessionManager {
  private websocket: WebSocket | null = null;
  private sessionId: string | null = null;

  async createSession(title: string): Promise<Session> {
    try {
      setLoading(true);

      // Create session via Tauri
      const session = await tauri.createSession(title);

      // Save to IndexedDB
      await indexedDB.saveSession(session);

      // Update store
      currentSession.set(session);

      this.sessionId = session.id;

      setLoading(false);
      return session;
    } catch (error) {
      setLoading(false);
      setError(`Failed to create session: ${error}`);
      throw error;
    }
  }

  async loadSession(sessionId: string): Promise<void> {
    try {
      setLoading(true);

      // Load session from IndexedDB
      const session = await indexedDB.getSession(sessionId);

      if (!session) {
        throw new Error('Session not found');
      }

      // Load messages
      const messages = await indexedDB.getMessages(sessionId);

      // Update stores
      currentSession.set(session);
      setMessages(messages);

      this.sessionId = sessionId;

      setLoading(false);
    } catch (error) {
      setLoading(false);
      setError(`Failed to load session: ${error}`);
      throw error;
    }
  }

  async sendMessage(content: string): Promise<void> {
    if (!this.sessionId) {
      throw new Error('No active session');
    }

    try {
      // Add user message to UI immediately
      const userMessage: Message = {
        type: 'UserMessage',
        content,
      };

      addMessage(userMessage);

      // Save to IndexedDB
      await indexedDB.saveMessage(this.sessionId, userMessage);

      // Connect WebSocket if not connected
      if (!this.websocket) {
        await this.connectWebSocket();
      }

      // Send message to sidecar
      await tauri.sendMessage(this.sessionId, content);

      // Save to history file
      await tauri.appendToHistory(this.sessionId, `## User\n${content}`);
    } catch (error) {
      setError(`Failed to send message: ${error}`);
      throw error;
    }
  }

  async pauseSession(): Promise<void> {
    if (!this.sessionId) {
      throw new Error('No active session');
    }

    try {
      // Pause via sidecar
      await tauri.pauseSession(this.sessionId);

      // Update status
      updateSessionStatus('paused');

      // Save to IndexedDB
      const session = await indexedDB.getSession(this.sessionId);
      if (session) {
        await indexedDB.saveSession({ ...session, status: 'paused' });
      }

      // Disconnect WebSocket
      this.disconnectWebSocket();
    } catch (error) {
      setError(`Failed to pause session: ${error}`);
      throw error;
    }
  }

  async resumeSession(): Promise<void> {
    if (!this.sessionId) {
      throw new Error('No active session');
    }

    try {
      // Resume via sidecar
      await tauri.resumeSession(this.sessionId);

      // Update status
      updateSessionStatus('active');

      // Save to IndexedDB
      const session = await indexedDB.getSession(this.sessionId);
      if (session) {
        await indexedDB.saveSession({ ...session, status: 'active' });
      }

      // Reconnect WebSocket
      await this.connectWebSocket();
    } catch (error) {
      setError(`Failed to resume session: ${error}`);
      throw error;
    }
  }

  private async connectWebSocket(): Promise<void> {
    if (this.websocket) {
      return; // Already connected
    }

    try {
      const wsUrl = await tauri.getWebsocketUrl();
      const url = `${wsUrl}?session_id=${this.sessionId}`;

      this.websocket = new WebSocket(url);

      this.websocket.onopen = () => {
        console.log('WebSocket connected');
      };

      this.websocket.onmessage = async (event) => {
        await this.handleWebSocketMessage(event.data);
      };

      this.websocket.onerror = (error) => {
        console.error('WebSocket error:', error);
        setError('WebSocket connection error');
      };

      this.websocket.onclose = () => {
        console.log('WebSocket disconnected');
        this.websocket = null;
      };
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
      throw error;
    }
  }

  private disconnectWebSocket(): void {
    if (this.websocket) {
      this.websocket.close();
      this.websocket = null;
    }
  }

  private async handleWebSocketMessage(data: string): Promise<void> {
    try {
      const message = JSON.parse(data);

      // Handle different message types
      switch (message.type) {
        case 'ReasoningStep':
        case 'AssistantMessage':
        case 'ToolApprovalRequest':
        case 'ToolExecution':
        case 'SystemMessage':
        case 'Error':
          addMessage(message);

          // Save to IndexedDB
          if (this.sessionId) {
            await indexedDB.saveMessage(this.sessionId, message);
          }
          break;

        default:
          console.warn('Unknown message type:', message.type);
      }
    } catch (error) {
      console.error('Failed to handle WebSocket message:', error);
    }
  }

  cleanup(): void {
    this.disconnectWebSocket();
    this.sessionId = null;
  }
}

// Singleton instance
export const sessionManager = new SessionManager();
