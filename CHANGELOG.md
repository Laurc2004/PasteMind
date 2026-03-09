# Changelog

## [0.1.5] - 2026-03-09

### Fixed

- Prevented `tauri build` from reading Apple signing/notarization environment variables too early in CI.
- Scoped Apple credentials to post-build signing/notarization steps, fixing `v0.1.4` release workflow failure.

## [0.1.4] - 2026-03-09

### Fixed

- Reworked macOS release pipeline to explicitly sign the generated `.app` before packaging, preventing malformed bundle signatures.
- Added optional Developer ID signing + notarization path in CI when Apple credentials are configured.
- Rebuilt release `.dmg` from the signed app in CI instead of relying on unsigned intermediate output.

## [0.1.3] - 2026-03-09

### Fixed

- Made CI release asset collection robust to dynamic bundle filename outputs.
- Added explicit app/dmg output discovery and failure diagnostics in release workflow.

## [0.1.2] - 2026-03-09

### Fixed

- Fixed GitHub Actions release workflow YAML parsing issue.
- Restored automatic macOS release pipeline trigger on tag push.

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
