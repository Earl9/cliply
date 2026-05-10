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
- [ ] Current version lower than GitHub latest signed updater metadata discovers an update.
- [ ] Current version equal to latest signed updater metadata shows "already up to date".
- [ ] Network failure shows an inline update error.
- [ ] Update available state shows version, published time, and release notes.
- [ ] Installing an update warns that Cliply may temporarily close.
- [ ] Download progress displays normally.
- [ ] Installed update state offers immediate relaunch.
- [ ] Release contains `latest.json` and updater signature `.sig` artifacts.
- [ ] Release contains `Cliply_*_x64-modern-installer.exe` for manual installation.
- [ ] Release notes tell users to download the modern installer and treat NSIS setup as an updater asset.
- [ ] Release is not marked as a GitHub Pre-release while the updater endpoint uses `/releases/latest/download/latest.json`.
- [ ] Update flow never forces updates, silently installs in the background, uses a custom update server, accepts unsigned updates, or manually replaces the exe.
- [ ] Installer matrix passes.
- [ ] Logs and diagnostics are redacted.
- [ ] DPI and multi-monitor smoke tests pass.

## Artifacts

Expected local artifacts:

```text
apps\cliply-installer\src-tauri\target\release\cliply-modern-installer.exe
release-assets\Cliply_0.4.1-beta.1_x64-modern-installer.exe
src-tauri\target\release\bundle\nsis\Cliply_0.4.1-beta.1_x64-setup.exe
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
