# Clipper Backend Architecture

This document describes the Rust/Tauri backend in `src-tauri/src`, the main
backend components, and the events they emit. Keep it current when backend
commands, managers, app events, internal bus messages, or schema ownership
change.

## Runtime Shape

Clipper is an Angular frontend hosted by a Tauri 2 backend. Frontend services
call Rust functions through Tauri commands registered in
`src-tauri/src/main.rs`. The backend persists structured data in a SQLite
database at `$HOME/clipper.db`, stores dropped files under `$HOME/clipper/`,
and uses app-wide Tauri events to keep all open windows in sync.

Startup flow:

1. `main.rs` configures Tauri plugins for OS integration, global shortcuts,
   opening URLs/files, logging, autostart, and the system tray.
2. `setup()` creates a `MessageBus`, opens the SQLite connection, runs SQLx
   migrations, constructs all managers, and registers them with `app.manage`.
3. The main widget window is positioned on the active monitor, made floating or
   always-on-top depending on platform, and wired to drag/drop and tray events.
4. Background tasks start for clipboard polling and internal bus subscribers.

## Backend Components

| Component | File | Responsibility |
| --- | --- | --- |
| App bootstrap | `src-tauri/src/main.rs` | Registers commands, Tauri plugins, tray handlers, window handlers, app-managed state, and platform-specific window behavior. |
| Error layer | `src-tauri/src/error.rs` | Defines `AppError`, serializes command failures, and emits `backend_error` from `with_error_event` wrappers. |
| Database connection | `src-tauri/src/content_managers/db.rs` | Opens `$HOME/clipper.db`, creates it if missing, runs embedded migrations, and exposes DB maintenance commands. |
| Internal message bus | `src-tauri/src/content_managers/message_bus.rs` | Broadcasts backend-only `AppMessage` values between managers without round-tripping through Angular. |
| Clipboard watcher | `src-tauri/src/content_managers/clipboard_watcher.rs` | Polls the system clipboard, saves text/image history, applies regex filters, writes selected history items back to the clipboard, and trims history. |
| Bookmarks manager | `src-tauri/src/content_managers/bookmarks_manager.rs` | Subscribes to copied text, extracts URLs, fetches title/description/image metadata, persists bookmarks, and trims bookmark history. |
| Notes manager | `src-tauri/src/content_managers/notes_manager.rs` | Creates, reads, updates, deletes, and copies saved text notes. |
| Settings manager | `src-tauri/src/content_managers/settings.rs` | Stores theme/history/autostart/shortcut settings, updates OS autostart and global shortcut state, and broadcasts settings changes. |
| Filters manager | `src-tauri/src/content_managers/filters_manager.rs` | Persists regex clipboard filters and broadcasts compiled filter updates to the clipboard watcher. |
| Tags manager | `src-tauri/src/content_managers/tags_manager.rs` | Manages tag definitions and tag assignments for clipboard, bookmark, and note items. |
| Files manager | `src-tauri/src/content_managers/files_manager.rs` | Copies dropped files/folders into `$HOME/clipper/`, lists managed files, and deletes one or all stored files. |
| Global shortcut | `src-tauri/src/content_managers/global_shortcut.rs` | Registers the configured shortcut and toggles the widget window near the active monitor or mouse position. |
| Window commands | `src-tauri/src/utils/window_commands.rs` | Hides the widget and creates or focuses manager and QR viewer windows. |
| Window handlers | `src-tauri/src/utils/window_handlers.rs` | Bridges native drag/drop lifecycle events to Angular and forwards dropped paths to `FilesManager`. |
| Tray handlers | `src-tauri/src/utils/tray_handlers.rs` | Handles tray toggle, about, quit, and left-click widget visibility behavior. |
| Monitor utilities | `src-tauri/src/utils/monitor_utils.rs` | Finds the monitor for a point and repositions the widget window. |

## Persistence Ownership

SQLite migrations live in `src-tauri/migrations/` and are embedded by SQLx.
The current tables are:

| Table | Owner | Notes |
| --- | --- | --- |
| `clipboard` | `ClipboardWatcher` | Text and PNG-encoded image clipboard history. |
| `bookmarks` | `BookmarksManager` | URL metadata discovered from copied text. |
| `notes` | `NotesManager` | User-authored text notes. |
| `filters` | `FiltersManager` | Regexes used to ignore matching clipboard text. |
| `tags` | `TagsManager` | Tag labels and color/kind metadata. |
| `tag_items` | `TagsManager` plus cleanup in item owners | Many-to-many tag assignments for `clipboard`, `bookmark`, and `note` items. |
| `settings` | `SettingsManager` | Singleton row for UI preferences, history limits, and global shortcut. |

When deleting clipboard, bookmark, or note records, the owning manager also
cleans matching `tag_items` rows so the tag assignment table does not keep
orphaned item references.

## Frontend Event Contract

These events are emitted through `AppHandle::emit` and consumed by Angular
services through `@tauri-apps/api/event`.

