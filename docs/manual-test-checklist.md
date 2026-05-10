# Manual Test Checklist

Use this checklist before publishing a beta build. Record evidence such as
screenshots, installer paths, logs, database counts, and remote directory
screenshots when possible.

## Environment

| Field | Value |
| --- | --- |
| Date |  |
| Tester |  |
| Commit |  |
| Version |  |
| Windows version |  |
| DPI scale |  |
| Multi-monitor |  |
| Installer path |  |
| WebDAV server |  |
| FTP/FTPS server |  |

## Clipboard

- [ ] Launch Cliply and confirm the main window opens quickly.
- [ ] Copy plain text and confirm it appears in history.
- [ ] Copy a URL and confirm it is classified as a link.
- [ ] Copy a code snippet and confirm code preview works.
- [ ] Copy or capture an image and confirm thumbnail/detail view works.
- [ ] Search Chinese, English, URL, and code content.
- [ ] Filter all/text/link/image/code/pinned.
- [ ] Pin and unpin an item.
- [ ] Delete an unpinned item.
- [ ] Pause and resume clipboard monitoring.
- [ ] Confirm duplicate copies update existing records instead of creating obvious duplicates.

## Paste

- [ ] Paste into Notepad using Enter or double-click.
- [ ] Paste into a browser input.
- [ ] Paste into an editor input.
- [ ] Plain-text paste does not include rich text or HTML formatting.
- [ ] Image copy/paste works when the target application supports images.
- [ ] Failure path shows a friendly message and does not leak content in logs.

## Settings

- [ ] General settings save and persist after restart.
- [ ] Shortcut conflict checks show understandable messages.
- [ ] History retention settings save and enforce limits.
- [ ] Theme and accent color changes apply consistently.
- [ ] Sync page clearly shows the active provider.
- [ ] About page shows version, paths, database size, history count, and sync status.
- [ ] Copied diagnostics do not contain clipboard body, passwords, tokens, or private keys.

## Sync

- [ ] Export encrypted `.cliply-sync` package.
- [ ] Import package with the correct password.
- [ ] Import with the wrong password fails without changing local data.
- [ ] Import a corrupted package and confirm a friendly error.
- [ ] Local Folder sync creates `CliplySync/manifest.json`, `snapshots/`, `events/`, and `devices/`.
- [ ] WebDAV sync succeeds.
- [ ] WebDAV wrong credential or bad URL fails safely.
- [ ] FTP sync succeeds.
- [ ] FTPS sync succeeds.
- [ ] FTP/FTPS wrong credential fails safely.
- [ ] Auto sync triggers after the configured interval.
- [ ] Image metadata-only mode does not upload original image blobs.
- [ ] Image compressed/original modes upload encrypted blobs and display after download.

## Installer

- [ ] Fresh install to default path.
- [ ] Fresh install to custom path.
- [ ] Update over an existing install.
- [ ] Update while Cliply is running.
- [ ] Desktop shortcut option works.
- [ ] Startup option writes/removes the startup entry.
- [ ] Uninstall preserves user data by default.
- [ ] Uninstall with delete-data option removes local data.
- [ ] Reinstall after preserved-data uninstall reuses existing data.
- [ ] Installer and uninstaller do not show stray console windows.

## Logs And Privacy

- [ ] Logs do not contain clipboard body text.
- [ ] Logs do not contain image binary/base64 payloads.
- [ ] Logs do not contain sync passwords.
- [ ] Logs do not contain provider passwords.
- [ ] Logs do not contain Authorization headers, tokens, secrets, or private keys.
- [ ] Recent error and last sync error shown in diagnostics are redacted.

## Performance

- [ ] History list loads with a large local history.
- [ ] Search remains responsive.
- [ ] Image thumbnails load asynchronously and scrolling remains usable.
- [ ] Sync import/export does not freeze the main UI.
- [ ] Startup time is acceptable on the test machine.

## DPI And Multi-Monitor

- [ ] 100% scale has no clipping or overlap.
- [ ] 125% scale has no clipping or overlap.
- [ ] 150% scale has no clipping or overlap.
- [ ] Minimum window size does not crush text or buttons.
- [ ] Hotkey opens the window on a reasonable monitor.
- [ ] Tray restore works across monitors.
