# Security Policy

## Supported Versions

Cliply is currently in `v0.4.0-beta.1` stabilization. Until a stable release is
published, security fixes are made on the main development branch.

| Version | Supported |
| --- | --- |
| 0.4.x beta | Yes |
| Earlier versions | No |

## Reporting A Vulnerability

Please do not publish security issues publicly before maintainers have had time
to investigate.

When reporting a vulnerability, include:

- A short description of the issue
- Affected version or commit
- Steps to reproduce
- Impact and affected data
- Whether clipboard contents, passwords, tokens, private keys, or sync data may leak

If the repository has GitHub private vulnerability reporting enabled, use that.
Otherwise, contact the maintainers privately through the project owner's
preferred channel.

## Security-Sensitive Areas

- Clipboard capture and paste behavior
- Local SQLite storage
- Image blob storage
- Sync package encryption/decryption
- Remote provider authentication
- Saved sync password storage
- Logging and diagnostics
- Installer update/uninstall behavior

## Handling Secrets In Issues

Do not paste real secrets into issues, discussions, pull requests, screenshots,
or logs. Redact:

- Passwords
- API tokens
- Authorization headers
- Private keys
- WebDAV/FTP/FTPS server addresses if private
- Clipboard contents that include personal or confidential information

## Build Provenance

Release binaries should be built from tagged commits. Public release binaries
should be signed when possible.
