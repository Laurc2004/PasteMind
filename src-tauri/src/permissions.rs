use std::{process::Command, time::Duration};

use tauri::{AppHandle, Emitter};

use crate::{
  error::{AppError, AppResult},
  models::PermissionState
};

#[cfg(target_os = "macos")]
use macos_accessibility_client::accessibility;

pub fn permission_state() -> PermissionState {
  let granted = is_accessibility_granted();
  PermissionState {
    accessibility_granted: granted,
    can_auto_paste: granted
  }
}

pub fn is_accessibility_granted() -> bool {
  #[cfg(target_os = "macos")]
  {
    accessibility::application_is_trusted()
  }

  #[cfg(not(target_os = "macos"))]
  {
    false
  }
}

pub fn request_accessibility_permission() -> bool {
  #[cfg(target_os = "macos")]
  {
    accessibility::application_is_trusted_with_prompt()
  }

  #[cfg(not(target_os = "macos"))]
  {
    false
  }
}

pub fn open_accessibility_settings() -> AppResult<()> {
  let status = Command::new("open")
    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
    .status()
    .map_err(|err| AppError::Other(format!("failed to open System Settings: {err}")))?;

  if status.success() {
    Ok(())
  } else {
    Err(AppError::Other(format!(
      "open command exited with status {status}"
    )))
  }
}

pub fn frontmost_bundle_id() -> Option<String> {
  let output = Command::new("osascript")
    .arg("-e")
    .arg(
      r#"tell application "System Events" to get bundle identifier of first application process whose frontmost is true"#
    )
    .output()
    .ok()?;

  if !output.status.success() {
    return None;
  }

  let value = String::from_utf8(output.stdout).ok()?;
  let trimmed = value.trim();
  if trimmed.is_empty() {
    None
  } else {
    Some(trimmed.to_string())
  }
}

pub fn should_exclude_source(source: &str, excluded_bundle_ids: &[String]) -> bool {
  excluded_bundle_ids.iter().any(|item| item == source)
}

pub fn start_permission_watcher(app: AppHandle) {
  tauri::async_runtime::spawn(async move {
    let mut last = is_accessibility_granted();

    loop {
      tokio::time::sleep(Duration::from_secs(2)).await;
      let current = is_accessibility_granted();

      if current != last {
        last = current;
        let _ = app.emit("permission:changed", permission_state());
      }
    }
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn source_is_excluded_when_in_list() {
    let list = vec!["com.1password.1password".to_string()];
    assert!(should_exclude_source("com.1password.1password", &list));
    assert!(!should_exclude_source("com.apple.TextEdit", &list));
  }
}
