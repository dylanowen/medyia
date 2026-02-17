use crate::media_sources::MediaSource;
use crate::tabs_state::{TabKey, TabState, TabStatus};
use crate::EnhancedManagerEmitter;
use log::debug;
use std::time::Instant;
use tauri::{
    AppHandle, Url,
};

pub fn create_tab(
    source: MediaSource,
    url_override: Option<String>,
    app: &AppHandle,
) -> anyhow::Result<TabKey> {
    let tab_key = source.next_tab_key();
    let url_str = url_override.unwrap_or_else(|| source.default_url().to_string());

    let tab = TabState {
        key: tab_key.clone(),
        source,
        url: Url::parse(&url_str)?,
        status: TabStatus::Active,
        is_playing: false,
        last_interaction: Instant::now(),
        display_name: source.name().to_string(),
    };

    app.tabs_state_mut(|tabs_state| {
        tabs_state.create_tab(tab, app)?;
        tabs_state.show_tab(&tab_key, app)?;
        Ok(())
    })?;

    debug!("Tab Created: {tab_key:?}");

    Ok(tab_key)
}

pub fn switch_to_tab(app: &AppHandle, key: &str) -> anyhow::Result<()> {
    app.tabs_state_mut(|tabs_state| tabs_state.show_tab(key, app))
}

pub fn close_tab(app: &AppHandle, key: &str) -> anyhow::Result<()> {
    app.tabs_state_mut(|tabs_state| tabs_state.close_tab(key, app))
}

pub fn relayout(app: &AppHandle) -> anyhow::Result<()> {
    app.tabs_state_mut(|tabs_state| tabs_state.relayout(app))
}