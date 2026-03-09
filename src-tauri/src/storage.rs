use std::{
  fs,
  io::Cursor,
  path::{Path, PathBuf}
};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder, RgbaImage};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::{
  error::{AppError, AppResult},
  models::{ClipboardEntry, EntryKind, ImagePayload, Settings}
};

const DEDUPE_WINDOW_MS: i64 = 1_500;

pub struct Storage {
  conn: Mutex<Connection>,
  image_root: PathBuf
}

impl Storage {
  pub fn new(db_path: impl AsRef<Path>, image_root: impl AsRef<Path>) -> AppResult<Self> {
    if let Some(parent) = db_path.as_ref().parent() {
      fs::create_dir_all(parent)?;
    }
    fs::create_dir_all(image_root.as_ref())?;

    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    conn.execute_batch(
      r#"
      CREATE TABLE IF NOT EXISTS clipboard_entries (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        text_preview TEXT,
        text_content TEXT,
        image_path TEXT,
        image_preview_data TEXT,
        size_bytes INTEGER NOT NULL,
        source_app TEXT NOT NULL,
        hash TEXT NOT NULL,
        created_at TEXT NOT NULL,
        created_ts INTEGER NOT NULL
      );

      CREATE INDEX IF NOT EXISTS idx_clipboard_created_ts ON clipboard_entries(created_ts DESC);
      CREATE INDEX IF NOT EXISTS idx_clipboard_hash ON clipboard_entries(hash);

      CREATE TABLE IF NOT EXISTS app_settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        payload TEXT NOT NULL
      );
      "#
    )?;
    Self::ensure_column_exists(&conn, "clipboard_entries", "image_preview_data", "TEXT")?;

