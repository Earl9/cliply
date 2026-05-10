# Cliply

[English](README.md) | [Simplified Chinese](README.zh-CN.md)

Cliply is a local-first clipboard manager for Windows. It keeps clipboard
history fast, searchable, and under your control without requiring an account
or sending clipboard contents to a Cliply-hosted cloud service.

Status: Beta. Cliply is Windows-first and currently focused on stabilization,
installer validation, and sync reliability.

## Screenshots

### Main Window

![Cliply main window](docs/assets/screenshots/main-light.png)

### Dark Mode

![Cliply dark mode](docs/assets/screenshots/main-dark.png)

### Settings

![Cliply settings](docs/assets/screenshots/settings-sync.png)

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
- Signed update checks from the About tab with a Modern Installer update flow

## Privacy

Cliply is local-first by design:

- Clipboard history is stored locally on your Windows machine.
- Cliply does not require an account.
- Cliply does not provide or use a hosted cloud service for your clipboard data.
- Sync packages are encrypted before they are written to disk or uploaded to a
  provider you configure.
- Remote sync providers receive encrypted sync packages, not plaintext
  clipboard history.
- Update checks contact GitHub Releases for update metadata and do
  not include clipboard history, sync passwords, or local database content.
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

## Updates

Cliply checks GitHub Releases for `latest.json`. The manifest points to the
Modern Installer asset and includes a SHA256 checksum. The About tab shows
update availability and download progress, then Cliply verifies the installer
before launching Modern Installer in update mode.

During installation, Cliply temporarily closes while Modern Installer replaces
program files, preserves user data, updates shortcuts, and starts Cliply again.
If automatic installation fails, download the full
`Cliply_*_x64-modern-installer.exe` from GitHub Releases and run it manually.

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
```

Build the modern installer:

```powershell
npm run build:modern-installer
```

## Documentation

- [Privacy Policy](PRIVACY.md)
- [Security Policy](SECURITY.md)
- [Changelog](CHANGELOG.md)
- [Sync Design](docs/sync-design.md)
- [Installer Notes](docs/installer.md)
- [Privacy And Logs](docs/privacy-and-logs.md)

## Tech Stack

- Desktop shell: Tauri v2
- Frontend: React, TypeScript, Vite, Tailwind CSS
- Backend: Rust
- Storage: SQLite via `rusqlite`
- Sync crypto: AES-GCM with Argon2 key derivation
- Installer: Tauri app-based modern installer plus NSIS fallback

## License

Cliply is licensed under the [MIT License](LICENSE).