| Event | Payload | Emitted by | Consumed by | Why it is emitted |
| --- | --- | --- | --- | --- |
| `backend_error` | `{ code, message }` | `error.rs` and explicit error paths in tray/shortcut handlers | `BackendErrorService` | Shows backend failures in the UI instead of leaving command failures or background errors silent. |
| `clipboard_entry_added` | `ClipboardEvent` | `ClipboardWatcher` when polling detects new text/image | `ClipboardHistoryService` | Pushes new clipboard entries immediately to open windows without waiting for a refetch. |
| `clipboard_updated` | `()` | `ClipboardWatcher::notify_clipboard_updated` after delete one/all | `ClipboardHistoryService` | Invalidates clipboard history after destructive changes so every window refetches. |
| `clipboard_status_changed` | `bool` | `clipboard_pause_watcher`, `clipboard_resume_watcher` | `ClipboardHistoryService` | Keeps pause/resume controls synchronized with watcher state across windows. |
| `bookmark_entry_added` | `BookmarkItem` | `BookmarksManager` after URL metadata is created or refreshed | `BookmarksService` | Pushes newly discovered or refreshed bookmark metadata to open windows. |
| `bookmarks_updated` | `()` | `BookmarksManager::notify_bookmarks_updated` after update/delete/delete-all | `BookmarksService` | Invalidates bookmark lists after mutations that are easier to refetch than patch locally. |
| `notes_updated` | `()` | `NotesManager::notify_notes_updated` after update/delete/delete-all | `NotesService` | Keeps note lists consistent across widget and manager windows. |
| `settings_changed` | `SettingsEntry` | `settings_update` after settings are saved | `SettingsService` | Broadcasts preferences so all windows apply theme, history limits, autostart, and shortcut changes consistently. |
| `tags_updated` | `()` | `TagsManager::notify_tags_updated` after tag create/update/delete | `TagsService` | Invalidates tag metadata when tag labels or colors change. |
| `tag_items_updated` | `()` | `TagsManager::notify_tag_items_updated` after assignments change or a tag is deleted | `TagsService` | Invalidates per-item tag queries and the tagged-items page after assignment changes. |
| `window_dragdrop` | `{ eventType, paths? }` | `handle_window_event` on native drag enter/drop/leave | `DropperService` | Mirrors native drag state so Angular can show or clear drop overlays. |
| `files_added_paths` | `FileEntry[]` | `FilesManager::handle_drop` after copied dropped paths | `DropperService` | Sends the frontend the managed storage entries created from a drop. |

Most "updated" events intentionally carry no payload. They are invalidation
signals: the frontend service owns its local signal state and refetches through
the corresponding command. Events with payloads are used when the backend has a
single concrete item ready and pushing it avoids a full read.

## Internal Backend Bus

`MessageBus` is a Tokio broadcast channel for backend-only coordination. It is
created once in `setup()` and passed to managers that need it.

| Message | Payload | Sent by | Received by | Why it exists |
| --- | --- | --- | --- | --- |
| `AddedToClipboard` | copied text | `ClipboardWatcher` after saving new text clipboard entry | `BookmarksManager` | Lets bookmark extraction run from copied text without Angular involvement. |
| `SetClipboardText` | text | `NotesManager` when copying a note | `ClipboardWatcher` | Writes note text into the system clipboard while updating `last_text`, preventing the watcher from re-adding the same text as a new history item. |
| `FiltersUpdated` | compiled regex list | `FiltersManager` after filter create/update/delete/delete-all | `ClipboardWatcher` | Refreshes active clipboard filters without restarting the watcher. |
| `SettingsUpdated` | clipboard and bookmark history limits | `SettingsManager` after settings save | `ClipboardWatcher`, `BookmarksManager` | Applies history size changes to long-lived managers. |

Use the bus for backend-to-backend state propagation. Use Tauri events for
backend-to-frontend synchronization.

## Command Surface

Commands are registered in `main.rs` with `tauri::generate_handler!`. Keep this
list and frontend `invoke(...)` calls in sync when adding or renaming commands.

- Clipboard: `clipboard_pause_watcher`, `clipboard_resume_watcher`,
  `clipboard_add_entry`, `clipboard_read_entries`,
  `clipboard_delete_one_entry`, `clipboard_delete_all_entries`,
  `clipboard_open_entry`, `clipboard_read_status`
- Bookmarks: `bookmarks_read_entries`, `bookmarks_delete_one`,
  `bookmarks_delete_all`, `bookmarks_update_entry`
- Notes: `create_note`, `delete_note`, `read_notes`, `update_note`,
  `clipboard_add_note`, `delete_all_notes`
- Settings and filters: `settings_read`, `settings_update`,
  `filters_create_entry`, `filters_update_entry`, `filters_delete_one`,
  `filters_delete_all`, `filters_read_entries`
- Tags: `tags_create_entry`, `tags_update_entry`, `tags_delete_one`,
  `tags_read_entries`, `tags_set_item_tags`, `tags_assign_item`,
  `tags_remove_item`, `tags_read_item_tags`, `tags_read_items`
- Files and DB: `files_get_entries`, `files_get_storage_path`,
  `files_delete_storage_path`, `files_delete_one_file`, `db_delete_dbfile`,
  `db_get_dbfile_path`
- Windows: `window_hide`, `window_show_qrviewer`, `window_show_manager`

## Backend Change Checklist

When changing backend behavior:

- Wrap Tauri command bodies with `with_error_event` or
  `with_error_event_sync` so failures emit `backend_error`.
- Add new commands to `generate_handler!` and to the matching Angular service.
- Prefer Tauri events for UI synchronization and `MessageBus` messages for
  manager-to-manager synchronization.
- Document any new, renamed, or removed event in the event tables above.
- Use a new SQLx migration for schema changes; do not edit shared migrations.
- Clean dependent `tag_items` rows when deleting records that can be tagged.
- Run at least `cargo fmt --manifest-path src-tauri/Cargo.toml` and
  `cargo check --manifest-path src-tauri/Cargo.toml` after Rust changes when
  practical.
