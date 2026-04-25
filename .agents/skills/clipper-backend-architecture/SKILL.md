---
name: clipper-backend-architecture
description: Use when modifying or reviewing Clipper's Tauri/Rust backend, backend emitted events, internal MessageBus messages, SQLite migrations, content managers, or Angular service contracts. Ensures ARCHITECTURE.md and this project skill stay current as backend behavior changes.
---

# Clipper Backend Architecture

Use this skill for backend work in this repository, especially changes under
`src-tauri/src`, `src-tauri/migrations`, or Angular services that call
`invoke(...)` or `listen(...)`.

## Required maintenance

- Read `ARCHITECTURE.md` before changing backend architecture, commands,
  emitted events, `MessageBus` messages, database ownership, or manager setup.
- Update `ARCHITECTURE.md` in the same change whenever those contracts change.
- Update this `SKILL.md` only when the agent workflow or maintenance rules need
  to change. Keep detailed architecture facts in `ARCHITECTURE.md`, not here.

## Fast orientation commands

From the repo root:

```bash
rg -n "emit\\(|listen\\(|AppMessage|MessageBus" src-tauri/src src/app
rg -n "#\\[tauri::command\\]|generate_handler|invoke\\(" src-tauri/src src/app
rg -n "CREATE TABLE|ALTER TABLE|tag_items" src-tauri/migrations src-tauri/src
```

## Local conventions

- Tauri events are the backend-to-frontend contract. Document their name,
  payload, emitter, consumer, and reason in `ARCHITECTURE.md`.
- `MessageBus` is backend-only coordination. Document new messages separately
  from Tauri events.
- Command handlers should use `with_error_event` or `with_error_event_sync` so
  failures are surfaced through `backend_error`.
- Schema changes belong in new SQLx migrations under `src-tauri/migrations/`.
  Do not edit migrations that may already have been applied.
- If a record kind can be tagged, deleting records should also clean matching
  `tag_items` rows.
- After Rust edits, run `cargo fmt --manifest-path src-tauri/Cargo.toml` and
  `cargo check --manifest-path src-tauri/Cargo.toml` when practical.
