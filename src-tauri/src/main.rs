// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
extern crate objc;

mod content_managers;
mod utils;

use content_managers::clipboard_watcher::{
    clipboard_add_entry, clipboard_clean_old_entries, clipboard_delete_all_entries,
    clipboard_delete_one_entry, clipboard_open_entry, clipboard_pause_watcher,
    clipboard_read_entries, clipboard_read_status, clipboard_resume_watcher, ClipboardWatcher,
};
use content_managers::db::{db_delete_dbfile, db_get_dbfile_path, DbConnection};
use content_managers::files_manager::{
    files_delete_one_file, files_delete_storage_path, files_get_entries, files_get_storage_path,
    FilesManager,
};
use content_managers::notes_manager::{
    clipboard_add_note, create_note, delete_all_notes, delete_note, read_notes, update_note,
    NotesManager,
};
use content_managers::settings::{settings_read, settings_update, SettingsManager};
use std::env;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{async_runtime, AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;
use utils::global_shortcut::create_global_shortcut;
use utils::monitor_utils::move_to_active_monitor;
use utils::tray_handlers::{handle_system_tray_icon_event, handle_system_tray_menu_event};
use utils::window_commands::{window_hide, window_show_qrviewer};
use utils::window_custom::WebviewWindowExt;
use utils::window_handlers::handle_window_event;

#[cfg(target_os = "macos")]
use utils::window_custom::macos::WebviewWindowExtMacos;

#[cfg(target_os = "macos")]
use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
use system_notification::WorkspaceListener;

/// Float panel window level
#[cfg(target_os = "macos")]
pub static OVERLAYED_NORMAL_LEVEL: i32 = 8;

#[cfg(target_os = "macos")]
fn apply_macos_specifics(window: &WebviewWindow) {
    use tauri::Manager;
    use tauri_nspanel::ManagerExt;

    window.remove_shadow();

    window.set_float_panel(OVERLAYED_NORMAL_LEVEL);

    let app_handle = window.app_handle();
    let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app_handle.listen_workspace(
        "NSWorkspaceDidActivateApplicationNotification",
        |app_handle| {
            let panel = app_handle.get_webview_panel("main").unwrap();
            panel.set_level(OVERLAYED_NORMAL_LEVEL);
        },
    );
}

#[tokio::main]
async fn main() {
    // share the current runtime with Tauri
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    #[cfg(target_os = "linux")]
    env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    // define the builder
    let mut builder = tauri::Builder::default();
    builder = builder
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
            clipboard_clean_old_entries,
            clipboard_read_status,
            // window related
            window_hide,
            window_show_qrviewer,
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
            // db related
            db_delete_dbfile,
            db_get_dbfile_path,
        ])
        .on_window_event(handle_window_event)
        .setup(|app| {
            // global shortcut
            create_global_shortcut(app.handle())?;
            let window = app
                .get_webview_window("main")
                .ok_or("Unable to load window")?;
            window.set_document_title("Clipper - Main");
            // reposition
            let primary_monitor = app
                .primary_monitor()
                .expect("There must be a monitor")
                .expect("There must be a monitor");
            move_to_active_monitor(
                app.app_handle(),
                &window,
                primary_monitor.position().x.into(),
                primary_monitor.position().y.into(),
                false,
            );
            // mac specific settings
            #[cfg(target_os = "macos")]
            {
                apply_macos_specifics(&window);
            }
            #[cfg(not(target_os = "macos"))]
            {
                window.set_always_on_top(true);
            }
            // create tray
            let toggle = MenuItemBuilder::with_id("toggle", "Show/Hide").build(app)?;
            let about = MenuItemBuilder::with_id("about", "About").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&toggle, &about, &quit])
                .build()?;
            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(handle_system_tray_menu_event)
                .on_tray_icon_event(handle_system_tray_icon_event)
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .build(app)?;
            // hide menu on left click
            tray.set_show_menu_on_left_click(false)?;

            async_runtime::spawn(setup(app.handle().clone()));
            Ok(())
        });

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn setup(app: AppHandle) -> Result<(), tauri::Error> {
    let db = Arc::new(DbConnection::new(app.clone()).await);
    // register notes manager
    let notes_manager = NotesManager::new(Arc::clone(&db)).await;
    app.manage(notes_manager);
    // register watcher state
    let clipboard_watcher = ClipboardWatcher::new(Arc::clone(&db), app.clone()).await;
    app.manage(clipboard_watcher);
    // register settings service
    let settings_manager = SettingsManager::new(Arc::clone(&db), app.clone()).await;
    app.manage(settings_manager);
    // register file service
    let files_manager = FilesManager::new(
        // Arc::clone(&db),
        app.clone(),
    )
    .await;
    app.manage(files_manager);
    Ok(())
}
