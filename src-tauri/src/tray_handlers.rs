use tauri::{AppHandle, Manager, SystemTrayEvent};
use tauri_plugin_positioner::{Position, WindowExt};

pub fn handle_system_tray_menu_event(app: &AppHandle, event: SystemTrayEvent) {
    if let SystemTrayEvent::MenuItemClick { id, .. } = event {
        match id.as_str() {
            "toggle" => {
                if let Some(window) = app.get_window("main") {
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
                println!("Open author website");
                if let Err(e) = open::that("https://anuradhawick.com") {
                    println!("Could not open author website {:?}", e);
                }
            }
            _ => {}
        }
    }
}

pub fn handle_system_tray_icon_event(app: &AppHandle, event: SystemTrayEvent) {
    {
        if let SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } = event
        {
            println!("system tray received a left click");
            let Some(window) = app.get_window("main") else {
                eprintln!("Unable to get window");
                return;
            };

            if window.is_visible().expect("Unable to check visibility") {
                window.hide().expect("Unable to hide");
                println!("window made invisible");
            } else {
                window
                    .move_window(Position::TopCenter)
                    .expect("Unable to move window");
                window.show().expect("Unable to show");
                window.set_focus().expect("Unable to focus");
                println!("window made visible");
            }
        }
    }
}
