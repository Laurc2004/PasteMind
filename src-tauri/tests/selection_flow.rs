use pastemind::{
  error::AppError,
  models::{ClipboardEntry, EntryKind},
  selection::{select_entry_with_handlers, SelectOutcome}
};
use std::cell::Cell;

fn text_entry() -> ClipboardEntry {
  ClipboardEntry {
    id: "entry-1".to_string(),
    kind: EntryKind::Text,
    text_preview: Some("demo".to_string()),
    text_content: Some("demo".to_string()),
    image_path: None,
    image_preview_data: None,
    size_bytes: 4,
    source_app: "com.apple.TextEdit".to_string(),
    created_at: "2026-03-08T00:00:00Z".to_string(),
    hash: "hash".to_string()
  }
}

#[test]
fn select_entry_runs_autopaste_when_permission_available() {
  let called = Cell::new(false);
  let outcome = select_entry_with_handlers(
    &text_entry(),
    true,
    true,
    |_| Ok(()),
    || {
      called.set(true);
      Ok(())
    }
  )
  .expect("selection should succeed");

  assert_eq!(outcome, SelectOutcome::ClipboardAndAutoPaste);
  assert!(called.get());
}

#[test]
fn select_entry_returns_permission_error_when_missing_accessibility() {
  let result = select_entry_with_handlers(&text_entry(), true, false, |_| Ok(()), || Ok(()));
  assert!(matches!(result, Err(AppError::PermissionDenied)));
}
