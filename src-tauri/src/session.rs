use crate::state::{
    AppState, EnhancerAppStateManager, EnhancerAppStateManagerEmitter,
};
use log::info;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "medyia-session.json";
const SESSION_KEY: &str = "session";

pub fn save_session(app: &AppHandle) -> anyhow::Result<()> {
    let session = app.app_state(AppState::read_session);

    if let Ok(store) = app.store(STORE_PATH) {
        store.set(SESSION_KEY, serde_json::to_value(&session)?);
        store.save()?;
        info!("Session saved to {STORE_PATH} @ {SESSION_KEY}");
    }

    Ok(())
}

pub fn restore_session(app: &AppHandle) -> anyhow::Result<()> {
    let store = app.store(STORE_PATH)?;

    if let Some(session) = store.get(SESSION_KEY) {
        let session = serde_json::from_value(session)?;
        app.app_state_mut(|state| state.restore_session(session, app))?
    }

    Ok(())
}
