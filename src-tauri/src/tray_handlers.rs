use crate::utils::monitor_utils::move_to_active_monitor;
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon};
use tauri::{menu::MenuEvent, tray::TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition};
use tauri_plugin_opener::OpenerExt;

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
                    let primary_monitor = app
                        .primary_monitor()
                        .expect("There must be a monitor")
                        .expect("There must be a monitor");
                    move_to_active_monitor(
                        app,
                        &window,
                        primary_monitor.position().x.into(),
                        primary_monitor.position().y.into(),
                    );
                    window.show().expect("Window cannot be displayed");
                }
            }
        }
        "quit" => {
            std::process::exit(0);
        }
        "about" => {
            log::info!("Open author website");
            if let Err(e) = app
                .opener()
                .open_url("https://anuradhawick.com", None::<&str>)
            {
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
        position: PhysicalPosition { x, y },
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
            move_to_active_monitor(app, &window, x, y);
            window.show().expect("Unable to show");
            // window.set_focus().expect("Unable to focus");
            log::info!("window made visible");
        }
    }
}
