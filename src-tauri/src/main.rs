// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod content_managers;
mod error;
mod utils;

use crate::error::{emit_backend_error, AppResult};
use content_managers::bookmarks_manager::{
    bookmarks_delete_all, bookmarks_delete_one, bookmarks_read_entries, bookmarks_update_entry,
    BookmarksManager,
};
use content_managers::clipboard_watcher::{
    clipboard_add_entry, clipboard_delete_all_entries, clipboard_delete_one_entry,
    clipboard_open_entry, clipboard_pause_watcher, clipboard_read_entries, clipboard_read_status,
    clipboard_resume_watcher, ClipboardWatcher,
};
use content_managers::db::{db_delete_dbfile, db_get_dbfile_path, DbConnection};
use content_managers::files_manager::{
    files_delete_one_file, files_delete_storage_path, files_get_entries, files_get_storage_path,
    FilesManager,
};
use content_managers::filters_manager::{
    filters_create_entry, filters_delete_all, filters_delete_one, filters_read_entries,
    filters_update_entry, FiltersManager,
};
use content_managers::net_manager::{
    net_authorize_peer, net_generate_otp, net_get_status, net_list_peers, net_revoke_peer,
    net_start, net_stop, NetworkManager,
};
use content_managers::notes_manager::{
    clipboard_add_note, create_note, delete_all_notes, delete_note, read_notes, update_note,
    NotesManager,
};
use content_managers::settings::{settings_read, settings_update, SettingsManager};
use content_managers::tags_manager::{
    tags_assign_item, tags_create_entry, tags_delete_one, tags_read_entries, tags_read_item_tags,
    tags_read_items, tags_remove_item, tags_set_item_tags, tags_update_entry, TagsManager,
};
use std::env;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{async_runtime, AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;
use utils::monitor_utils::{
    default_primary_monitor, move_to_active_monitor, MAIN_WINDOW_HEIGHT, MAIN_WINDOW_WIDTH,
};
use utils::tray_handlers::{handle_system_tray_icon_event, handle_system_tray_menu_event};
use utils::window_commands::{window_hide, window_show_manager, window_show_qrviewer};
use utils::window_handlers::handle_window_event;

/// Float panel window level
#[cfg(target_os = "macos")]
const FLOATING_WINDOW_LEVEL: i64 = 10_000;

#[cfg(target_os = "macos")]
fn apply_macos_specifics(window: &WebviewWindow) {
    use objc::runtime::{Object, NO};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let Ok(raw_window) = window.ns_window() else {
            log::error!("Unable to access NSWindow handle");
            return;
        };
        let ns_window = raw_window as *mut Object;

        let _: () = msg_send![ns_window, setHasShadow: NO];
        let _: () = msg_send![ns_window, setLevel: FLOATING_WINDOW_LEVEL];

        // Preserve Tauri's defaults and add Space/fullscreen behavior needed by
        // the floating widget: CanJoinAllSpaces | Stationary | FullScreenAuxiliary.
        let existing: usize = msg_send![ns_window, collectionBehavior];
        let _: () = msg_send![ns_window, setCollectionBehavior: existing | 1 | 16 | 256];
    }
}

fn create_main_window(app: &mut tauri::App) -> tauri::Result<WebviewWindow> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    WebviewWindowBuilder::new(app, "main", WebviewUrl::App("/".into()))
        .title("Clipper")
        .decorations(false)
        .inner_size(MAIN_WINDOW_WIDTH, MAIN_WINDOW_HEIGHT)
        .always_on_top(true)
        .accept_first_mouse(true)
        .visible(false)
        .transparent(true)
        .resizable(false)
        .visible_on_all_workspaces(true)
        .build()
}

