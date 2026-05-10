# Changelog

All notable changes to Cliply will be documented in this file.

The format follows the spirit of Keep a Changelog, and release versions follow
semantic versioning where practical during the beta period.

## Unreleased

### Added

- GitHub-ready project documentation, issue templates, pull request template,
  and baseline CI workflow.
- Release and manual testing documents under `docs/`.

### Changed

- README now describes the current `v0.4.0-beta.1` stabilization state instead
  of the early MVP scaffold.

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
