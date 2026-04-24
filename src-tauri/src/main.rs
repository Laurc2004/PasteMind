#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use pastemind::{
  app_state::AppState, clipboard_watcher, commands,
  error::{AppError, AppResult},
  hotkey,
  permissions,
  settings,
  storage::Storage
};
use tauri::{
  image::Image,
  menu::{Menu, MenuItem},
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
  AppHandle, Emitter, Manager, WindowEvent
};
use tauri_plugin_global_shortcut::ShortcutState;

fn main() {
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

  tauri::Builder::default()
    .plugin(
      tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, shortcut, event| {
          if let Some(state) = app.try_state::<Arc<AppState>>() {
            if state.is_hotkey_recording() {
              return;
            }
          }

          if hotkey::matches_active_hotkey(app, shortcut) && event.state() == ShortcutState::Pressed {
            let _ = toggle_main_window(app);
          }
        })
        .build()
    )
    .on_window_event(|window, event| {
      if window.label() == "main" {
        if let WindowEvent::CloseRequested { api, .. } = event {
          api.prevent_close();
          let _ = window.hide();
        }
      }
    })
    .setup(|app| {
      let state = initialize_state(app)?;
      app.manage(state.clone());

      setup_tray(app)?;
      register_hotkey(app, &state)?;
      clipboard_watcher::start(app.handle().clone(), state.clone());
      permissions::start_permission_watcher(app.handle().clone());
      let _ = app.emit("permission:changed", permissions::permission_state());

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::get_history,
      commands::select_entry,
      commands::delete_entry,
      commands::clear_history,
      commands::get_settings,
      commands::update_settings,
      commands::get_permission_state,
      commands::request_accessibility_permission,
      commands::open_accessibility_settings,
      commands::show_main_window,
      commands::set_hotkey_recording
    ])
    .run(tauri::generate_context!())
    .expect("failed to run PasteMind");
}

fn register_hotkey(app: &tauri::App, state: &Arc<AppState>) -> AppResult<()> {
  let configured = state.settings_snapshot().hotkey;
  let normalized = hotkey::normalize_hotkey(&configured);

  match hotkey::apply_hotkey(app.handle(), &normalized) {
    Ok(_) => Ok(()),
    Err(err) => {
      log::warn!(
        "failed to register configured hotkey '{}': {}. Falling back to {}",
        configured,
        err,
        hotkey::DEFAULT_HOTKEY
      );

      hotkey::apply_hotkey(app.handle(), hotkey::DEFAULT_HOTKEY)?;

      let mut next = state.settings_snapshot();
      next.hotkey = hotkey::DEFAULT_HOTKEY.to_string();
      state.storage.save_settings(&next)?;
      state.replace_settings(next);
      Ok(())
    }
  }
}

fn setup_tray(app: &tauri::App) -> AppResult<()> {
  let toggle = MenuItem::with_id(app, "toggle", "Show / Hide", true, None::<&str>)?;
  let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
  let menu = Menu::with_items(app, &[&toggle, &quit])?;

  let tray_icon = build_tray_icon();

  TrayIconBuilder::with_id("main-tray")
    .tooltip("PasteMind")
    .icon(tray_icon)
    .menu(&menu)
    .on_menu_event(|app, event| match event.id().as_ref() {
      "toggle" => {
        let _ = toggle_main_window(app);
      }
      "quit" => app.exit(0),
      _ => {}
    })
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        let _ = toggle_main_window(tray.app_handle());
      }
    })
    .build(app)?;

  Ok(())
}

fn build_tray_icon() -> Image<'static> {
  let width = 24u32;
  let height = 24u32;
  let mut rgba = vec![0u8; (width * height * 4) as usize];

  let set_px = |buf: &mut Vec<u8>, x: u32, y: u32, color: [u8; 4]| {
    let index = ((y * width + x) * 4) as usize;
    buf[index..index + 4].copy_from_slice(&color);
  };

  // Letter P — monochrome white silhouette
  // P vertical stem
  for y in 6..19 {
    for x in 8..11 {
      set_px(&mut rgba, x as u32, y as u32, [255, 255, 255, 255]);
    }
  }
  // P — top bar
  for y in 6..9 {
    for x in 8..17 {
      set_px(&mut rgba, x as u32, y as u32, [255, 255, 255, 255]);
    }
  }
  // P — right vertical (bowl)
  for y in 6..14 {
    for x in 14..17 {
      set_px(&mut rgba, x as u32, y as u32, [255, 255, 255, 255]);
    }
  }
  // P — bottom bar (bowl closure)
  for y in 11..14 {
    for x in 8..17 {
      set_px(&mut rgba, x as u32, y as u32, [255, 255, 255, 255]);
    }
  }

  Image::new_owned(rgba, width, height)
}

fn initialize_state(app: &tauri::App) -> AppResult<Arc<AppState>> {
  let data_dir = app
    .path()
    .app_data_dir()
    .map_err(|err| AppError::Other(format!("failed to resolve app data dir: {err}")))?;

  std::fs::create_dir_all(&data_dir)?;

  let storage = Arc::new(Storage::new(
    data_dir.join("pastemind.db"),
    data_dir.join("images")
  )?);
  let settings = settings::load_or_default(&storage)?;

  Ok(Arc::new(AppState::new(storage, settings)))
}

fn toggle_main_window(app: &AppHandle) -> AppResult<()> {
  let window = app
    .get_webview_window("main")
    .ok_or_else(|| AppError::WindowNotFound("main".to_string()))?;

  if window.is_visible().unwrap_or(false) {
    window.hide()?;
  } else {
    remember_foreground_app(app);
    window.show()?;
    window.unminimize()?;
    window.set_focus()?;
  }

  Ok(())
}

fn remember_foreground_app(app: &AppHandle) {
  let Some(state) = app.try_state::<Arc<AppState>>() else {
    return;
  };

  let own_identifier = app.config().identifier.clone();
  if let Some(focused) = permissions::frontmost_bundle_id().filter(|bundle| bundle != &own_identifier) {
    state.set_last_foreground_bundle(Some(focused));
  }
}