#[tokio::main]
async fn main() {
    // share the current runtime with Tauri
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    #[cfg(target_os = "linux")]
    env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    // define the builder
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_os::init());
    builder = builder
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            // clipboard related
            clipboard_pause_watcher,
            clipboard_resume_watcher,
            clipboard_add_entry,
            clipboard_read_entries,
            clipboard_delete_one_entry,
            clipboard_delete_all_entries,
            clipboard_open_entry,
            clipboard_read_status,
            // bookmarks related
            bookmarks_read_entries,
            bookmarks_delete_one,
            bookmarks_delete_all,
            bookmarks_update_entry,
            // window related
            window_hide,
            window_show_qrviewer,
            window_show_manager,
            // notes related
            create_note,
            delete_note,
            read_notes,
            update_note,
            clipboard_add_note,
            delete_all_notes,
            // settings related
            settings_read,
            settings_update,
            // files related
            files_get_entries,
            files_get_storage_path,
            files_delete_storage_path,
            files_delete_one_file,
            // filters related
            filters_create_entry,
            filters_update_entry,
            filters_delete_one,
            filters_delete_all,
            filters_read_entries,
            // tags related
            tags_create_entry,
            tags_update_entry,
            tags_delete_one,
            tags_read_entries,
            tags_set_item_tags,
            tags_assign_item,
            tags_remove_item,
            tags_read_item_tags,
            tags_read_items,
            // db related
            db_delete_dbfile,
            db_get_dbfile_path,
            // network related
            net_get_status,
            net_list_peers,
            net_generate_otp,
            net_authorize_peer,
            net_revoke_peer,
            net_start,
            net_stop,
        ])
        .on_window_event(handle_window_event)
        .setup(|app| {
            let window = create_main_window(app)?;
            if let Err(error) = window.eval("document.title = 'Clipper - Main'") {
                log::error!("Unable to set window title: {}", error);
            }
            // reposition
            let primary_monitor = default_primary_monitor(app.app_handle())?;
            if let Err(error) = move_to_active_monitor(
                app.app_handle(),
                &window,
                primary_monitor.position().x.into(),
                primary_monitor.position().y.into(),
                false,
            ) {
                log::error!("Unable to move main window during setup: {}", error);
            }
            // mac specific settings
            #[cfg(target_os = "macos")]
            {
                apply_macos_specifics(&window);
            }
            #[cfg(not(target_os = "macos"))]
            {
                window.set_always_on_top(true)?;
            }
            // create tray
            let toggle = MenuItemBuilder::with_id("toggle", "Show/Hide").build(app)?;
            let about = MenuItemBuilder::with_id("about", "About").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&toggle, &about, &quit])
                .build()?;
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(handle_system_tray_menu_event)
                .on_tray_icon_event(handle_system_tray_icon_event)
                .icon_as_template(true);
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            } else {
                log::warn!("Default window icon unavailable. Tray icon fallback will be used");
            }
            let tray = tray_builder.build(app)?;
            // hide menu on left click
            tray.set_show_menu_on_left_click(false)?;

            let setup_handle = app.handle().clone();
            async_runtime::spawn(async move {
                if let Err(error) = setup(setup_handle.clone()).await {
                    emit_backend_error(&setup_handle, &error);
                    log::error!("Application setup failed: {}", error);
                }
            });
            Ok(())
        });

    if let Err(error) = builder.run(tauri::generate_context!()) {
        log::error!("Error while running tauri application: {}", error);
    }
}

async fn setup(app: AppHandle) -> AppResult<()> {
    let bus = content_managers::message_bus::MessageBus::new(100);
    let db = Arc::new(DbConnection::new(app.clone()).await?);
    // register settings service
    let settings_manager = SettingsManager::new(Arc::clone(&db), bus.clone(), app.clone()).await?;
    app.manage(Arc::clone(&settings_manager));
    // register notes manager
    let notes_manager = NotesManager::new(Arc::clone(&db), app.clone(), bus.clone()).await;
    app.manage(notes_manager);
    // register filters manager
    let filters_manager = FiltersManager::new(Arc::clone(&db), bus.clone()).await;
    // preload initial settings/filters once and pass them into managers
    let initial_settings = settings_manager.read().await?;
    let initial_filters = match filters_manager.read().await {
        Ok(filters) => filters,
        Err(err) => {
            log::error!(
                "Unable to read initial filters for clipboard watcher: {}",
                err
            );
            Vec::new()
        }
    };
    app.manage(filters_manager);
    // register tags manager
    let tags_manager = TagsManager::new(Arc::clone(&db), app.clone()).await;
    app.manage(tags_manager);
    // register watcher state
    let clipboard_watcher = ClipboardWatcher::new(
        Arc::clone(&db),
        bus.clone(),
        app.clone(),
        initial_settings.clone(),
        FiltersManager::compile_filter_regexes(initial_filters),
    )
    .await;
    app.manage(clipboard_watcher);
    // register bookmarks manager
    let bookmarks_manager =
        BookmarksManager::new(Arc::clone(&db), bus.clone(), app.clone(), initial_settings).await;
    app.manage(bookmarks_manager);
    // register network clipboard manager
    let network_manager = NetworkManager::new(bus.clone(), app.clone()).await;
    app.manage(network_manager);
    // register file service
    let files_manager = FilesManager::new(
        // Arc::clone(&db),
        app.clone(),
    )
    .await?;
    app.manage(files_manager);
    Ok(())
}
