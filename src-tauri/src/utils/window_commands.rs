use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use url::form_urlencoded;

#[tauri::command]
pub fn window_hide(window: tauri::Window) {
    window.hide().unwrap();
}

#[tauri::command]
pub fn window_show_qrviewer(app_handle: AppHandle, url: String) {
    let encoded: String = form_urlencoded::Serializer::new(String::new())
        .append_pair("url", &url)
        .finish();

    if app_handle.get_webview_window("qrviewer").is_none() {
        let _ = WebviewWindowBuilder::new(
            &app_handle,
            "qrviewer",
            WebviewUrl::App(format!("/qrviewer?{}", encoded).into()),
        )
        .title("Clipper QR Viewer")
        .inner_size(500.0, 500.0)
        .resizable(false)
        .always_on_top(true)
        .focused(true)
        .visible_on_all_workspaces(true)
        .center()
        .build();
    }
}
