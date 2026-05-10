# Sync Design

Cliply sync is local-first and provider-agnostic. The goal is to move encrypted
sync packages through user-selected storage without introducing a hosted Cliply
service.

## Goals

- Keep clipboard data local unless the user explicitly syncs.
- Encrypt sync payloads before writing them to disk or remote storage.
- Support common user-controlled storage providers.
- Preserve local data on sync failures.
- Avoid logging clipboard contents, passwords, tokens, or encrypted payloads.

## Non-Goals

- Hosted Cliply cloud service
- Account system
- Real-time collaboration
- Server-side conflict resolution

## Sync Package

Exported packages use the `.cliply-sync` extension. A package contains:

- Package metadata
- Device metadata
- Encryption metadata
- Encrypted payload

The encrypted payload can include:

- Clipboard item metadata
- Text formats
- Image metadata
- Tags
- Sync events
- Tombstones/deletion state
- Sync blob metadata

## Encryption

Sync payloads are encrypted before export. The current implementation uses:

- Argon2 for key derivation
- AES-GCM for authenticated encryption

Wrong passwords or corrupted packages must fail safely and leave local data
unchanged.

## Providers

Supported provider types:

- Disabled
- Local Folder
- WebDAV
- FTP
- FTPS

The provider abstraction supports basic file operations:

- `read`
- `write`
- `list`
- `exists`
- `delete`

Provider paths are normalized and parent traversal is rejected.

## Remote Layout

Remote providers use a scoped directory layout:

```text
CliplySync/
  manifest.json
  snapshots/
  events/
  devices/
  blobs/
```

Snapshots are encrypted `.cliply-sync` packages. Blob files are stored under
the blob namespace and should be encrypted before upload.

## Merge Behavior

Sync merge handles:

- New remote items
- Higher-revision updates
- Duplicate hashes
- Pin preservation
- Tombstones
- Local pinned item protection

Import operations should run in a transaction. If any later import step fails,
the transaction must roll back and avoid partial local history changes.

## Auto Sync

Auto sync stores scheduling state locally. A saved sync password may be used for
background sync and should be protected with platform secure storage when
available.

Auto sync should:

- Skip when disabled.
- Skip when no provider is configured.
- Fail clearly when no saved password is available.
- Record sanitized status and errors.
- Avoid blocking the UI.

## Security And Logging

Sync logs may include:

- Provider type
- Counts
- Snapshot counts
- Status
- Sanitized error categories

Sync logs must not include:

- Sync password
- Provider password
- Authorization header
- Token
- Private key
- Clipboard body text
- Encrypted payload

## Manual Validation

Before release, validate:

- Local Folder success path
- WebDAV success and wrong credential paths
- FTP success and wrong credential paths
- FTPS success and wrong credential paths
- Wrong sync password package import
- Corrupted sync package import
- Rollback after import failure
- Image metadata-only and blob modes
