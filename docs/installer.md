# Installer Notes

Cliply includes a modern installer app and a fallback NSIS installer.

## Outputs

Modern installer:

```text
apps\cliply-installer\src-tauri\target\release\cliply-modern-installer.exe
```

Fallback NSIS installer:

```text
src-tauri\target\release\bundle\nsis\Cliply_0.4.0-beta.1_x64-setup.exe
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
