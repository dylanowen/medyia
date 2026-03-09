use crate::state::EnhancerAppStateManagerEmitter;
use std::time::Duration;
use tauri::AppHandle;
use tokio::time;
use crate::EnhancedResult;

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
    app.app_state_mut(|state| {
        for tab in state.tabs_mut() {
            tab.try_unload_inactive(UNLOAD_TIMEOUT)?;
        }

        Ok(())
    }).log_error();
}
