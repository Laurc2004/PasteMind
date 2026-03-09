use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
  Text,
  Image
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
  pub id: String,
  pub kind: EntryKind,
  pub text_preview: Option<String>,
  pub text_content: Option<String>,
  pub image_path: Option<String>,
  pub image_preview_data: Option<String>,
  pub size_bytes: u64,
  pub source_app: String,
  pub created_at: String,
  pub hash: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
  pub max_items: u32,
  pub max_image_mb: u32,
  pub hotkey: String,
  pub auto_paste_default: bool,
  pub excluded_bundle_ids: Vec<String>,
  #[serde(default = "default_ui_locale")]
  pub ui_locale: String
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      max_items: 500,
      max_image_mb: 10,
      hotkey: "Cmd+Shift+V".to_string(),
      auto_paste_default: true,
      excluded_bundle_ids: vec![
        "com.apple.keychainaccess".to_string(),
        "com.agilebits.onepassword7".to_string(),
        "com.1password.1password".to_string(),
        "com.lastpass.LastPass".to_string(),
        "com.bitwarden.desktop".to_string()
      ],
      ui_locale: default_ui_locale()
    }
  }
}

fn default_ui_locale() -> String {
  "zh-CN".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsPatch {
  pub max_items: Option<u32>,
  pub max_image_mb: Option<u32>,
  pub hotkey: Option<String>,
  pub auto_paste_default: Option<bool>,
  pub excluded_bundle_ids: Option<Vec<String>>,
  pub ui_locale: Option<String>
}

impl Settings {
  pub fn apply_patch(&mut self, patch: SettingsPatch) {
    if let Some(value) = patch.max_items {
      self.max_items = value;
    }
    if let Some(value) = patch.max_image_mb {
      self.max_image_mb = value;
    }
    if let Some(value) = patch.hotkey {
      self.hotkey = value;
    }
    if let Some(value) = patch.auto_paste_default {
      self.auto_paste_default = value;
    }
    if let Some(value) = patch.excluded_bundle_ids {
      self.excluded_bundle_ids = value;
    }
    if let Some(value) = patch.ui_locale {
      self.ui_locale = value;
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionState {
  pub accessibility_granted: bool,
  pub can_auto_paste: bool
}

#[derive(Debug, Clone)]
pub struct ImagePayload {
  pub width: usize,
  pub height: usize,
  pub bytes: Vec<u8>
}

#[derive(Debug, Clone)]
pub enum ClipboardPayload {
  Text(String),
  Image(ImagePayload)
}
