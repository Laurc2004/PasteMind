use std::{
  collections::VecDeque,
  sync::Arc,
  time::{Duration, Instant}
};

use parking_lot::RwLock;

use crate::{models::Settings, storage::Storage};

#[derive(Clone)]
pub struct AppState {
  pub storage: Arc<Storage>,
  pub settings: Arc<RwLock<Settings>>,
  pub last_foreground_bundle: Arc<RwLock<Option<String>>>,
  pub hotkey_recording: Arc<RwLock<bool>>,
  pub suppressed_hashes: Arc<RwLock<VecDeque<(String, Instant)>>>
}

impl AppState {
  pub fn new(storage: Arc<Storage>, settings: Settings) -> Self {
    Self {
      storage,
      settings: Arc::new(RwLock::new(settings)),
      last_foreground_bundle: Arc::new(RwLock::new(None)),
      hotkey_recording: Arc::new(RwLock::new(false)),
      suppressed_hashes: Arc::new(RwLock::new(VecDeque::new()))
    }
  }

  pub fn settings_snapshot(&self) -> Settings {
    self.settings.read().clone()
  }

  pub fn replace_settings(&self, settings: Settings) {
    *self.settings.write() = settings;
  }

  pub fn set_last_foreground_bundle(&self, bundle: Option<String>) {
    *self.last_foreground_bundle.write() = bundle;
  }

  pub fn last_foreground_bundle(&self) -> Option<String> {
    self.last_foreground_bundle.read().clone()
  }

  pub fn set_hotkey_recording(&self, value: bool) {
    *self.hotkey_recording.write() = value;
  }

  pub fn is_hotkey_recording(&self) -> bool {
    *self.hotkey_recording.read()
  }

  pub fn suppress_history_hash(&self, hash: String) {
    const SUPPRESS_WINDOW: Duration = Duration::from_secs(2);

    let mut suppressed = self.suppressed_hashes.write();
    let now = Instant::now();
    while matches!(suppressed.front(), Some((_, expire_at)) if *expire_at <= now) {
      suppressed.pop_front();
    }

    if let Some((_, expire_at)) = suppressed.iter_mut().find(|(candidate, _)| *candidate == hash) {
      *expire_at = now + SUPPRESS_WINDOW;
    } else {
      suppressed.push_back((hash, now + SUPPRESS_WINDOW));
    }
  }

  pub fn consume_suppressed_hash(&self, hash: &str) -> bool {
    let mut suppressed = self.suppressed_hashes.write();
    let now = Instant::now();

    while matches!(suppressed.front(), Some((_, expire_at)) if *expire_at <= now) {
      suppressed.pop_front();
    }

    suppressed
      .iter()
      .any(|(candidate, expire_at)| candidate == hash && *expire_at > now)
  }
}
