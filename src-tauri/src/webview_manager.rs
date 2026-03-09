use crate::media_sources::MediaSource;
use crate::state::{EnhancerAppStateManagerEmitter, TabKey};
use log::debug;
use tauri::AppHandle;

pub fn create_tab(
    source: MediaSource,
    url_override: Option<String>,
    app: &AppHandle,
) -> anyhow::Result<TabKey> {
    app.app_state_mut(|state| {
        let tab_key = state.create_tab(source, url_override)?;
        state.show_tab(&tab_key, app)?;

        debug!("Tab Created: {tab_key:?}");

        Ok(tab_key)
    })
}

pub fn switch_to_source(app: &AppHandle, source: MediaSource) -> anyhow::Result<()> {
    app.app_state_mut(|state| state.show_source(source, app)).map(|_| ())
}

pub fn switch_to_tab(app: &AppHandle, key: &str) -> anyhow::Result<()> {
    app.app_state_mut(|state| state.show_tab(key, app))
}

pub fn close_tab(app: &AppHandle, key: &str) -> anyhow::Result<()> {
    app.app_state_mut(|state| state.close_tab(key, app))
}

pub fn relayout(app: &AppHandle) -> anyhow::Result<()> {
    app.app_state_mut(|state| state.relayout(app))
}
