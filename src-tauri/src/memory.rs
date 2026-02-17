use log::error;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio::time;

use crate::tabs_state::TabStatus;
use crate::EnhancedManagerEmitter;

const CHECK_INTERVAL: Duration = Duration::from_secs(30);
const UNLOAD_TIMEOUT: Duration = Duration::from_mins(15);

pub fn start_memory_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = time::interval(CHECK_INTERVAL);
        loop {
            interval.tick().await;
            check_idle_tabs(&app);
        }
    });
}

fn check_idle_tabs(app: &AppHandle) {
    let now = Instant::now();

    if let Err(e) = app.tabs_state_mut(|tabs_state| {
        for (key, tab) in tabs_state.tabs.iter_mut() {
            // Skip active and playing tabs
            if Some(key.as_str()) != tabs_state.active_tab_key.as_deref()
                && !tab.is_playing
                && tab.status != TabStatus::Unloaded
                && now.duration_since(tab.last_interaction) >= UNLOAD_TIMEOUT {
                    tab.unload_tab(app)?;
                }
        }
        Ok(())
    }) {
        error!("{:?}", e);
    }
}
