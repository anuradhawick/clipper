use crate::{
    error::{emit_backend_error, AppError},
    utils::monitor_utils::{default_primary_monitor, move_to_active_monitor},
};
use mouse_position::mouse_position::Mouse;
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon};
use tauri::{menu::MenuEvent, tray::TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition};
use tauri_plugin_opener::OpenerExt;

pub fn handle_system_tray_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "toggle" => {
            if let Some(window) = app.get_webview_window("main") {
                let is_visible = match window.is_visible() {
                    Ok(value) => value,
                    Err(error) => {
                        let app_error =
                            AppError::runtime(format!("Window visibility check failed: {error}"));
                        emit_backend_error(app, &app_error);
                        log::error!("{}", app_error);
                        return;
                    }
                };

                if is_visible {
                    if let Err(error) = window.hide() {
                        let app_error =
                            AppError::runtime(format!("Window cannot be hidden: {error}"));
                        emit_backend_error(app, &app_error);
                        log::error!("{}", app_error);
                    }
                } else {
                    let position = Mouse::get_mouse_position();
                    match position {
                        Mouse::Position { x, y } => {
                            if let Err(error) =
                                move_to_active_monitor(app, &window, x.into(), y.into(), true)
                            {
                                emit_backend_error(app, &error);
                                log::error!("{}", error);
                            }
                        }
                        Mouse::Error => {
                            log::error!("Error getting mouse position. Moving to primary monitor");
                            let primary_monitor = match default_primary_monitor(app) {
                                Ok(monitor) => monitor,
                                Err(error) => {
                                    emit_backend_error(app, &error);
                                    log::error!("{}", error);
                                    return;
                                }
                            };
                            if let Err(error) = move_to_active_monitor(
                                app,
                                &window,
                                primary_monitor.position().x.into(),
                                primary_monitor.position().y.into(),
                                false,
                            ) {
                                emit_backend_error(app, &error);
                                log::error!("{}", error);
                            }
                        }
                    }
                    if let Err(error) = window.show() {
                        let app_error =
                            AppError::runtime(format!("Window cannot be displayed: {error}"));
                        emit_backend_error(app, &app_error);
                        log::error!("{}", app_error);
                    }
                    if let Err(error) = window.set_focus() {
                        let app_error =
                            AppError::runtime(format!("Window cannot be focused: {error}"));
                        emit_backend_error(app, &app_error);
                        log::error!("{}", app_error);
                    }
                }
            }
        }
        "quit" => {
            app.exit(0);
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

        let is_visible = match window.is_visible() {
            Ok(value) => value,
            Err(error) => {
                let app_error = AppError::runtime(format!("Unable to check visibility: {error}"));
                emit_backend_error(app, &app_error);
                log::error!("{}", app_error);
                return;
            }
        };

        if is_visible {
            if let Err(error) = window.hide() {
                let app_error = AppError::runtime(format!("Unable to hide window: {error}"));
                emit_backend_error(app, &app_error);
                log::error!("{}", app_error);
            }
            log::info!("window made invisible");
        } else {
            if let Err(error) = move_to_active_monitor(app, &window, x, y, false) {
                emit_backend_error(app, &error);
                log::error!("{}", error);
            }
            if let Err(error) = window.show() {
                let app_error = AppError::runtime(format!("Unable to show window: {error}"));
                emit_backend_error(app, &app_error);
                log::error!("{}", app_error);
            }
            log::info!("window made visible");
        }
    }
}
