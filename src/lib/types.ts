export type EntryKind = 'text' | 'image';

export interface ClipboardEntry {
  id: string;
  kind: EntryKind;
  text_preview: string | null;
  text_content: string | null;
  image_path: string | null;
  image_preview_data: string | null;
  size_bytes: number;
  source_app: string;
  created_at: string;
  hash: string;
}

export interface Settings {
  max_items: number;
  max_image_mb: number;
  hotkey: string;
  auto_paste_default: boolean;
  excluded_bundle_ids: string[];
  ui_locale: 'zh-CN' | 'en-US';
}

export interface SettingsPatch {
  max_items?: number;
  max_image_mb?: number;
  hotkey?: string;
  auto_paste_default?: boolean;
  excluded_bundle_ids?: string[];
  ui_locale?: 'zh-CN' | 'en-US';
}

export interface PermissionState {
  accessibility_granted: boolean;
  can_auto_paste: boolean;
}
