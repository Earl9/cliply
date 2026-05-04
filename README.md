# Cliply

Cliply is a local-first clipboard manager desktop app built with Tauri v2, React,
TypeScript, Vite, Tailwind CSS, and Rust.

This repository is currently initialized through Phase 2 of the MVP plan:

- Phase 1: project skeleton, Tauri shell, Tailwind setup, static main window.
- Phase 2: typed mock clipboard data, search, filters, detail preview, keyboard navigation.

No cloud service, account system, system clipboard integration, or database persistence
is implemented yet.

## Run

Install dependencies:

```bash
npm install
```

Run the web UI:

```bash
npm run dev
```

Run the Tauri shell after Rust is installed:

```bash
npm run tauri dev
```

## Next Backend Steps

1. Replace mock command implementations in `src-tauri/src/services/clipboard_service.rs`
   with SQLite-backed queries.
2. Wire `src-tauri/src/db/migrations` through `rusqlite` or `sqlx`.
3. Implement Windows clipboard adapter under `src-tauri/src/platform/windows`.
4. Expose stable Tauri commands to the frontend and keep mock fallback available for tests.
