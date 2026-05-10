# Installer Notes

Cliply includes a modern installer app and a fallback NSIS installer.

## Outputs

Modern installer:

```text
apps\cliply-installer\src-tauri\target\release\cliply-modern-installer.exe
GitHub Release: Cliply_0.4.1-beta.1_x64-modern-installer.exe
```

Signed NSIS fallback asset:

```text
src-tauri\target\release\bundle\nsis\Cliply_0.4.1-beta.1_x64-setup.exe
```

## Build Command

```powershell
npm run build:modern-installer
```

This command:

1. Builds the main Tauri app.
2. Builds the NSIS fallback bundle.
3. Prepares the modern installer payload.
4. Builds the modern installer app.

## Install Behavior

The modern installer:

- Requires administrator privileges.
- Installs the main `cliply.exe` into the selected install directory.
- Writes install information to the registry.
- Creates Start Menu shortcuts.
- Can create or remove a Desktop shortcut.
- Can create or remove a startup entry.
- Attempts to stop a running Cliply process before replacing files.
- Preserves user data during install and update.

The GitHub Release also includes the Tauri-generated NSIS setup and `.sig`
files as a fallback. The in-app update path reads `latest.json`, downloads the
Modern Installer asset, verifies its SHA256 checksum, and launches the Modern
Installer in update mode.

## Update Manifest

`latest.json` keeps the Tauri updater fields and adds a `modernInstaller`
object:

```json
{
  "version": "0.4.1-beta.1",
  "notes": "Release notes",
  "published_at": "2026-05-10T00:00:00Z",
  "modernInstaller": {
    "version": "0.4.1-beta.1",
    "notes": "Release notes",
    "published_at": "2026-05-10T00:00:00Z",
    "name": "Cliply_0.4.1-beta.1_x64-modern-installer.exe",
    "url": "https://github.com/Earl9/cliply/releases/download/v0.4.1-beta.1/Cliply_0.4.1-beta.1_x64-modern-installer.exe",
    "sha256": "<64 hex characters>"
  }
}
```

Do not put signed URLs with sensitive query parameters in the manifest. If a
future provider requires signed URLs, logs must redact query strings.

## In-App Update Install UX

The About tab is the primary update experience:

- Check for updates.
- Read GitHub Release `latest.json`.
- Show the release metadata.
- Show download progress in Cliply.
- Verify the downloaded Modern Installer SHA256.
- Show a preparation state after the installer has downloaded.
- Tell the user that Cliply will temporarily close during installation.

After checksum verification, Cliply starts:

```text
cliply-modern-installer.exe --mode update --install-dir "<current install dir>" --source-version "<current>" --target-version "<latest>" --preserve-user-data --launch-after-install --parent-pid <pid>
```

Argument meanings:

- `--mode update`: run the modern update UI instead of the fresh install setup.
- `--install-dir`: overwrite the current Cliply install directory.
- `--source-version`: version currently running.
- `--target-version`: version being installed.
- `--preserve-user-data`: never delete local history or settings during update.
- `--launch-after-install`: start Cliply after a successful update.
- `--parent-pid`: wait briefly for the launching Cliply process to exit before replacing files.

Modern Installer update mode:

- Shows "正在更新 Cliply".
- Waits for the parent Cliply process to exit.
- Closes any remaining running `cliply.exe`.
- Preserves `%APPDATA%\com.cliply.app` and `%LOCALAPPDATA%\com.cliply.app`.
- Overwrites program files from the embedded payload.
- Updates registry information and shortcuts.
- Starts Cliply after a successful update.

The NSIS setup is retained only as a fallback for manual installation or
recovery. It is not the primary in-app update UI.

If automatic installation fails, the UI should offer the GitHub Release page so
users can download `Cliply_*_x64-modern-installer.exe` and run it manually.

## Uninstall Behavior

The uninstaller:

- Stops the running Cliply process.
- Removes program files.
- Removes shortcuts.
- Removes startup entry.
- Removes install registry entries.
- Preserves user data by default.
- Deletes local user data only when the user chooses that option.

User data locations considered for deletion:

```text
%APPDATA%\com.cliply.app
%LOCALAPPDATA%\com.cliply.app
```

## Manual Test Matrix

- [ ] Fresh default install.
- [ ] Fresh custom path install.
- [ ] Update over existing install.
- [ ] Update while Cliply is running.
- [ ] Desktop shortcut checked.
- [ ] Desktop shortcut unchecked.
- [ ] Startup checked.
- [ ] Startup unchecked.
- [ ] Uninstall preserving user data.
- [ ] Uninstall deleting user data.
- [ ] Reinstall after preserved-data uninstall.
- [ ] No visible console windows during install/uninstall.

## Release Notes

- Public release binaries should be signed when possible.
- Record SHA256 checksums for uploaded artifacts.
- Do not commit installer binaries, payload archives, or target directories.
