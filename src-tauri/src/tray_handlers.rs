use tauri::tray::{MouseButton, MouseButtonState, TrayIcon};
use tauri::{menu::MenuEvent, tray::TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_positioner::{Position, WindowExt};

pub fn handle_system_tray_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "toggle" => {
            if let Some(window) = app.get_webview_window("main") {
                if window
                    .is_visible()
                    .expect("Window visibility must be available")
                {
                    window.hide().expect("Window cannot be hidden");
                } else {
                    window
                        .move_window(Position::TopCenter)
                        .expect("Unable to move window");
                    window.show().expect("Window cannot be displayed");
                }
            }
        }
        "quit" => {
            std::process::exit(0);
        }
        "about" => {
            log::info!("Open author website");
            if let Err(e) = open::that("https://anuradhawick.com") {
                log::error!("Could not open author website {:?}", e);
            }
        }
        _ => {}
    }
}

pub fn handle_system_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        log::info!("system tray received a left click");
        let app = tray.app_handle();
        let Some(window) = app.get_webview_window("main") else {
            log::error!("Unable to get window");
            return;
        };

        if window.is_visible().expect("Unable to check visibility") {
            window.hide().expect("Unable to hide");
            log::info!("window made invisible");
        } else {
            window
                .move_window(Position::TopCenter)
                .expect("Unable to move window");
            window.show().expect("Unable to show");
            // window.set_focus().expect("Unable to focus");
            log::info!("window made visible");
        }
    }
}
