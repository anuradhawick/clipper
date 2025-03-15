use tauri::{AppHandle, LogicalPosition, Monitor, Position, WebviewWindow};

fn get_monitor_for_point(window: &WebviewWindow, x: f64, y: f64, logical: bool) -> Option<Monitor> {
    let Ok(monitors) = window.available_monitors() else {
        return None;
    };

    for monitor in monitors {
        let position = monitor.position(); // Physical position
        let size = monitor.size(); // Physical size
        let scale_factor = monitor.scale_factor(); // Get scale factor

        // Convert physical position to logical position
        let left = position.x as f64 / scale_factor;
        let top = position.y as f64 / scale_factor;
        let right = left + (size.width as f64 / scale_factor);
        let bottom = top + (size.height as f64 / scale_factor);

        // Convert input point to logical coordinates (if they are already not)
        let mut logical_x = x;
        let mut logical_y = y;

        if !logical {
            logical_x /= scale_factor;
            logical_y /= scale_factor;
        }

        // Check if the logical point is within the monitor's logical bounds
        if logical_x >= left && logical_x <= right && logical_y >= top && logical_y <= bottom {
            return Some(monitor);
        }
    }

    None
}

pub fn move_to_active_monitor(
    app: &AppHandle,
    window: &WebviewWindow,
    x: f64,
    y: f64,
    logical: bool,
) {
    if let Some(monitor) = get_monitor_for_point(window, x, y, logical) {
        let window_width = app.config().app.windows[0].width;
        let screen_width = monitor.size().width as f64 / monitor.scale_factor();
        let new_x = monitor.position().x as f64 / monitor.scale_factor() + screen_width / 2.0
            - window_width / 2.0;
        let new_y = monitor.position().y as f64 / monitor.scale_factor();

        window
            .set_position(Position::Logical(LogicalPosition { x: new_x, y: new_y }))
            .expect("Unable to set position");
    }
}
