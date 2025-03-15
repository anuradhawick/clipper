use crate::utils::monitor_utils::move_to_active_monitor;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn create_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    let global_shortcut_keys = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyC);
    #[cfg(target_os = "macos")]
    let global_shortcut_keys = Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyC);

    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
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
                    }
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(global_shortcut_keys)?;

    Ok(())
}
