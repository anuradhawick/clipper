// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod content_managers;
mod tray_handlers;
mod window_commands;
use std::sync::Arc;

use content_managers::clipboard_watcher::{
    clipboard_add_entry, delete_all_clipboard_entries, delete_one_clipboard_entry,
    pause_clipboard_watcher, read_clipboard_entries, resume_clipboard_watcher, ClipboardWatcher,
};
use content_managers::db::DbConnection;
use content_managers::notes_manager::{
    create_note, delete_note, read_notes, update_note, NotesManager,
};
use content_managers::settings::{read_settings, update_settings, SettingsManager};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{async_runtime, AppHandle, Manager};
use tauri_plugin_positioner::{Position, WindowExt};
use tray_handlers::{handle_system_tray_icon_event, handle_system_tray_menu_event};
use window_commands::hide_window;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // clipboard related
            pause_clipboard_watcher,
            resume_clipboard_watcher,
            clipboard_add_entry,
            read_clipboard_entries,
            delete_one_clipboard_entry,
            delete_all_clipboard_entries,
            // window related
            hide_window,
            // notes related
            create_note,
            delete_note,
            read_notes,
            update_note,
            // settings related
            update_settings,
            read_settings
        ])
        // .system_tray(system_tray)
        // .on_system_tray_event(handle_system_tray_icon_event)
        // .on_system_tray_event(handle_system_tray_menu_event)
        .setup(|app| {
            // reposition
            let window = app
                .get_webview_window("main")
                .ok_or("Unable to load window")?;
            window.move_window(Position::TopCenter)?;
            window.set_always_on_top(true)?;
            window.set_visible_on_all_workspaces(true)?;
            // create tray
            let toggle = MenuItemBuilder::with_id("toggle", "Show/Hide").build(app)?;
            let about = MenuItemBuilder::with_id("about", "About").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&toggle, &about, &quit])
                .build()?;
            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .menu_on_left_click(true)
                .on_menu_event(handle_system_tray_menu_event)
                .on_tray_icon_event(handle_system_tray_icon_event)
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .build(app)?;

            async_runtime::spawn(setup(app.handle().clone()));

            // hide menu on left click
            tray.set_show_menu_on_left_click(false)?;
            // hide app icon
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            Ok(())
        })
        .plugin(tauri_plugin_positioner::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn setup(app: AppHandle) -> Result<(), ()> {
    let db = Arc::new(DbConnection::new(app.clone()).await);
    // register notes manager
    let notes_manager = NotesManager::new(Arc::clone(&db)).await;
    app.manage(notes_manager);
    // register watcher state
    let clipboard_watcher = ClipboardWatcher::new(Arc::clone(&db), app.clone()).await;
    app.manage(clipboard_watcher);
    // register settings service
    let settings_manager = SettingsManager::new(Arc::clone(&db)).await;
    app.manage(settings_manager);
    Ok(())
}
