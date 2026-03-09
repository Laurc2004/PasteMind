use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
  app_state::AppState,
  autopaste,
  clipboard,
  error::AppError,
  hotkey,
  models::{ClipboardEntry, PermissionState, Settings, SettingsPatch},
  permissions,
  selection,
  settings
};

#[tauri::command]
pub fn get_history(
  state: State<'_, Arc<AppState>>,
  query: Option<String>,
  limit: Option<u32>,
  offset: Option<u32>
) -> Result<Vec<ClipboardEntry>, String> {
  let limit = limit.unwrap_or(200).min(500);
  let offset = offset.unwrap_or(0);
  state
    .storage
    .history(query.as_deref(), limit, offset)
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn select_entry(
  app: AppHandle,
  state: State<'_, Arc<AppState>>,
  id: String,
  auto_paste: bool
) -> Result<(), String> {
  let entry = state
    .storage
    .entry_by_id(&id)
    .map_err(|err| err.to_string())?
    .ok_or_else(|| AppError::EntryNotFound.to_string())?;
  let target_bundle = state.last_foreground_bundle();

  state.suppress_history_hash(entry.hash.clone());

  if let Some(window) = app.get_webview_window("main") {
    let _ = window.hide();
  }

  selection::select_entry_with_handlers(
    &entry,
    auto_paste,
    permissions::is_accessibility_granted(),
    clipboard::write_entry_to_clipboard,
    move || autopaste::trigger_cmd_v_to_bundle(target_bundle.as_deref())
  )
  .map_err(|err| err.to_string())?;

  let _ = app.emit("permission:changed", permissions::permission_state());
  Ok(())
}

#[tauri::command]
pub fn delete_entry(app: AppHandle, state: State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
  state
    .storage
    .delete_entry(&id)
    .map_err(|err| err.to_string())?;
  let _ = app.emit("history:updated", serde_json::json!({ "reason": "delete" }));
  Ok(())
}

#[tauri::command]
pub fn clear_history(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
  state.storage.clear_history().map_err(|err| err.to_string())?;
  let _ = app.emit("history:updated", serde_json::json!({ "reason": "clear" }));
  Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
  Ok(state.settings_snapshot())
}

#[tauri::command]
pub fn update_settings(
  app: AppHandle,
  state: State<'_, Arc<AppState>>,
  patch: SettingsPatch
) -> Result<Settings, String> {
  let current = state.settings_snapshot();
  let next = settings::merge_and_validate(&current, patch).map_err(|err| err.to_string())?;

  if next.hotkey != current.hotkey {
    hotkey::apply_hotkey(&app, &next.hotkey).map_err(|err| err.to_string())?;
  }

  state
    .storage
    .save_settings(&next)
    .map_err(|err| err.to_string())?;
  state.replace_settings(next.clone());

  let _ = app.emit("history:updated", serde_json::json!({ "reason": "settings" }));
  Ok(next)
}

#[tauri::command]
pub fn get_permission_state() -> Result<PermissionState, String> {
  Ok(permissions::permission_state())
}

#[tauri::command]
pub fn request_accessibility_permission(app: AppHandle) -> Result<bool, String> {
  let granted = permissions::request_accessibility_permission();
  let _ = app.emit("permission:changed", permissions::permission_state());
  Ok(granted)
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
  permissions::open_accessibility_settings().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<(), String> {
  remember_foreground_bundle(&app, &state);

  let window = app
    .get_webview_window("main")
    .ok_or_else(|| AppError::WindowNotFound("main".to_string()).to_string())?;

  window.show().map_err(|err| err.to_string())?;
  window.unminimize().map_err(|err| err.to_string())?;
  window.set_focus().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn set_hotkey_recording(state: State<'_, Arc<AppState>>, active: bool) -> Result<(), String> {
  state.set_hotkey_recording(active);
  Ok(())
}

fn remember_foreground_bundle(app: &AppHandle, state: &Arc<AppState>) {
  let own_identifier = app.config().identifier.clone();
  if let Some(bundle) = permissions::frontmost_bundle_id().filter(|bundle| bundle != &own_identifier) {
    state.set_last_foreground_bundle(Some(bundle));
  }
}
