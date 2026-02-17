use crate::tabs_state::TabsState;
use crate::{
    commands, media_bridge, memory, playback, session, webview_manager, window_size, MAIN_WEBVIEW,
    MAIN_WINDOW,
};
use log::error;
use std::sync::Mutex;
use tauri::menu::{MenuBuilder, MenuItem, SubmenuBuilder};
use tauri::{
    Builder, LogicalPosition, Manager, WebviewBuilder, WebviewUrl, WindowBuilder, WindowEvent,
};

#[cfg_attr(mobile, mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .manage(TabsState::new())
        .invoke_handler(tauri::generate_handler![
            commands::create_tab,
            commands::switch_tab,
            commands::close_tab,
            commands::get_sources,
            commands::get_backend_state,
        ])
        .setup(|app| {
            // Build application menu with Cmd+W → Close Tab
            let close_tab = MenuItem::with_id(app, "close_tab", "Close Tab", true, Some("cmd+w"))?;
            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&close_tab)
                .separator()
                .close_window()
                .build()?;
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

            let main_view = window.add_child(
                WebviewBuilder::new(MAIN_WEBVIEW, WebviewUrl::App("index.html".into()))
                    .auto_resize(),
                LogicalPosition::new(0., 0.),
                window_size(app)?,
            )?;

            // Open devtools for all webviews in debug builds
            #[cfg(debug_assertions)]
            main_view.open_devtools();

            // let window = WebviewWindowBuilder::new(
            //     app,
            //     MAIN_WINDOW,
            //     WebviewUrl::App("index.html".into()),
            // )
            //     .title("Medyia")
            //     .inner_size(1400.0, 1000.0)
            //     .min_inner_size(800.0, 600.0)
            //     .build()?;
            //
            // // let main_view = window.add_child(
            // //     WebviewBuilder::new(MAIN_WEBVIEW, WebviewUrl::App("index.html".into()))
            // //         .auto_resize(),
            // //     LogicalPosition::new(0.0, 0.0),
            // //     window_size(app)?,
            // // )?;
            //
            // // Open devtools for all webviews in debug builds
            // #[cfg(debug_assertions)]
            // window.open_devtools();

            let handle = app.handle().clone();
            playback::setup_playback_listener(&handle);
            media_bridge::setup_media_keys(&handle);
            memory::start_memory_monitor(handle.clone());
            session::restore_session(&handle);
            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id() == "close_tab" {
                let state_mutex = app.state::<Mutex<TabsState>>();
                let active = {
                    let state = state_mutex.lock().unwrap();
                    state.active_tab_key.clone()
                };
                if let Some(label) = active {
                    let _ = webview_manager::close_tab(app, &label);
                }
            }
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
