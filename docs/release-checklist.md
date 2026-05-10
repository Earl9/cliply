# Release Checklist

Use this checklist for GitHub releases and beta builds.

## Pre-Release

- [ ] Confirm the release version in:
  - `package.json`
  - `package-lock.json`
  - `src-tauri/Cargo.toml`
  - `src-tauri/tauri.conf.json`
  - `apps/cliply-installer/package.json`
  - `apps/cliply-installer/src-tauri/Cargo.toml`
  - `apps/cliply-installer/src-tauri/tauri.conf.json`
- [ ] Update `CHANGELOG.md`.
- [ ] Update README release status.
- [ ] Update docs/manual-test-checklist.md results or stabilization report.
- [ ] Confirm no generated artifacts are staged.
- [ ] Confirm no local databases, logs, sync packages, certificates, or secrets are staged.

## Automated Checks

```powershell
npm install
npm run build
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run build:modern-installer
```

## Manual Checks

- [ ] Clipboard core workflows pass.
- [ ] Paste workflows pass.
- [ ] Sync package import/export pass.
- [ ] WebDAV pass/fail paths pass.
- [ ] FTP pass/fail paths pass.
- [ ] FTPS pass/fail paths pass.
- [ ] About > Update shows current version, channel, last checked time, and manual check status.
- [ ] Manual update check handles latest/no-update, update-available, and network failure states.
- [ ] Update available state shows version, published time, release notes summary, and GitHub actions.
- [ ] "View Release" opens the GitHub Release page.
- [ ] "Download Update" opens the matched installer asset URL, or falls back to the GitHub Release page.
- [ ] Update checks never silently download, replace the executable, restart the app, or change installer logic.
- [ ] Automatic update check starts after launch delay, does not block startup, and runs at most once per day for the daily interval.
- [ ] Failed automatic update checks do not show a strong popup; status remains visible from About or logs.
- [ ] Update logs contain only `update_check_started`, `update_check_success`, `update_available`, or `update_check_failed` with non-sensitive metadata.
- [ ] Installer matrix passes.
- [ ] Logs and diagnostics are redacted.
- [ ] DPI and multi-monitor smoke tests pass.

## Artifacts

Expected local artifacts:

```text
apps\cliply-installer\src-tauri\target\release\cliply-modern-installer.exe
src-tauri\target\release\bundle\nsis\Cliply_0.4.0-beta.1_x64-setup.exe
```

For each release artifact:

- [ ] Record SHA256 checksum.
- [ ] Confirm Authenticode signature status.
- [ ] Run a smoke install on Windows.
- [ ] Upload only final release artifacts to GitHub Releases.

## GitHub Release Notes

Include:

- Version
- Release date
- Highlights
- Known manual validation status
- Checksums
- Installation notes
- Privacy/security notes
- Upgrade notes

## Final Gate

- [ ] No P0 release blockers remain.
- [ ] P1 issues are fixed or documented.
- [ ] Maintainer has approved release.
