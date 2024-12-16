// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
extern crate objc;

mod app_handle;
mod content_managers;
mod global_shortcut;
mod tray_handlers;
mod window_commands;
mod window_custom;

use content_managers::clipboard_watcher::{
    clean_old_entries, clipboard_add_entry, delete_all_clipboard_entries,
    delete_one_clipboard_entry, open_clipboard_entry, pause_clipboard_watcher,
    read_clipboard_entries, resume_clipboard_watcher, ClipboardWatcher,
};
use content_managers::db::{delete_db, get_db_path, DbConnection};
use content_managers::notes_manager::{
    clipboard_add_note, create_note, delete_note, read_notes, update_note, NotesManager,
};
use content_managers::settings::{read_settings, update_settings, SettingsManager};
use global_shortcut::create_global_shortcut;
use std::env;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{async_runtime, AppHandle, Manager};
use tauri_plugin_positioner::{Position, WindowExt};
use tray_handlers::{handle_system_tray_icon_event, handle_system_tray_menu_event};
use window_commands::hide_window;
use window_custom::WebviewWindowExt;

#[cfg(target_os = "macos")]
use window_custom::macos::WebviewWindowExtMacos;

#[cfg(target_os = "macos")]
use app_handle::AppHandleExt;

#[cfg(target_os = "macos")]
use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
use system_notification::WorkspaceListener;

/// window levels
// NOTE: league sets it's window to 1000 so we go one higher
#[cfg(target_os = "macos")]
pub static HIGHER_LEVEL_THAN_LEAGUE: i32 = 1001;
/// Float panel window level
#[cfg(target_os = "macos")]
pub static OVERLAYED_NORMAL_LEVEL: i32 = 8;

#[cfg(target_os = "macos")]
fn apply_macos_specifics(window: &WebviewWindow) {
    use tauri::Manager;
    use tauri::{AppHandle, Wry};
    use tauri_nspanel::ManagerExt;

    window.remove_shadow();

    window.set_float_panel(OVERLAYED_NORMAL_LEVEL);

    let app_handle = window.app_handle();

    app_handle.listen_workspace(
        "NSWorkspaceDidActivateApplicationNotification",
        |app_handle| {
            let bundle_id = AppHandle::<Wry>::frontmost_application_bundle_id();

            if let Some(bundle_id) = bundle_id {
                let is_league_of_legends = bundle_id == "com.riotgames.LeagueofLegends.GameClient";

                let panel = app_handle.get_webview_panel("main").unwrap();

                panel.set_level(if is_league_of_legends {
                    HIGHER_LEVEL_THAN_LEAGUE
                } else {
                    OVERLAYED_NORMAL_LEVEL
                });
            }
        },
    );
}

#[tokio::main]
async fn main() {
    #[cfg(target_os = "linux")]
    env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    // define the builder
    let mut builder = tauri::Builder::default();
    builder = builder
        .plugin(tauri_plugin_log::Builder::new().build())
        // .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            // clipboard related
            pause_clipboard_watcher,
            resume_clipboard_watcher,
            clipboard_add_entry,
            read_clipboard_entries,
            delete_one_clipboard_entry,
            delete_all_clipboard_entries,
            open_clipboard_entry,
            clean_old_entries,
            // window related
            hide_window,
            // notes related
            create_note,
            delete_note,
            read_notes,
            update_note,
            clipboard_add_note,
            // settings related
            update_settings,
            read_settings,
            // db related
            delete_db,
            get_db_path,
        ])
        .setup(|app| {
            // global shortcut
            create_global_shortcut(app.handle())?;
            // reposition
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_positioner::init())?;
            let window = app
                .get_webview_window("main")
                .ok_or("Unable to load window")?;
            window.set_document_title("Clipper - Main");
            window.move_window(Position::TopCenter).unwrap();
            // mac specific settings
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                window.set_float_panel(HIGHER_LEVEL_THAN_LEAGUE);
                // the window should always be on top
                window.set_always_on_top(true)?;
                // this helps bringing window on top
                window.set_visible_on_all_workspaces(true)?;
                // mac settings
                apply_macos_specifics(&window);
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
                .menu_on_left_click(true)
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
    let settings_manager = SettingsManager::new(Arc::clone(&db)).await;
    app.manage(settings_manager);
    Ok(())
}
