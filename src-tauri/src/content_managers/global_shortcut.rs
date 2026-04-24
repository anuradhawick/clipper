use crate::utils::monitor_utils::move_to_active_monitor;
use crate::{
    error::{emit_backend_error, AppError, AppResult},
    utils::monitor_utils::default_primary_monitor,
};
use mouse_position::mouse_position::Mouse;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn register_global_shortcut(app: &AppHandle, global_shortcut_keys: Shortcut) -> AppResult<()> {
    // check if the global shortcut is already registered, avoid duplicate registration - which fails understandably
    if app.global_shortcut().is_registered(global_shortcut_keys) {
        return Ok(());
    }
    // we unregister all shortcuts first to avoid conflicts
    app.global_shortcut().unregister_all()?;
    // register the global shortcut
    app.global_shortcut()
        .on_shortcut(global_shortcut_keys, move |app, shortcut, event| {
            if shortcut == &global_shortcut_keys {
                match event.state() {
                    ShortcutState::Pressed => {
                        // ignore
                    }
                    ShortcutState::Released => {
                        if let Some(window) = app.get_webview_window("main") {
                            let is_visible = match window.is_visible() {
                                Ok(value) => value,
                                Err(error) => {
                                    let app_error = AppError::runtime(format!(
                                        "Window visibility check failed: {}",
                                        error
                                    ));
                                    emit_backend_error(app, &app_error);
                                    log::error!("{}", app_error);
                                    return;
                                }
                            };

                            if is_visible {
                                if let Err(error) = window.hide() {
                                    let app_error = AppError::runtime(format!(
                                        "Window cannot be hidden: {error}"
                                    ));
                                    emit_backend_error(app, &app_error);
                                    log::error!("{}", app_error);
                                }
                            } else {
                                let position = Mouse::get_mouse_position();
                                match position {
                                    Mouse::Position { x, y } => {
                                        if let Err(error) = move_to_active_monitor(
                                            app,
                                            &window,
                                            x.into(),
                                            y.into(),
                                            true,
                                        ) {
                                            emit_backend_error(app, &error);
                                            log::error!("{}", error);
                                        }
                                    }
                                    Mouse::Error => {
                                        log::error!(
                                        "Error getting mouse position. Moving to primary monitor"
                                    );
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
                                    let app_error = AppError::runtime(format!(
                                        "Window cannot be displayed: {error}"
                                    ));
                                    emit_backend_error(app, &app_error);
                                    log::error!("{}", app_error);
                                }
                                if let Err(error) = window.set_focus() {
                                    let app_error = AppError::runtime(format!(
                                        "Window cannot be focused: {error}"
                                    ));
                                    emit_backend_error(app, &app_error);
                                    log::error!("{}", app_error);
                                }
                            }
                        }
                    }
                }
            }
        })?;

    Ok(())
}

pub fn unregister_global_shortcut(app: &AppHandle) -> AppResult<()> {
    app.global_shortcut().unregister_all()?;

    Ok(())
}
