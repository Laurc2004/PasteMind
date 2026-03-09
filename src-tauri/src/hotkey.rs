use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{
  app_state::AppState,
  error::{AppError, AppResult}
};

pub const DEFAULT_HOTKEY: &str = "Cmd+Shift+V";

pub fn normalize_hotkey(input: &str) -> String {
  input
    .split('+')
    .map(str::trim)
    .filter(|token| !token.is_empty())
    .collect::<Vec<_>>()
    .join("+")
}

pub fn validate_hotkey(input: &str) -> AppResult<()> {
  let normalized = normalize_hotkey(input);
  if normalized.is_empty() {
    return Err(AppError::InvalidSettings("hotkey cannot be empty".to_string()));
  }

  normalized
    .parse::<Shortcut>()
    .map_err(|err| {
      AppError::InvalidSettings(format!(
        "invalid hotkey format: {err}. Example: Cmd+Shift+V"
      ))
    })
    .map(|_| ())
}

pub fn apply_hotkey(app: &AppHandle, input: &str) -> AppResult<()> {
  let normalized = normalize_hotkey(input);
  validate_hotkey(&normalized)?;

  app
    .global_shortcut()
    .unregister_all()
    .map_err(|err| AppError::Shortcut(err.to_string()))?;

  app
    .global_shortcut()
    .register(normalized.as_str())
    .map_err(|err| AppError::Shortcut(err.to_string()))
}

pub fn configured_hotkey_or_default(app: &AppHandle) -> String {
  if let Some(state) = app.try_state::<Arc<AppState>>() {
    let configured = state.settings_snapshot().hotkey;
    let normalized = normalize_hotkey(&configured);
    if validate_hotkey(&normalized).is_ok() {
      return normalized;
    }
  }

  DEFAULT_HOTKEY.to_string()
}

pub fn matches_active_hotkey(app: &AppHandle, shortcut: &Shortcut) -> bool {
  configured_hotkey_or_default(app)
    .parse::<Shortcut>()
    .map(|configured| configured == *shortcut)
    .unwrap_or(false)
}
