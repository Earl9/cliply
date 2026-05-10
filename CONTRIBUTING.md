# Contributing To Cliply

Thanks for your interest in Cliply. The project is currently preparing for a
beta release, so changes should stay focused on stability, tests, security,
performance, documentation, and release readiness.

## Development Setup

Install dependencies:

```powershell
npm install
```

Run the desktop app:

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

## Pull Request Guidelines

- Keep changes focused and small.
- Do not include generated build outputs.
- Do not include local runtime data, logs, databases, sync packages, secrets, or certificates.
- Add tests for backend behavior when possible.
- Update docs when behavior changes.
- For UI changes, include screenshots or a concise before/after description.
- For sync, logging, installer, or clipboard changes, describe privacy and rollback considerations.

## Commit Style

Use short imperative commit messages, for example:

```text
Fix remote sync password redaction
Update release checklist
```

## Stability Phase Rules

During `v0.4.0-beta.1` stabilization:

- Do not add new product features unless explicitly approved.
- Prefer tests and fixes over refactors.
- Keep logging free of clipboard body text, passwords, tokens, Authorization headers, and private keys.
- Treat installer, sync, paste, and local data retention changes as high risk.

## Running Release Checks

Before submitting a release-facing PR, run:

```powershell
npm run build
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run build:modern-installer
```

The installer build is slower and only required for release-facing changes or
changes affecting Tauri/Rust assets.
