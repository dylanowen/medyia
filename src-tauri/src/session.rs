use crate::media_sources::MediaSource;
use crate::tabs_state::TabsState;
use crate::webview_manager;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;
use url::Url;

const STORE_PATH: &str = "medya-session.json";
const SESSION_KEY: &str = "session";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedTab {
    label: String,
    source_id: MediaSource,
    url: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedSession {
    tabs: Vec<SavedTab>,
    active_tab: Option<String>,
}

pub fn save_session(app: &AppHandle) {
    let state_mutex = app.state::<Mutex<TabsState>>();
    let state = state_mutex.lock().unwrap();

    let saved = SavedSession {
        tabs: state
            .tab_order
            .iter()
            .filter_map(|label| {
                state.tabs.get(label).map(|tab| SavedTab {
                    label: tab.key.clone(),
                    source_id: tab.source.clone(),
                    url: tab.url.clone(),
                })
            })
            .collect(),
        active_tab: state.active_tab_key.clone(),
    };

    if let Ok(store) = app.store(STORE_PATH) {
        store.set(
            SESSION_KEY,
            serde_json::to_value(&saved).unwrap_or_default(),
        );
        let _ = store.save();
    }
}

pub fn restore_session(app: &AppHandle) {
    let store = match app.store(STORE_PATH) {
        Ok(s) => s,
        Err(_) => return,
    };

    let session: SavedSession = match store.get(SESSION_KEY) {
        Some(val) => match serde_json::from_value(val.clone()) {
            Ok(s) => s,
            Err(_) => return,
        },
        None => return,
    };

    if session.tabs.is_empty() {
        return;
    }

    for tab in &session.tabs {
        let _ = webview_manager::create_tab(tab.source_id, Some(tab.url.to_string()), app);
    }

    // Switch to the previously active tab
    if let Some(ref active) = session.active_tab {
        let _ = webview_manager::switch_to_tab(app, active);
    }
}
