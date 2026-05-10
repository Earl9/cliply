# Provider Testing

This document tracks manual and automated checks for Cliply sync providers.
Keep real server addresses, usernames, passwords, tokens, and private paths out
of commits, screenshots, logs, and public issues.

## Local Folder

- Export to a local sync folder.
- Import from the same folder into a clean profile.
- Confirm `manifest.json`, `snapshots/`, `events/`, `devices/`, and `blobs/`
  are created as expected.
- Confirm wrong sync passwords and corrupted packages fail without changing
  local history.

## WebDAV

- Save a WebDAV provider with a valid HTTPS URL, username, password, and remote
  path.
- Run export, import, and immediate sync.
- Confirm automatic sync works with a saved sync password.
- Confirm bad URLs, missing fields, and wrong credentials fail with sanitized
  errors.
- Confirm logs do not include WebDAV passwords, Authorization headers, tokens,
  private server details, clipboard body text, or payload contents.

## FTP And FTPS

- Save an FTP provider with host, port, username, password, secure mode, and
  remote path.
- Run the matrix for FTP and FTPS separately.
- Verify export, import, immediate sync, and automatic sync.
- Confirm wrong credentials and unreachable hosts fail safely.
- Confirm logs do not include FTP passwords, server credentials, clipboard body
  text, image content, or raw sync payloads.

## Optional Real FTP Test

The real FTP roundtrip test is ignored by default. To run it locally, provide
environment variables for a disposable test server:

```text
CLIPLY_TEST_FTP_HOST
CLIPLY_TEST_FTP_PORT
CLIPLY_TEST_FTP_USER
CLIPLY_TEST_FTP_PASSWORD
CLIPLY_TEST_FTP_SECURE
CLIPLY_TEST_FTP_REMOTE_PATH
```

Then run:

```powershell
cargo test --manifest-path .\src-tauri\Cargo.toml ftp -- --ignored --nocapture
```

Use a dedicated test account and delete the remote test directory after the
run.

## Related Checks

- [Sync Design](sync-design.md)
- [Manual Test Checklist](manual-test-checklist.md)
- [Release Checklist](release-checklist.md)
- [Privacy And Logs](privacy-and-logs.md)
