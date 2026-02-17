use tauri::{AppHandle, Manager};

use crate::media_sources::{MediaSource, MediaDefinition};
use crate::tabs_state::{TabKey, TabsState};
use crate::{webview_manager, BackendState, EnhancedManager};

#[tauri::command]
pub fn create_tab(app: AppHandle, source: MediaSource) -> tauri::Result<TabKey> {
    Ok(webview_manager::create_tab( source, None, &app)?)
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
pub fn get_backend_state(app: AppHandle) -> BackendState {
    app.tabs_state(TabsState::state)
}