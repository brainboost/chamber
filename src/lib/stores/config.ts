import { writable, type Writable } from 'svelte/store';
import type { ChamberConfig } from '$lib/types/config';
import * as tauri from '$lib/services/tauri';

export const config: Writable<ChamberConfig | null> = writable(null);

export async function loadConfigStore(): Promise<void> {
  try {
    const loadedConfig = await tauri.loadConfig();
    config.set(loadedConfig);
  } catch (error) {
    console.error('Failed to load config:', error);
    throw error;
  }
}

export async function saveConfigStore(newConfig: ChamberConfig): Promise<void> {
  try {
    await tauri.saveConfig(newConfig);
    config.set(newConfig);
  } catch (error) {
    console.error('Failed to save config:', error);
    throw error;
  }
}
