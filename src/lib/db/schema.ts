import Dexie, { type Table } from 'dexie';
import type { Session } from '$lib/types/session';
import type { Message } from '$lib/types/message';
import type { ChamberState } from '$lib/types/chamber';

export interface DBSession extends Session {
  id: string;
}

export interface DBMessage {
  id?: number;
  session_id: string;
  timestamp: number;
  message: Message;
}

export interface DBChamberState extends ChamberState {
  id?: string;
  session_id: string;
  updated_at: number;
}

export class ChamberDatabase extends Dexie {
  sessions!: Table<DBSession, string>;
  messages!: Table<DBMessage, number>;
  chamber_states!: Table<DBChamberState, string>;

  constructor() {
    super('ChamberDB');

    this.version(1).stores({
      sessions: 'id, status, created_at, updated_at',
      messages: '++id, session_id, timestamp',
      chamber_states: 'session_id, updated_at',
    });
  }
}

export const db = new ChamberDatabase();
