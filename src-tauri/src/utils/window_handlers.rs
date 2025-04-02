use crate::content_managers::files_manager::FilesManager;
use serde::Serialize;
use tauri::{DragDropEvent, Emitter, Manager, Window, WindowEvent};

#[derive(Debug, Serialize, Clone)]
pub enum DragEventType {
    Started,
    Dropped,
    Ended,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DragEvent {
    event_type: DragEventType,
    paths: Option<Vec<String>>,
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    let app_handle = window.app_handle();

    match event {
        WindowEvent::DragDrop(DragDropEvent::Enter { paths, position: _ }) => {
            // There is an artefact in tauri that causes the paths to be empty on enter
            // especially when blobs are dragged over from other apps (images, etc)
            if paths.is_empty() {
                return;
            }
            app_handle
                .emit(
                    "dragdrop",
                    DragEvent {
                        event_type: DragEventType::Started,
                        paths: None,
                    },
                )
                .expect("Events must be emittable");
            log::info!("Hovering: started with paths: {:#?}", paths);
        }
        WindowEvent::DragDrop(DragDropEvent::Drop { paths, position: _ }) => {
            if paths.is_empty() {
                return;
            }

            tokio::task::block_in_place(|| {
                tauri::async_runtime::block_on(async {
                    let files_manager = app_handle.state::<FilesManager>();
                    if files_manager.handle_drop(paths.clone()).await.is_err() {
                        log::error!("Failed to handle drop of files: {:#?}", paths);
                    }
                })
            });

            app_handle
                .emit(
                    "dragdrop",
                    DragEvent {
                        event_type: DragEventType::Dropped,
                        paths: Some(
                            paths
                                .iter()
                                .filter_map(|p| p.clone().into_os_string().into_string().ok())
                                .collect(),
                        ),
                    },
                )
                .expect("Events must be emittable");
            log::info!("Hovering: dropped files: {:#?}", paths);
        }
        WindowEvent::DragDrop(DragDropEvent::Leave) => {
            app_handle
                .emit(
                    "dragdrop",
                    DragEvent {
                        event_type: DragEventType::Ended,
                        paths: None,
                    },
                )
                .expect("Events must be emittable");
            log::info!("Hovering: ended");
        }

        _ => {}
    }
}
