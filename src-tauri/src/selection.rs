use crate::{
  error::{AppError, AppResult},
  models::ClipboardEntry
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectOutcome {
  ClipboardOnly,
  ClipboardAndAutoPaste
}

pub fn select_entry_with_handlers<F, G>(
  entry: &ClipboardEntry,
  auto_paste: bool,
  accessibility_granted: bool,
  write_clipboard: F,
  trigger_autopaste: G
) -> AppResult<SelectOutcome>
where
  F: Fn(&ClipboardEntry) -> AppResult<()>,
  G: Fn() -> AppResult<()>
{
  if auto_paste {
    if !accessibility_granted {
      return Err(AppError::PermissionDenied);
    }
  }

  write_clipboard(entry)?;

  if auto_paste {
    trigger_autopaste()?;
    return Ok(SelectOutcome::ClipboardAndAutoPaste);
  }

  Ok(SelectOutcome::ClipboardOnly)
}

#[cfg(test)]
mod tests {
  use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc
  };

  use super::*;
  use crate::models::{ClipboardEntry, EntryKind};

  fn make_entry() -> ClipboardEntry {
    ClipboardEntry {
      id: "1".to_string(),
      kind: EntryKind::Text,
      text_preview: Some("hello".to_string()),
      text_content: Some("hello".to_string()),
      image_path: None,
      image_preview_data: None,
      size_bytes: 5,
      source_app: "com.apple.TextEdit".to_string(),
      created_at: "2026-01-01T00:00:00Z".to_string(),
      hash: "hash".to_string()
    }
  }

  #[test]
  fn auto_paste_requires_permission() {
    let entry = make_entry();
    let writes = Arc::new(AtomicUsize::new(0));
    let writes_for_handler = writes.clone();
    let result = select_entry_with_handlers(
      &entry,
      true,
      false,
      move |_| {
        writes_for_handler.fetch_add(1, Ordering::SeqCst);
        Ok(())
      },
      || Ok(())
    );
    assert!(matches!(result, Err(AppError::PermissionDenied)));
    assert_eq!(writes.load(Ordering::SeqCst), 0);
  }

  #[test]
  fn selection_copies_and_pastes_when_permission_is_granted() {
    let entry = make_entry();
    let result = select_entry_with_handlers(
      &entry,
      true,
      true,
      |_| Ok(()),
      || Ok(())
    )
    .expect("select");

    assert_eq!(result, SelectOutcome::ClipboardAndAutoPaste);
  }
}