    Ok(Self {
      conn: Mutex::new(conn),
      image_root: image_root.as_ref().to_path_buf()
    })
  }

  pub fn load_settings(&self) -> AppResult<Option<Settings>> {
    let conn = self.conn.lock();
    let mut stmt = conn.prepare("SELECT payload FROM app_settings WHERE id = 1")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
      let payload: String = row.get(0)?;
      let settings: Settings = serde_json::from_str(&payload)?;
      Ok(Some(settings))
    } else {
      Ok(None)
    }
  }

  pub fn save_settings(&self, settings: &Settings) -> AppResult<()> {
    let payload = serde_json::to_string(settings)?;
    let conn = self.conn.lock();
    conn.execute(
      "
      INSERT INTO app_settings (id, payload)
      VALUES (1, ?1)
      ON CONFLICT(id) DO UPDATE SET payload = excluded.payload
      ",
      params![payload]
    )?;
    Ok(())
  }

  pub fn history(&self, query: Option<&str>, limit: u32, offset: u32) -> AppResult<Vec<ClipboardEntry>> {
    let conn = self.conn.lock();
    let mut entries = Vec::new();

    if let Some(query) = query {
      let trimmed = query.trim();
      if !trimmed.is_empty() {
        let pattern = format!("%{}%", trimmed);
        let mut stmt = conn.prepare(
          "
          SELECT id, kind, text_preview, text_content, image_path, image_preview_data, size_bytes, source_app, hash, created_at
          FROM clipboard_entries
          WHERE text_preview LIKE ?1 OR source_app LIKE ?1
          ORDER BY created_ts DESC
          LIMIT ?2 OFFSET ?3
          "
        )?;
        let rows = stmt.query_map(params![pattern, limit as i64, offset as i64], Self::row_to_entry)?;
        for row in rows {
          entries.push(row?);
        }
        return Ok(entries);
      }
    }

    let mut stmt = conn.prepare(
      "
      SELECT id, kind, text_preview, text_content, image_path, image_preview_data, size_bytes, source_app, hash, created_at
      FROM clipboard_entries
      ORDER BY created_ts DESC
      LIMIT ?1 OFFSET ?2
      "
    )?;

    let rows = stmt.query_map(params![limit as i64, offset as i64], Self::row_to_entry)?;
    for row in rows {
      entries.push(row?);
    }

    Ok(entries)
  }

  pub fn entry_by_id(&self, id: &str) -> AppResult<Option<ClipboardEntry>> {
    let conn = self.conn.lock();
    let mut stmt = conn.prepare(
      "
      SELECT id, kind, text_preview, text_content, image_path, image_preview_data, size_bytes, source_app, hash, created_at
      FROM clipboard_entries
      WHERE id = ?1
      LIMIT 1
      "
    )?;
    let mut rows = stmt.query(params![id])?;

    if let Some(row) = rows.next()? {
      Ok(Some(Self::map_row(row)?))
    } else {
      Ok(None)
    }
  }

  pub fn insert_text(&self, text: &str, source_app: &str, hash: &str, max_items: u32) -> AppResult<bool> {
    if text.trim().is_empty() {
      return Ok(false);
    }

    let now = Utc::now();
    let now_ms = now.timestamp_millis();

    {
      let conn = self.conn.lock();
      if Self::has_recent_duplicate(&conn, hash, now_ms)? {
        return Ok(false);
      }

      let id = Uuid::new_v4().to_string();
      conn.execute(
        "
        INSERT INTO clipboard_entries
        (id, kind, text_preview, text_content, image_path, image_preview_data, size_bytes, source_app, hash, created_at, created_ts)
        VALUES (?1, 'text', ?2, ?3, NULL, NULL, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
          id,
          truncate_preview(text, 180),
          text,
          text.as_bytes().len() as i64,
          source_app,
          hash,
          now.to_rfc3339(),
          now_ms
        ]
      )?;
    }

    self.prune_to_limit(max_items)?;
    Ok(true)
  }

  pub fn insert_image(
    &self,
    image: &ImagePayload,
    source_app: &str,
    hash: &str,
    max_image_mb: u32,
    max_items: u32
  ) -> AppResult<bool> {
    let limit = max_image_mb as usize * 1024 * 1024;

    let now = Utc::now();
    let now_ms = now.timestamp_millis();

    {
      let conn = self.conn.lock();
      if Self::has_recent_duplicate(&conn, hash, now_ms)? {
        return Ok(false);
      }
    }

    let image_rgba = RgbaImage::from_raw(image.width as u32, image.height as u32, image.bytes.clone())
      .ok_or(AppError::InvalidImageData)?;
    let encoded_png = encode_png(&image_rgba)?;
    if encoded_png.len() > limit {
      return Ok(false);
    }
    let preview_data = build_preview_data_url(&image_rgba)?;

    let id = Uuid::new_v4().to_string();
    let path = self.persist_image_bytes(&id, now, &encoded_png)?;
    let path_string = path.to_string_lossy().to_string();
    let size_bytes = encoded_png.len() as i64;

    {
      let conn = self.conn.lock();
      conn.execute(
        "
        INSERT INTO clipboard_entries
        (id, kind, text_preview, text_content, image_path, image_preview_data, size_bytes, source_app, hash, created_at, created_ts)
        VALUES (?1, 'image', NULL, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
          id,
          path_string,
          preview_data,
          size_bytes,
          source_app,
          hash,
          now.to_rfc3339(),
          now_ms
        ]
      )?;
    }

    self.prune_to_limit(max_items)?;
    Ok(true)
  }

  pub fn delete_entry(&self, id: &str) -> AppResult<()> {
    let image_path = {
      let conn = self.conn.lock();
      let mut stmt = conn.prepare("SELECT image_path FROM clipboard_entries WHERE id = ?1")?;
      let mut rows = stmt.query(params![id])?;
      let image_path = if let Some(row) = rows.next()? {
        row.get::<_, Option<String>>(0)?
      } else {
        None
      };
      conn.execute("DELETE FROM clipboard_entries WHERE id = ?1", params![id])?;
      image_path
    };

    if let Some(path) = image_path {
      let _ = remove_image_file(&path);
    }

    Ok(())
  }

  pub fn clear_history(&self) -> AppResult<()> {
    let image_paths = {
      let conn = self.conn.lock();
      let mut stmt = conn.prepare("SELECT image_path FROM clipboard_entries WHERE image_path IS NOT NULL")?;
      let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
      let mut paths = Vec::new();
      for row in rows {
        paths.push(row?);
      }
      conn.execute("DELETE FROM clipboard_entries", [])?;
      paths
    };

    for path in image_paths {
      let _ = remove_image_file(&path);
    }

    Ok(())
  }

  fn has_recent_duplicate(conn: &Connection, hash: &str, now_ms: i64) -> AppResult<bool> {
    let mut stmt = conn.prepare(
      "
      SELECT created_ts
      FROM clipboard_entries
      WHERE hash = ?1
      ORDER BY created_ts DESC
      LIMIT 1
      "
    )?;

    let mut rows = stmt.query(params![hash])?;
    if let Some(row) = rows.next()? {
      let created_ts: i64 = row.get(0)?;
      Ok(now_ms - created_ts <= DEDUPE_WINDOW_MS)
    } else {
      Ok(false)
    }
  }

  fn persist_image_bytes(&self, id: &str, now: DateTime<Utc>, bytes: &[u8]) -> AppResult<PathBuf> {
    let rel_dir = now.format("%Y/%m/%d").to_string();
    let dir = self.image_root.join(rel_dir);
    fs::create_dir_all(&dir)?;

    let path = dir.join(format!("{}.png", id));
    fs::write(&path, bytes)?;

    Ok(path)
  }

  fn ensure_column_exists(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    column_type: &str
  ) -> AppResult<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;

    for column in columns {
      if column? == column_name {
        return Ok(());
      }
    }

    conn.execute(
      &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_type}"),
      []
    )?;

    Ok(())
  }

  fn prune_to_limit(&self, max_items: u32) -> AppResult<()> {
    let stale: Vec<(String, Option<String>)> = {
      let conn = self.conn.lock();
      let mut stmt = conn.prepare(
        "
        SELECT id, image_path
        FROM clipboard_entries
        ORDER BY created_ts DESC
        LIMIT -1 OFFSET ?1
        "
      )?;
      let rows = stmt.query_map(params![max_items as i64], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
      })?;

      let mut stale = Vec::new();
      for row in rows {
        stale.push(row?);
      }
      stale
    };

    if stale.is_empty() {
      return Ok(());
    }

    {
      let conn = self.conn.lock();
      for (id, _) in &stale {
        conn.execute("DELETE FROM clipboard_entries WHERE id = ?1", params![id])?;
      }
    }

    for (_, image_path) in stale {
      if let Some(path) = image_path {
        let _ = remove_image_file(&path);
      }
    }

    Ok(())
  }

  fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipboardEntry> {
    Self::map_row(row)
  }

  fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipboardEntry> {
    let kind_raw: String = row.get(1)?;
    let kind = match kind_raw.as_str() {
      "text" => EntryKind::Text,
      "image" => EntryKind::Image,
      _ => EntryKind::Text
    };

    Ok(ClipboardEntry {
      id: row.get(0)?,
      kind,
      text_preview: row.get(2)?,
      text_content: row.get(3)?,
      image_path: row.get(4)?,
      image_preview_data: row.get(5)?,
      size_bytes: row.get::<_, i64>(6)? as u64,
      source_app: row.get(7)?,
      hash: row.get(8)?,
      created_at: row.get(9)?
    })
  }
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
  if text.chars().count() <= max_chars {
    return text.to_string();
  }

  let mut output = String::new();
  for (index, ch) in text.chars().enumerate() {
    if index >= max_chars {
      break;
    }
    output.push(ch);
  }
  output.push('…');
  output
}

