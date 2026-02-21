mod commands;
mod media_bridge;
mod media_sources;
mod memory;
mod osx_utils;
mod playback;
mod run;
mod session;
mod tabs_state;
mod webview_manager;
mod utils;

use serde::Serialize;
use std::sync::Mutex;
use tauri::{Emitter, LogicalSize, Manager, Runtime, Webview, Window};

use crate::tabs_state::{TabKey, TabState, TabsState};
pub use run::*;

pub const MAIN_WINDOW: &str = "MAIN_WINDOW";
pub const MAIN_WEBVIEW: &str = "MAIN_WINDOW";

pub const BACKEND_STATE_EVENT: &str = "BACKEND_STATE_EVENT";

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct BackendState {
    active_tab: Option<TabKey>,
    tabs: Vec<TabState>,
}

trait EnhancedManager<R: Runtime> {
    fn main_window(&self) -> Window<R>;
    fn main_webview(&self) -> Webview<R>;
    fn tabs_state<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&TabsState) -> V;
}

impl<M, R> EnhancedManager<R> for M
where
    M: Manager<R>,
    R: Runtime,
{
    fn main_window(&self) -> Window<R> {
        self.get_window(MAIN_WINDOW).unwrap()
    }

    fn main_webview(&self) -> Webview<R> {
        self.get_webview(MAIN_WEBVIEW).unwrap()
    }

    fn tabs_state<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&TabsState) -> V,
    {
        let mutex = self.state::<Mutex<TabsState>>();
        let mut state = mutex.lock().unwrap();
        f(&mut state)
    }
}

trait EnhancedManagerEmitter<R: Runtime> {
    fn tabs_state_mut<F, V>(&self, f: F) -> anyhow::Result<V>
    where
        F: FnOnce(&mut TabsState) -> anyhow::Result<V>;
}

impl<M, R> EnhancedManagerEmitter<R> for M
where
    M: Manager<R> + Emitter<R>,
    R: Runtime,
{
    fn tabs_state_mut<F, V>(&self, f: F) -> anyhow::Result<V>
    where
        F: FnOnce(&mut TabsState) -> anyhow::Result<V>,
    {
        let mutex = self.state::<Mutex<TabsState>>();
        let mut state = mutex.lock().unwrap();
        let result = f(&mut state)?;

        // after mutating our event emit an updated tab event
        self.emit(BACKEND_STATE_EVENT, state.state())?;

        Ok(result)
    }
}