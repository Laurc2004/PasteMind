import { invoke } from '@tauri-apps/api/core';
import type { ClipboardEntry, PermissionState, Settings, SettingsPatch } from '$lib/types';

export async function getHistory(params?: {
  query?: string;
  limit?: number;
  offset?: number;
}): Promise<ClipboardEntry[]> {
  return invoke<ClipboardEntry[]>('get_history', {
    query: params?.query,
    limit: params?.limit,
    offset: params?.offset
  });
}

export async function selectEntry(id: string, autoPaste: boolean): Promise<void> {
  await invoke('select_entry', { id, autoPaste });
}

export async function deleteEntry(id: string): Promise<void> {
  await invoke('delete_entry', { id });
}

export async function clearHistory(): Promise<void> {
  await invoke('clear_history');
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>('get_settings');
}

export async function updateSettings(patch: SettingsPatch): Promise<Settings> {
  return invoke<Settings>('update_settings', { patch });
}

export async function getPermissionState(): Promise<PermissionState> {
  return invoke<PermissionState>('get_permission_state');
}

export async function requestAccessibilityPermission(): Promise<boolean> {
  return invoke<boolean>('request_accessibility_permission');
}

export async function openAccessibilitySettings(): Promise<void> {
  await invoke('open_accessibility_settings');
}

export async function showMainWindow(): Promise<void> {
  await invoke('show_main_window');
}

export async function setHotkeyRecording(active: boolean): Promise<void> {
  await invoke('set_hotkey_recording', { active });
}
