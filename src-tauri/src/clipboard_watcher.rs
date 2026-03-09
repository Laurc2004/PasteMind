use std::{sync::Arc, time::Duration};

use tauri::{AppHandle, Emitter};

use crate::{
  app_state::AppState,
  clipboard::{payload_hash, read_payload},
  error::AppResult,
  models::ClipboardPayload,
  permissions
};

pub fn start(app: AppHandle, state: Arc<AppState>) {
  tauri::async_runtime::spawn(async move {
    let mut last_change_count = current_change_count().unwrap_or(0);
    let mut fallback_last_hash: Option<String> = None;

    loop {
      tokio::time::sleep(Duration::from_millis(300)).await;

      if let Some(current) = current_change_count() {
        if current == last_change_count {
          continue;
        }
        last_change_count = current;
      }

      if let Err(err) = handle_change(&app, &state, &mut fallback_last_hash) {
        log::debug!("clipboard watcher skipped: {err}");
      }
    }
  });
}

fn handle_change(app: &AppHandle, state: &Arc<AppState>, fallback_last_hash: &mut Option<String>) -> AppResult<()> {
  let settings = state.settings_snapshot();
  let source_app = permissions::frontmost_bundle_id().unwrap_or_else(|| "unknown".to_string());

  if permissions::should_exclude_source(&source_app, &settings.excluded_bundle_ids) {
    return Ok(());
  }

  let Some(payload) = read_payload()? else {
    log::debug!("clipboard watcher found no supported payload");
    return Ok(());
  };

  let hash = payload_hash(&payload);

  if state.consume_suppressed_hash(&hash) {
    log::debug!("clipboard watcher skipped self-initiated clipboard write");
    return Ok(());
  }

  if current_change_count().is_none() && fallback_last_hash.as_ref() == Some(&hash) {
    return Ok(());
  }

  let inserted = match payload {
    ClipboardPayload::Text(text) => state
      .storage
      .insert_text(&text, &source_app, &hash, settings.max_items)?,
    ClipboardPayload::Image(image) => state.storage.insert_image(
      &image,
      &source_app,
      &hash,
      settings.max_image_mb,
      settings.max_items
    )?
  };

  if inserted {
    *fallback_last_hash = Some(hash);
    let _ = app.emit("history:updated", serde_json::json!({ "reason": "insert" }));
  } else {
    log::debug!("clipboard watcher ignored payload due to dedupe or size policy");
  }

  Ok(())
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
fn current_change_count() -> Option<i64> {
  use objc::{class, msg_send, runtime::Object, sel, sel_impl};

  unsafe {
    let pasteboard: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
    if pasteboard.is_null() {
      return None;
    }

    let count: i64 = msg_send![pasteboard, changeCount];
    Some(count)
  }
}

#[cfg(not(target_os = "macos"))]
fn current_change_count() -> Option<i64> {
  None
}
