# Database Migrations (SQLx)

This project uses SQLx migrations for the Tauri backend database.

The migrations are embedded at build time and executed during app startup in:
- `src-tauri/src/content_managers/db.rs`

Migration files live in:
- `src-tauri/migrations/`

## Install SQLx CLI

You can install SQLx CLI with:

```bash
cargo install sqlx-cli
```

## Create a New Migration

From the repository root:

```bash
cd src-tauri
sqlx migrate add <migration_name>
```

Example:

```bash
sqlx migrate add add_notes_index
```

This creates a new SQL file in `src-tauri/migrations/` with a timestamp prefix.

## Apply Migrations Manually (Optional)

The app runs migrations automatically on startup, but you can run them manually during development:

```bash
cd src-tauri
DATABASE_URL="sqlite://$HOME/clipper.db" sqlx migrate run
```

Check migration status:

```bash
cd src-tauri
DATABASE_URL="sqlite://$HOME/clipper.db" sqlx migrate info
```

## Recommended Workflow

1. Add migration file with `sqlx migrate add ...`
2. Write SQL changes in the new migration file
3. Run the app (`pnpm tauri dev`) or run `sqlx migrate run`
4. Validate with:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

## Rules and Conventions

- Never edit old migrations that were already applied/shared.
- Always create a new migration for schema changes.
- Keep migrations deterministic and safe to run once.
- Prefer additive schema changes when possible.

