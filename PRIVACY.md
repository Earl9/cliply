# Privacy Policy

Cliply is designed as a local-first clipboard manager. This document explains
what data Cliply stores, where it is stored, and what may leave your machine.

## Summary

- Cliply does not include a hosted cloud service.
- Cliply does not include an account system.
- Clipboard history is stored locally by default.
- Sync packages are encrypted before export or upload to a configured remote provider.
- Logs are intended to contain operational metadata only, not clipboard contents.

## Local Data

Cliply stores local application data in the operating system app data directory.
On Windows, the default location is:

```text
%APPDATA%\com.cliply.app\
```

Local data can include:

- SQLite database files
- Clipboard item metadata and text history
- Image files and thumbnails
- Sync state and device metadata
- Application settings
- Operational logs

## Clipboard Contents

Clipboard history can include text, links, code, and images copied by the user.
This data remains local unless the user explicitly exports or syncs it.

Cliply includes sensitive-content detection for common secrets such as
passwords, API keys, tokens, private keys, and one-time codes. Sensitive
detection is a safety feature, not a guarantee that all secrets will be
identified.

## Sync

Cliply supports encrypted sync packages and remote providers such as Local
Folder, WebDAV, FTP, and FTPS.

When sync is configured:

- Sync package payloads are encrypted before being written to a remote provider.
- Remote provider credentials are stored locally.
- Saved sync passwords are protected with platform secure storage when available.
- Provider passwords should not be returned to the frontend as plaintext.

The remote provider is chosen and configured by the user. Review the privacy and
security practices of that provider before syncing.

## Logs And Diagnostics

Cliply logs operational events such as command names, item IDs, sync counts, and
error categories. Logs should not contain:

- Clipboard body text
- Image binary data or base64 payloads
- Sync passwords
- Provider passwords
- Authorization headers
- Tokens
- Private keys

Diagnostic information may include local paths, database size, history counts,
sync status, and sanitized recent errors.

## Uninstall And Data Removal

The installer/uninstaller is designed to preserve user data by default. If the
user chooses to delete local data during uninstall, Cliply removes its local app
data directories.

Before public release, uninstall data retention behavior must be manually
validated on Windows.

## Contact

If you find a privacy issue, report it privately before publishing details. See
`SECURITY.md`.
