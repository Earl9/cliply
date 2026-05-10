# Privacy And Logs

This document describes privacy expectations for logs and diagnostics.

## Rule Of Thumb

Logs should help debug Cliply behavior without exposing user clipboard content
or secrets.

## Allowed In Logs

- Command names
- Item IDs
- Provider type
- Sync counts
- Snapshot counts
- Boolean state such as `enabled=true`
- Sanitized error categories
- Local operational paths when needed for diagnostics

## Not Allowed In Logs

- Clipboard body text
- Link or code body text copied by the user
- Image binary data
- Base64 image payloads
- Sync passwords
- Provider passwords
- Authorization headers
- Bearer tokens
- Access tokens
- Refresh tokens
- Private keys
- Encrypted payload fields
- Raw sync package payloads

## Redaction Markers

Log sanitization should redact messages containing markers such as:

- `password`
- `passwd`
- `authorization`
- `bearer`
- `private key`
- `private_key`
- `secret`
- `access_token`
- `refresh_token`
- `data_text`
- `preview_text`
- `normalized_text`
- `payload_json`
- `encrypted_payload`

Large secret-like tokens should be replaced with a redaction placeholder.

## Diagnostics

Diagnostics may include:

- App version
- Data directory
- Log directory
- Database path
- Database size
- History count
- Last synced time
- Sync status
- Sanitized recent error
- Sanitized last sync error

Diagnostics must not include clipboard body text, passwords, tokens,
Authorization headers, private keys, or raw payloads.

## Manual Log Scan

After release candidate manual testing, scan logs for:

```text
password
passwd
Authorization
Bearer
access_token
refresh_token
private key
secret
data_text
preview_text
normalized_text
payload_json
encrypted_payload
large base64-like tokens
```

Expected result: zero sensitive hits.
