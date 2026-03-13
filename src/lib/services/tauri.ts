import { invoke } from '@tauri-apps/api/core';
import type { ChamberConfig } from '$lib/types/config';
import type { Session } from '$lib/types/session';

// Config Commands
export async function loadConfig(): Promise<ChamberConfig> {
  return await invoke<ChamberConfig>('load_config');
}

export async function saveConfig(config: ChamberConfig): Promise<void> {
  return await invoke('save_config', { config });
}

export async function getConfig(): Promise<ChamberConfig> {
  return await invoke<ChamberConfig>('get_config');
}

// Workspace Commands
export async function initWorkspace(): Promise<void> {
  return await invoke('init_workspace');
}

export async function listSessions(): Promise<string[]> {
  return await invoke<string[]>('list_sessions');
}

export async function loadSessionHistory(sessionId: string): Promise<string> {
  return await invoke<string>('load_session_history', { sessionId });
}

export async function savePlan(sessionId: string, plan: string): Promise<void> {
  return await invoke('save_plan', { sessionId, plan });
}

export async function appendToHistory(sessionId: string, content: string): Promise<void> {
  return await invoke('append_to_history', { sessionId, content });
}

// Sidecar Commands
export async function startSidecar(): Promise<void> {
  return await invoke('start_sidecar');
}

export async function stopSidecar(): Promise<void> {
  return await invoke('stop_sidecar');
}

export async function restartSidecar(): Promise<void> {
  return await invoke('restart_sidecar');
}

export async function healthCheckSidecar(): Promise<boolean> {
  return await invoke<boolean>('health_check_sidecar');
}

export async function getWebsocketUrl(): Promise<string> {
  return await invoke<string>('get_websocket_url');
}

export async function isSidecarRunning(): Promise<boolean> {
  return await invoke<boolean>('is_sidecar_running');
}

// Session Commands
export async function createSession(title: string): Promise<Session> {
  return await invoke<Session>('create_session', { title });
}

export async function sendMessage(sessionId: string, content: string): Promise<string | null> {
  return await invoke<string | null>('send_message', { sessionId, content });
}

export async function pauseSession(sessionId: string): Promise<void> {
  return await invoke('pause_session', { sessionId });
}

export async function resumeSession(sessionId: string): Promise<void> {
  return await invoke('resume_session', { sessionId });
}
