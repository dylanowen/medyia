use tauri::AppHandle;

use crate::media_sources::{MediaDefinition, MediaSource};
use crate::state::{EnhancerAppStateManagerEmitter, TabKey};
use crate::webview_manager;

#[tauri::command]
pub fn create_tab(app: AppHandle, source: MediaSource) -> tauri::Result<TabKey> {
    Ok(webview_manager::create_tab(source, None, &app)?)
}

#[tauri::command]
pub fn switch_source(app: AppHandle, source: MediaSource) -> tauri::Result<()> {
    webview_manager::switch_to_source(&app, source)?;
    Ok(())
}

#[tauri::command]
pub fn switch_tab(app: AppHandle, key: TabKey) -> tauri::Result<()> {
    webview_manager::switch_to_tab(&app, &key)?;
    Ok(())
}

#[tauri::command]
pub fn close_tab(app: AppHandle, key: TabKey) -> tauri::Result<()> {
    Ok(webview_manager::close_tab(&app, &key)?)
}

// #[tauri::command]
// pub fn get_tabs(app: AppHandle) -> tauri::Result<Vec<TabState>> {
//     app.tabs_state_mut(|tab_state| tab_state.get_ordered_tabs())
// }

#[tauri::command]
pub fn get_sources() -> Vec<MediaDefinition> {
    MediaSource::ALL
        .iter()
        .map(MediaSource::definition)
        .collect()
}

// #[tauri::command]
// pub fn get_active_tab(app: AppHandle) -> Option<TabKey> {
//     let state_mutex = app.state::<Mutex<TabsState>>();
//     let state = state_mutex.lock().unwrap();
//     state.active_tab_key.clone()
// }

#[tauri::command]
pub fn emit_backend_state(app: AppHandle) -> tauri::Result<()> {
    app.emit_app_state()?;
    Ok(())
}
