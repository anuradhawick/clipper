use crate::utils::monitor_utils::move_to_active_monitor;
use mouse_position::mouse_position::Mouse;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn register_global_shortcut(
    app: &AppHandle,
    global_shortcut_keys: Shortcut,
) -> Result<(), Box<dyn std::error::Error>> {
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
                            if window
                                .is_visible()
                                .expect("Window visibility must be available")
                            {
                                window.hide().expect("Window cannot be hidden");
                            } else {
                                let position = Mouse::get_mouse_position();
                                match position {
                                    Mouse::Position { x, y } => {
                                        move_to_active_monitor(
                                            app,
                                            &window,
                                            x.into(),
                                            y.into(),
                                            true,
                                        );
                                    }
                                    Mouse::Error => {
                                        log::error!(
                                        "Error getting mouse position. Moving to primary monitor"
                                    );
                                        let primary_monitor = app
                                            .primary_monitor()
                                            .expect("There must be a monitor")
                                            .expect("There must be a monitor");
                                        move_to_active_monitor(
                                            app,
                                            &window,
                                            primary_monitor.position().x.into(),
                                            primary_monitor.position().y.into(),
                                            false,
                                        );
                                    }
                                }
                                window.show().expect("Window cannot be displayed");
                                window.set_focus().expect("Window cannot be focused");
                            }
                        }
                    }
                }
            }
        })?;

    Ok(())
}

pub fn unregister_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().unregister_all()?;

    Ok(())
}
