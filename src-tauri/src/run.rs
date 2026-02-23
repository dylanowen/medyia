use crate::tabs_state::TabsState;
use crate::{
    EnhancedManager, MAIN_WEBVIEW, MAIN_WINDOW, commands, media_bridge, memory, playback, session,
    webview_manager,
};
use log::error;
use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::{MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::{
    Builder, LogicalPosition, Manager, WebviewBuilder, WebviewUrl, WindowBuilder, WindowEvent,
};
use tokio::time::sleep;
use crate::state::AppState;
use crate::utils::EnhancedWindow;

const CLOSE_TAB_KEY: &str = "CLOSE_TAB";
const TOGGLE_DEVTOOLS_KEY: &str = "TOGGLE_DEVTOOLS";

#[cfg_attr(mobile, mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .manage(AppState::new())
        .manage(TabsState::new())
        .invoke_handler(tauri::generate_handler![
            commands::create_tab,
            commands::switch_tab,
            commands::close_tab,
            commands::get_sources,
            commands::get_backend_state,
        ])
        .setup(|app| {
            let file_menu = SubmenuBuilder::new(app, "File").item(&MenuItem::with_id(
                app,
                CLOSE_TAB_KEY,
                "Close Tab",
                true,
                Some("cmd+w"),
            )?);
            #[cfg(debug_assertions)]
            let file_menu = file_menu.separator().item(&MenuItem::with_id(
                app,
                TOGGLE_DEVTOOLS_KEY,
                "Open DevTools",
                true,
                Some("cmd+option+i"),
            )?);
            let file_menu = file_menu.separator().close_window().build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;
            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&edit_menu)
                .build()?;
            app.set_menu(menu)?;

            // Create window programmatically (no config-created WebviewWindow)
            let window = WindowBuilder::new(app, MAIN_WINDOW)
                .title("Medyia")
                .inner_size(1400.0, 1000.0)
                .min_inner_size(800.0, 600.0)
                .build()?;

            let _main_view = window.add_child(
                WebviewBuilder::new(MAIN_WEBVIEW, WebviewUrl::App("index.html".into()))
                    .auto_resize(),
                LogicalPosition::new(0., 0.),
                app.main_window().size()?,
            )?;

            let handle = app.handle();
            playback::setup_playback_listener(&handle);
            media_bridge::setup_media_keys(&handle);
            memory::start_memory_monitor(handle.clone());
            session::restore_session(&handle);
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            CLOSE_TAB_KEY => {
                let state_mutex = app.state::<Mutex<TabsState>>();
                let active = {
                    let state = state_mutex.lock().unwrap();
                    state.active_tab_key.clone()
                };
                if let Some(label) = active {
                    let _ = webview_manager::close_tab(app, &label);
                }
            }
            TOGGLE_DEVTOOLS_KEY => {
                let main_webview = app.main_webview();
                if main_webview.is_devtools_open() {
                    main_webview.close_devtools();
                } else {
                    main_webview.open_devtools();
                }

                // TODO this isn't perfect
                tauri::async_runtime::spawn({
                    let app = app.clone();
                    async move {
                        sleep(Duration::from_millis(100)).await;
                        if let Err(e) = webview_manager::relayout(&app) {
                            error!("{e:?}")
                        }
                    }
                });
            }
            _ => (),
        })
        .on_window_event(|window, event| match event {
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Err(e) = webview_manager::relayout(window.app_handle()) {
                    error!("{e:?}")
                }
            }
            WindowEvent::CloseRequested { .. } => {
                session::save_session(window.app_handle());
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
