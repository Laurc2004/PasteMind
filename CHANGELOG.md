# Changelog

## [0.1.1] - 2026-03-09

### Added

- GitHub Actions macOS release workflow to build and publish both `.dmg` and `.app.zip`.
- Release asset generation with SHA256 checksum and release notes attachment.

### Changed

- Updated macOS bundle identifier to `com.laurc2004.pastemind`.
- Kept local default bundle target to `app` while CI builds `app,dmg`.

## [0.1.0] - 2026-03-08

### Added

- Initial Tauri + SvelteKit clipboard app scaffold.
- Clipboard watcher for text/image capture.
- SQLite metadata store + image file storage.
- History search, delete, clear and retention policy.
- Auto paste flow with accessibility permission checks.
- Menu bar/tray interaction and global shortcut target (`Cmd+Shift+V`).
- Bilingual open-source documentation.

### Changed

- Updated UI to glass style with bilingual toggle and improved interaction flow.
- Settings panel placed below clipboard history for Win+V-like usage rhythm.
- Added clickable preview area (text/image) for direct paste action.
- Hotkey recorder now supports `F1` to `F24` (with or without modifiers).

### Fixed

- Closing main window now hides the window instead of quitting the app.
- Improved image clipboard capture reliability on macOS with native pasteboard fallback.
- Prevented duplicate history insertion during auto-paste selection flow.