fn remove_image_file(path: &str) -> std::io::Result<()> {
  let p = PathBuf::from(path);
  if p.exists() {
    fs::remove_file(p)?;
  }
  Ok(())
}

fn encode_png(image: &RgbaImage) -> AppResult<Vec<u8>> {
  let mut encoded = Vec::new();
  {
    let mut cursor = Cursor::new(&mut encoded);
    let encoder = PngEncoder::new(&mut cursor);
    encoder
      .write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        ColorType::Rgba8.into()
      )
      .map_err(|err| AppError::Io(format!("failed to encode image png: {err}")))?;
  }
  Ok(encoded)
}

fn build_preview_data_url(image: &RgbaImage) -> AppResult<String> {
  let thumb = image::imageops::thumbnail(image, 360, 220);
  let encoded = encode_png(&thumb)?;
  Ok(format!("data:image/png;base64,{}", BASE64_STANDARD.encode(encoded)))
}

#[cfg(test)]
mod tests {
  use tempfile::tempdir;

  use super::*;

  fn test_storage() -> Storage {
    let dir = tempdir().expect("tempdir");
    let db = dir.path().join("pastemind.db");
    let images = dir.path().join("images");
    Storage::new(db, images).expect("storage")
  }

  #[test]
  fn insert_text_and_query_history() {
    let storage = test_storage();
    let inserted = storage
      .insert_text("hello world", "com.apple.TextEdit", "h1", 500)
      .expect("insert");
    assert!(inserted);

    let list = storage.history(None, 20, 0).expect("history");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].text_content.as_deref(), Some("hello world"));
  }

  #[test]
  fn dedupe_blocks_immediate_duplicate_hash() {
    let storage = test_storage();
    assert!(storage
      .insert_text("hello", "com.apple.TextEdit", "same-hash", 500)
      .expect("first"));
    assert!(!storage
      .insert_text("hello", "com.apple.TextEdit", "same-hash", 500)
      .expect("second"));
  }

  #[test]
  fn retention_prunes_old_records() {
    let storage = test_storage();
    for index in 0..3 {
      let hash = format!("h-{index}");
      let text = format!("text-{index}");
      std::thread::sleep(std::time::Duration::from_millis(2));
      let _ = storage.insert_text(&text, "com.apple.TextEdit", &hash, 2);
    }

    let list = storage.history(None, 10, 0).expect("history");
    assert_eq!(list.len(), 2);
  }

  #[test]
  fn image_size_cap_is_enforced() {
    let storage = test_storage();

    let payload = ImagePayload {
      width: 20,
      height: 20,
      bytes: vec![255; 20 * 20 * 4]
    };

    let inserted = storage
      .insert_image(&payload, "com.apple.Preview", "img-hash", 0, 500)
      .expect("insert image");
    assert!(!inserted);
  }
}
