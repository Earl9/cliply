# Cliply

[English](README.md) | [Simplified Chinese](README.zh-CN.md)

Cliply is a local-first clipboard manager for Windows. It keeps clipboard
history fast, searchable, and under your control — no account required, and
clipboard contents are never sent to a Cliply-hosted cloud service.

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

**Clipboard history**

- Text, link, code snippet, and image history
- Fast search with type filters, pinned items, and detail preview
- Paste, copy, and paste as plain text back to the previous app

**Privacy and local storage**

- Local SQLite storage with configurable retention and duplicate handling
- Sensitive content such as passwords and verification codes is masked by default
- Copies from common password managers are ignored by default (configurable)

**Appearance**

- Light, dark, and system-follow themes with customizable accent colors
- Windows-friendly UI controls and system tray integration

**Sync**

- Encrypted `.cliply-sync` package import and export
- Sync through user-controlled storage: Local Folder, WebDAV, FTP, and FTPS
- Auto sync with configurable intervals and image sync modes

**Install and updates**

- Windows installer with install, update, uninstall, startup, and data-retention controls
- One-click in-app updates from the About tab: automatic download, SHA-256 verification, and Modern Installer launch

## Keyboard Shortcuts

| Shortcut | Action |
| --- | --- |
| `Ctrl + Shift + V` | Open the Cliply window (global, customizable) |
| `↑` / `↓` | Select an item |
| `Enter` | Paste the selected item |
| `Shift + Enter` | Paste as plain text |
| `Ctrl + P` | Pin or unpin the selected item |
| `Delete` | Delete the selected item |
| `Ctrl + K` | Focus the search box |
| `Esc` | Close the window |

## Privacy

Cliply is local-first by design:

- Clipboard history is stored locally on your Windows machine.
- Cliply does not require an account.
- Cliply does not provide or use a hosted cloud service for your clipboard data.
- Sensitive content such as passwords and verification codes is masked by default.
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
Modern Installer asset and includes a SHA256 checksum. In the About tab, one
click on "Update Now" downloads the update package, verifies its SHA-256
checksum, and launches the Modern Installer in update mode. Update checks can
run automatically on a daily or weekly schedule, or be triggered manually.

During installation, Cliply temporarily closes while Modern Installer replaces
program files, preserves user data, updates shortcuts, and starts Cliply again.
If automatic installation fails, download the full
`Cliply_*_x64-modern-installer.exe` from GitHub Releases and run it manually.

## Development

Prerequisites: Windows 10/11, Node.js 20.19+ or 22+, and Rust with the MSVC
toolchain.

Clone the repository:

```powershell
git clone https://github.com/Earl9/cliply.git
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

Type-check and build the frontend:

```powershell
npm run typecheck
npm run build
```

Run backend checks:

```powershell
cargo check --manifest-path .\src-tauri\Cargo.toml
```

Build the modern installer (the NSIS fallback installer can be built with
`npm run build:tauri-nsis`):

```powershell
npm run build:modern-installer
```

## Documentation

- [Privacy Policy](PRIVACY.md)
- [Security Policy](SECURITY.md)
- [Changelog](CHANGELOG.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Installer Notes](docs/installer.md)
- [Sync Design](docs/sync-design.md)
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
