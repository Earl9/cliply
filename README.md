# Cliply

[English](README.md) | [Simplified Chinese](README.zh-CN.md)

Cliply is a local-first clipboard manager for Windows. It keeps clipboard
history fast, searchable, and under your control without requiring an account
or sending clipboard contents to a Cliply-hosted cloud service.

> Status: `v0.4.0-beta.1` is in beta stabilization. Core workflows are
> implemented and the project is being prepared for public beta validation.
> See the [release checklist](docs/release-checklist.md) for the current
> pre-release verification plan.

## Screenshots

| Main Window (Light) | Main Window (Dark) |
| --- | --- |
| ![Cliply main window in light mode](docs/assets/screenshots/main-light.png) | ![Cliply main window in dark mode](docs/assets/screenshots/main-dark.png) |

| Sync Settings | Installer |
| --- | --- |
| ![Cliply sync settings](docs/assets/screenshots/settings-sync.png) | ![Cliply installer](docs/assets/screenshots/installer.png) |

## Features

- Clipboard history for text, links, code snippets, and images
- Fast search with type filters, pinned items, deletion, and detail preview
- Paste, copy, plain-text paste, and automatic paste back to the previous app
- Local SQLite storage with configurable retention and duplicate handling
- Image thumbnails with local image blob storage
- Customizable shortcuts, startup behavior, and paste behavior
- Light/dark themes, accent colors, and Windows-friendly UI controls
- Encrypted `.cliply-sync` import/export packages
- Sync through user-controlled storage: Local Folder, WebDAV, FTP, and FTPS
- Auto sync with configurable intervals and image sync modes
- Windows installer with install, update, uninstall, startup, and data-retention controls

## Privacy

Cliply is local-first by design:

- Clipboard history is stored locally on your Windows machine.
- Cliply does not require an account.
- Cliply does not provide or use a hosted cloud service for your clipboard data.
- Sync packages are encrypted before they are written to disk or uploaded to a
  provider you configure.
- Remote sync providers receive encrypted sync packages, not plaintext
  clipboard history.
- Logs and diagnostics must not contain clipboard body text, sync passwords,
  provider passwords, tokens, Authorization headers, private keys, or image
  contents.

Default Windows data location:

```text
%APPDATA%\com.cliply.app\
```

For more detail, see [PRIVACY.md](PRIVACY.md) and
[docs/privacy-and-logs.md](docs/privacy-and-logs.md).

## Security

Security-sensitive areas include clipboard capture, paste behavior, sync
package encryption, remote provider authentication, diagnostics, and installer
upgrade/uninstall flows.

Please do not paste production secrets into public issues. If you discover a
security or privacy issue, follow [SECURITY.md](SECURITY.md).

## Current Release Readiness

Cliply is currently in `v0.4.0-beta.1` stabilization. The detailed pre-release
validation plan lives in [docs/release-checklist.md](docs/release-checklist.md).

## Development

Clone the repository:

```powershell
git clone https://github.com/<owner>/cliply.git
cd cliply
```

Install dependencies:

```powershell
npm install
```

Run the desktop app in development:

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

## Testing

Common verification commands:

```powershell
npm run lint
npm run typecheck
npm run build
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
```

Manual release validation is tracked in
[docs/release-checklist.md](docs/release-checklist.md). Sync provider details
are documented in [docs/provider-testing.md](docs/provider-testing.md) and
[docs/sync-design.md](docs/sync-design.md).

## Documentation

- [Privacy Policy](PRIVACY.md)
- [Security Policy](SECURITY.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Roadmap](ROADMAP.md)
- [Changelog](CHANGELOG.md)
- [Manual Test Checklist](docs/manual-test-checklist.md)
- [Release Checklist](docs/release-checklist.md)
- [Sync Design](docs/sync-design.md)
- [Provider Testing](docs/provider-testing.md)
- [Installer Notes](docs/installer.md)
- [Privacy And Logs](docs/privacy-and-logs.md)

## Tech Stack

- Desktop shell: Tauri v2
- Frontend: React, TypeScript, Vite, Tailwind CSS
- Backend: Rust
- Storage: SQLite via `rusqlite`
- Sync crypto: AES-GCM with Argon2 key derivation
- Installer: Tauri app-based modern installer plus NSIS fallback

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md)
before opening a pull request, keep changes focused, and avoid committing
generated build outputs or local runtime data.

## License

Cliply is licensed under the [MIT License](LICENSE).
