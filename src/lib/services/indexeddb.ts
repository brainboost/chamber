import { db, type DBSession, type DBMessage, type DBChamberState } from '$lib/db/schema';
import type { Session } from '$lib/types/session';
import type { Message } from '$lib/types/message';
import type { ChamberState } from '$lib/types/chamber';

export class IndexedDBService {
  // Sessions
  async saveSession(session: Session): Promise<void> {
    await db.sessions.put(session as DBSession);
  }

  async getSession(sessionId: string): Promise<Session | undefined> {
    return await db.sessions.get(sessionId);
  }

  async getAllSessions(): Promise<Session[]> {
    return await db.sessions.orderBy('updated_at').reverse().toArray();
  }

  async deleteSession(sessionId: string): Promise<void> {
    await db.transaction('rw', [db.sessions, db.messages, db.chamber_states], async () => {
      await db.sessions.delete(sessionId);
      await db.messages.where('session_id').equals(sessionId).delete();
      await db.chamber_states.where('session_id').equals(sessionId).delete();
    });
  }

  // Messages
  async saveMessage(sessionId: string, message: Message): Promise<void> {
    await db.messages.add({
      session_id: sessionId,
      timestamp: Date.now(),
      message,
    });
  }

  async getMessages(sessionId: string): Promise<Message[]> {
    const dbMessages = await db.messages
      .where('session_id')
      .equals(sessionId)
      .sortBy('timestamp');

    return dbMessages.map((m) => m.message);
  }

  async clearMessages(sessionId: string): Promise<void> {
    await db.messages.where('session_id').equals(sessionId).delete();
  }

  // Chamber State
  async saveChamberState(sessionId: string, state: ChamberState): Promise<void> {
    await db.chamber_states.put({
      ...state,
      session_id: sessionId,
      updated_at: Date.now(),
    });
  }

  async getChamberState(sessionId: string): Promise<ChamberState | undefined> {
    const dbState = await db.chamber_states.get(sessionId);
    if (!dbState) return undefined;

    const { session_id, updated_at, ...state } = dbState;
    return state as ChamberState;
  }

  async deleteChamberState(sessionId: string): Promise<void> {
    await db.chamber_states.where('session_id').equals(sessionId).delete();
  }

  // Utilities
  async clearAll(): Promise<void> {
    await db.transaction('rw', [db.sessions, db.messages, db.chamber_states], async () => {
      await db.sessions.clear();
      await db.messages.clear();
      await db.chamber_states.clear();
    });
  }
}

export const indexedDB = new IndexedDBService();
