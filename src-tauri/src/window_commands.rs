#[tauri::command]
pub fn hide_window(window: tauri::Window) {
    window.hide().unwrap();
}
