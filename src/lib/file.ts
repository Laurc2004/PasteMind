import { convertFileSrc } from '@tauri-apps/api/core';

export function toFileSrc(path: string | null): string {
  if (!path) {
    return '';
  }
  return convertFileSrc(path);
}
