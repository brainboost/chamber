export interface Session {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
  status: SessionStatus;
  workspace_path: string;
}

export type SessionStatus = 'active' | 'paused' | 'completed' | 'failed';
