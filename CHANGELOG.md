# Changelog

All notable changes to Cliply will be documented in this file.

The format follows the spirit of Keep a Changelog, and release versions follow
semantic versioning where practical during the beta period.

## Unreleased

## 0.4.1-beta.13 - 2026-08-30

### Changed

- The About-tab update flow is now one click: "Update Now" downloads the update
  package, verifies SHA-256, and launches the Modern Installer directly,
  removing the separate download, install, and confirmation steps.
- User-facing copy across settings, tray menu, context menus, dialogs, and the
  installer was standardized for clearer, more consistent wording.
- Installer retry actions and error recovery text were simplified, including a
  clearer message when program files are still in use.

### Documentation

- The README was refreshed with grouped features, a keyboard shortcut table,
  one-click update documentation, newly captured light/dark/settings
  screenshots, and verified development prerequisites.
- Removed unused design drafts, comparison screenshots, and superseded
  internal documents from the repository root; extended `.gitignore` for local
  scratch folders and release assets.

### Validation

- Main and installer frontend type checks, `vite build`, and both Rust
  `cargo check` runs pass.

## 0.4.1-beta.12 - 2026-08-14

### Changed

- Update failures now stop the progress animation and provide direct actions to
  retry the operation or close the installer.
- Installer error text now describes the recovery action without exposing a
  long operating-system error sentence in the primary interface.

### Fixed

- The application now arms a bounded exit watchdog before handing control to
  the update installer, preventing a stalled background sync operation from
  keeping `cliply.exe` open indefinitely.
- The update installer waits for the launching process, terminates its process
  tree when graceful shutdown exceeds the limit, and verifies that it exited.
- Program files are replaced atomically with the Windows file API and retry
  transient sharing locks for up to 30 seconds, avoiding the `os error 32`
  failure observed while security software briefly scans the executable.

### Validation

- Added a Windows regression test that holds the installed executable with a
  delete-denying file lock and confirms replacement succeeds after release.
- Main and installer frontend builds, both Rust checks, the installer lock
  regression test, and the complete 54-test Rust suite pass.

## 0.4.1-beta.11 - 2026-08-14

### Added

- Open Design source artifacts and a maintained Cliply design system document.
- Sync queue abandonment, checkpoint tracking, tombstone cleanup, and database
  maintenance services with migration and regression tests.
- A custom overlay scrollbar, development-only 1,000-record stress data, and
  dedicated performance and engineering optimization documents.
- Shared artwork generation for application, tray, NSIS, and modern installer
  icons and surfaces.

### Changed

- Rebuilt the main interface as a compact, sidebar-free Windows workbench with
  responsive two-pane layout, neutral Acrylic surfaces, and concise desktop
  copy.
- Rebuilt the modern installer and refreshed NSIS artwork, wording, sizing,
  shortcuts, update behavior, and uninstall data-retention controls.
- Clipboard additions, pinning, deletion, copying, and pasting now update the
  visible list incrementally instead of unconditionally reloading all records.
- Sync package formats and tags are loaded in batches, and the automatic sync
  scheduler now sleeps on an event-driven condition variable.
- Settings and About dialogs are loaded as separate frontend chunks.

### Fixed

- Minimum-window and high-density layouts no longer leave excessive vertical
  whitespace or clip the primary paste action.
- Installer and application icons use optically enlarged small-size artwork so
  they match neighboring Windows icons.
- Sync cleanup preserves pending or abandoned tombstones and removes only
  fully synchronized records after the configured retention period.
- Update, startup, clipboard, paste, FTP/FTPS, and diagnostic messages use
  consistent production wording and redaction rules.

### Performance

- Virtualized the clipboard history list; 1,000 records keep roughly 15–24
  row nodes in the DOM.
- Continuous 12,000 px scrolling measured approximately 59.99 FPS with no
  observed long tasks in the final 60 Hz validation run.
- Removed repeated large-area CSS blur layers while retaining native Windows
  Acrylic and a single small search-control blur.
- Reduced the main JavaScript bundle to approximately 298.63 KB, or 89.83 KB
  gzip.

### Validation

- TypeScript checks, production builds, Cargo Check, Clippy, the modern
  installer build, and 54 Rust tests pass; one real FTP integration test remains
  intentionally ignored unless test-server credentials are configured.

## 0.4.0-beta.1 - 2026-05-09

### Added

- Text, link, code, and image clipboard history.
- Search, filtering, pinning, deletion, detail preview, and keyboard actions.
- Plain-text paste and automatic paste back to the previous Windows target.
- Local SQLite storage and image thumbnail/blob handling.
- Encrypted sync package import/export.
- Local Folder, WebDAV, FTP, and FTPS remote sync providers.
- Auto sync configuration and saved local sync password support.
- Modern Windows installer and fallback NSIS installer.
- Settings UI for general behavior, shortcuts, history, appearance, sync, and diagnostics.

### Security

- Log and diagnostic redaction for clipboard body fields, passwords,
  Authorization headers, tokens, private keys, encrypted payload fields, and
  large secret-like blobs.
- Provider passwords are no longer returned to the frontend in remote sync status.

### Performance

- Added `clipboard_formats` indexes used by list/search/detail queries.
- Verified a 1000-record local performance smoke test.

### Known Manual Release Blockers

- Fresh install, update install, custom path install, uninstall, and data retention matrix.
- Real WebDAV, FTP, and FTPS success/failure paths.
- Automatic paste into real Windows applications.
- DPI and multi-monitor validation.
