use std::process::Command;

use crate::error::{AppError, AppResult};

pub fn trigger_cmd_v() -> AppResult<()> {
  trigger_cmd_v_to_bundle(None)
}

pub fn trigger_cmd_v_to_bundle(target_bundle: Option<&str>) -> AppResult<()> {
  let script = match target_bundle {
    Some(bundle) => format!(
      r#"tell application id "{bundle}" to activate
delay 0.08
tell application "System Events" to keystroke "v" using command down"#
    ),
    None => r#"delay 0.05
tell application "System Events" to keystroke "v" using command down"#
      .to_string()
  };

  let status = Command::new("osascript")
    .arg("-e")
    .arg(&script)
    .status()
    .map_err(|err| AppError::AutoPaste(format!("failed to invoke osascript: {err}")))?;

  if status.success() {
    Ok(())
  } else {
    Err(AppError::AutoPaste(format!(
      "osascript exited with status {status}"
    )))
  }
}
