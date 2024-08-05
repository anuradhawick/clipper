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
use content_managers::settings::SettingsManager;
use content_managers::settings::{read_settings, update_settings};
use tauri::{async_runtime, CustomMenuItem, Manager, SystemTray, SystemTrayMenu};
use tauri_plugin_positioner::{Position, WindowExt};
use tray_handlers::{handle_system_tray_icon_event, handle_system_tray_menu_event};
use window_commands::hide_window;

#[tokio::main]
async fn main() {
    let tray_menu = SystemTrayMenu::new();
    let toggle = CustomMenuItem::new("toggle".to_string(), "Show/Hide");
    let about = CustomMenuItem::new("about".to_string(), "About");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let mut system_tray =
        SystemTray::new().with_menu(tray_menu.add_item(toggle).add_item(about).add_item(quit));
    #[cfg(target_os = "macos")]
    {
        system_tray = system_tray.with_menu_on_left_click(false);
    }

    tauri::Builder::default()
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
        .system_tray(system_tray)
        .on_system_tray_event(handle_system_tray_icon_event)
        .on_system_tray_event(handle_system_tray_menu_event)
        .setup(|app| {
            // reposition
            let window = app.get_window("main").ok_or("Unable to load window")?;
            window.move_window(Position::TopCenter)?;
            // states registration
            let app_handle = app.app_handle();
            async_runtime::spawn(async move {
                let db = Arc::new(DbConnection::new(app_handle.clone()).await);
                // register notes manager
                let notes_manager = NotesManager::new(Arc::clone(&db)).await;
                app_handle.manage(notes_manager);
                // register watcher state
                let clipboard_watcher =
                    ClipboardWatcher::new(Arc::clone(&db), app_handle.clone()).await;
                app_handle.manage(clipboard_watcher);
                // register settings service
                let settings_manager = SettingsManager::new(Arc::clone(&db)).await;
                app_handle.manage(settings_manager);
            });
            #[cfg(target_os = "macos")]
            {
                // hide app icon
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
            Ok(())
        })
        // .on_window_event(|event| {
        //     if let tauri::WindowEvent::Focused(false) = event.event() {
        //         event.window().hide().unwrap();
        //     }
        // })
        .plugin(tauri_plugin_positioner::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
