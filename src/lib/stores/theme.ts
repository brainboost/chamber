import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';
import { config, saveConfigStore } from './config';
import type { ThemeMode } from '$lib/types/config';

export const theme = writable<ThemeMode>('system');

let mediaQuery: MediaQueryList | null = null;
let systemDark = false;

function applyTheme(mode: ThemeMode, isSystemDark: boolean) {
  if (!browser) return;
  
  const isDark = mode === 'dark' || (mode === 'system' && isSystemDark);
  if (isDark) {
    document.documentElement.classList.add('dark');
  } else {
    document.documentElement.classList.remove('dark');
  }
}

export function initTheme() {
  if (!browser) return;

  mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  systemDark = mediaQuery.matches;

  mediaQuery.addEventListener('change', (e) => {
    systemDark = e.matches;
    const currentMode = get(theme);
    if (currentMode === 'system') {
      applyTheme(currentMode, systemDark);
    }
  });

  // Subscribe to our config store to get the initial/saved value
  config.subscribe((chamberConfig) => {
    if (chamberConfig && chamberConfig.ui && chamberConfig.ui.theme) {
      theme.set(chamberConfig.ui.theme);
      applyTheme(chamberConfig.ui.theme, systemDark);
    }
  });

  // Also apply whenever the local theme store changes directly
  theme.subscribe((t) => {
    applyTheme(t, systemDark);
  });
}

export async function setTheme(newTheme: ThemeMode) {
  theme.set(newTheme);
  
  // Save to config
  const currentConfig = get(config);
  if (currentConfig) {
    const updatedConfig = { ...currentConfig, ui: { theme: newTheme } };
    try {
      await saveConfigStore(updatedConfig);
    } catch (err) {
      console.error('Failed to save theme setting:', err);
    }
  }
}
