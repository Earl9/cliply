# Cliply

[English](README.md) | [简体中文](README.zh-CN.md)

Cliply is a local-first clipboard manager for Windows, built with Tauri v2,
React, TypeScript, Vite, Tailwind CSS, SQLite, and Rust.

It is designed for people who want fast clipboard history without accounts,
cloud lock-in, or sending clipboard contents to a hosted service.

> Status: `v0.4.0-beta.1` stabilization. Core features are implemented, but
> real installer, WebDAV/FTP/FTPS, paste, DPI, and multi-monitor validation are
> still required before a beta release is marked ready.

![Cliply text history](cliply-ui-after-text.png)

## Features

- Text, link, code, and image clipboard history
- Search, type filters, pinning, deletion, and detail preview
- Plain-text paste and automatic paste back to the previous target window
- Local SQLite storage
- Image thumbnails and local image blob storage
- Sensitive-content detection for passwords, tokens, private keys, and codes
- Configurable retention, duplicate handling, startup behavior, and shortcuts
- Theme settings with accent color presets
- Encrypted `.cliply-sync` export/import packages
- Remote sync provider architecture
- Local Folder, WebDAV, FTP, and FTPS sync providers
- Auto sync with saved local sync password support
- Modern Windows installer and fallback NSIS installer
- Log and diagnostic redaction for clipboard body, passwords, tokens, and large payloads

## Privacy Model

Cliply is local-first by design:

- Clipboard history is stored under the local app data directory.
- There is no built-in account system.
- There is no hosted Cliply cloud service.
- Sync packages are encrypted before export.
- Remote sync providers receive encrypted sync packages; image sync blobs are
  encrypted before upload when remote blob sync is used.
- Logs should contain operational metadata only, not clipboard contents or
  secrets.

Default Windows data location:

```text
%APPDATA%\com.cliply.app\
```

## Current Release Readiness

The latest stabilization report is tracked in:

```text
docs/v0.4.0-beta.1-stabilization-test-results.md
```

Current automated checks pass:

- Frontend production build
- Rust `cargo check`
- Rust unit tests
- Modern installer build
- Log redaction scan
- 1000-row local performance smoke test

Manual release blockers still to verify:

- Fresh install, update install, custom install path, uninstall, and data retention
- Real WebDAV, FTP, and FTPS success/failure paths
- Automatic paste into real Windows target applications
- DPI and multi-monitor behavior
- Final log sampling after manual workflows

## Tech Stack

- Desktop shell: Tauri v2
- Frontend: React, TypeScript, Vite, Tailwind CSS
- Icons: lucide-react
- Backend: Rust
- Storage: SQLite via `rusqlite`
- Sync crypto: AES-GCM with Argon2 key derivation
- Installer: Tauri app-based modern installer plus NSIS fallback

## Requirements

- Windows 10/11 for the full desktop experience
- Node.js and npm
- Rust stable toolchain
- Tauri v2 prerequisites for Windows
- NSIS is downloaded/used by Tauri when building the NSIS bundle

## Getting Started

Install dependencies:

```powershell
npm install
```

Run the frontend only:

```powershell
npm run dev
```

Run the Tauri desktop app:

```powershell
npm run tauri dev
```

Build the frontend:

```powershell
npm run build
```

Run backend checks:

```powershell
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
```

Build the modern installer:

```powershell
npm run build:modern-installer
```

Modern installer output:

```text
apps\cliply-installer\src-tauri\target\release\cliply-modern-installer.exe
```

Fallback NSIS output:

```text
src-tauri\target\release\bundle\nsis\Cliply_0.4.0-beta.1_x64-setup.exe
```

## Project Structure

```text
apps/cliply-installer/      Modern installer app
docs/                       Manual test plans and stabilization reports
scripts/                    Build and packaging scripts
src/                        React frontend
src-tauri/                  Rust backend and Tauri configuration
src-tauri/src/commands/     Tauri command handlers
src-tauri/src/db/           SQLite migrations
src-tauri/src/platform/     Platform adapters
src-tauri/src/services/     Clipboard, sync, settings, logging, and paste services
```

## Testing

Common verification commands:

```powershell
npm run build
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run build:modern-installer
```

Targeted sync tests:

```powershell
cargo test --manifest-path .\src-tauri\Cargo.toml local_folder -- --nocapture
cargo test --manifest-path .\src-tauri\Cargo.toml sync_blob -- --nocapture
cargo test --manifest-path .\src-tauri\Cargo.toml sync_crypto -- --nocapture
```

The real FTP roundtrip test is ignored by default. To run it, provide:

```text
CLIPLY_TEST_FTP_HOST
CLIPLY_TEST_FTP_PORT
CLIPLY_TEST_FTP_USER
CLIPLY_TEST_FTP_PASSWORD
CLIPLY_TEST_FTP_SECURE
CLIPLY_TEST_FTP_REMOTE_PATH
```

## Release Checklist

Before tagging a beta release:

- Run all automated checks listed above.
- Build the modern installer.
- Verify the installer matrix:
  - fresh install
  - update install
  - custom path install
  - uninstall preserving user data
  - uninstall deleting user data
  - startup entry behavior
- Verify WebDAV, FTP, and FTPS in real environments.
- Verify automatic paste with Notepad and at least one browser or editor input.
- Re-run log redaction sampling after manual workflows.
- Update the stabilization report in `docs/`.
- Sign release binaries before public distribution when possible.

## Security

Please do not paste sensitive production secrets into public issues. If you
discover a vulnerability or a privacy leak, open a private report with the
maintainers before publishing details.

Security-sensitive areas:

- Clipboard capture and paste logic
- Log and diagnostic redaction
- Sync package encryption/decryption
- Remote provider authentication
- Installer and update behavior

## Contributing

Contributions are welcome once the repository is prepared for public intake.
For now:

1. Open an issue describing the bug or improvement.
2. Keep changes focused.
3. Add or update tests for backend behavior.
4. Run the verification commands before submitting a pull request.
5. Do not commit generated build outputs such as `dist/`, `target/`, installer
   executables, payload archives, or `node_modules/`.

## Roadmap

Near-term:

- Finish `v0.4.0-beta.1` release validation
- Complete real WebDAV/FTP/FTPS matrix testing
- Harden installer upgrade/uninstall workflows
- Expand regression tests for paste and sync failure paths

Later:

- More polished release automation
- Signed Windows builds
- Additional sync-provider hardening
- Better multi-device conflict reporting

## License

No license has been declared yet. Add a `LICENSE` file before publishing this
repository as open source; without a license, others do not have clear legal
permission to use, modify, or redistribute the code.
