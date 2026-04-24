use crate::error::{with_error_event_sync, AppError, AppResult};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use url::form_urlencoded;

#[tauri::command]
pub fn window_hide(window: tauri::Window) -> AppResult<()> {
    let app_handle = window.app_handle();
    with_error_event_sync(&app_handle, || {
        window.hide()?;
        Ok(())
    })
}

#[tauri::command]
pub fn window_show_manager(app_handle: AppHandle) -> AppResult<()> {
    with_error_event_sync(&app_handle, || {
        if app_handle.get_webview_window("manager").is_none() {
            let window = WebviewWindowBuilder::new(
                &app_handle,
                "manager",
                WebviewUrl::App("/manager".into()),
            )
            .title("Clipper Manager")
            .inner_size(1024.0, 768.0)
            .min_inner_size(1024.0, 768.0)
            .resizable(true)
            .always_on_top(false)
            .focused(true)
            .visible(true)
            .visible_on_all_workspaces(false)
            .center()
            .build()
            .map_err(AppError::from)?;

            window.show()?;
            window.set_focus()?;
        }
        Ok(())
    })
}

#[tauri::command]
pub fn window_show_qrviewer(app_handle: AppHandle, url: String) -> AppResult<()> {
    with_error_event_sync(&app_handle, || {
        let encoded: String = form_urlencoded::Serializer::new(String::new())
            .append_pair("url", &url)
            .finish();

        if app_handle.get_webview_window("qrviewer").is_none() {
            let window = WebviewWindowBuilder::new(
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
            .build()
            .map_err(AppError::from)?;

            window.on_window_event(move |event| match event {
                WindowEvent::Focused(true) => {
                    log::info!("QR Viewer window focused");
                }
                WindowEvent::CloseRequested { api: _, .. } => {
                    log::info!("QR Viewer window close requested");
                }
                _ => {}
            });
        }
        Ok(())
    })
}
