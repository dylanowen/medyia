mod app_state;
mod media_state;
mod tabs_state;

use crate::BACKEND_STATE_EVENT;
pub use app_state::*;
use std::sync::Mutex;
use tauri::{Emitter, Manager, Runtime};

pub type TabKey = String;
pub type TabKeyRef<'a> = &'a str;

pub trait EnhancerAppStateManager<R> {
    fn app_state<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&AppState<R>) -> V;
}

impl<M, R> EnhancerAppStateManager<R> for M
where
    M: Manager<R>,
    R: Runtime,
{
    fn app_state<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&AppState<R>) -> V,
    {
        let mutex = self.state::<Mutex<AppState<R>>>();
        let mut state = mutex.lock().unwrap();
        f(&mut state)
    }
}

pub trait EnhancerAppStateManagerEmitter<R> {
    fn app_state_mut<F, V>(&self, f: F) -> anyhow::Result<V>
    where
        F: FnOnce(&mut AppState<R>) -> anyhow::Result<V>;

    fn emit_app_state(&self) -> anyhow::Result<()>;
}

impl<M, R> EnhancerAppStateManagerEmitter<R> for M
where
    M: Manager<R> + Emitter<R>,
    R: Runtime,
{
    fn app_state_mut<F, V>(&self, f: F) -> anyhow::Result<V>
    where
        F: FnOnce(&mut AppState<R>) -> anyhow::Result<V>,
    {
        let result = {
            let mutex = self.state::<Mutex<AppState<R>>>();
            let mut state = mutex.lock().unwrap();
            f(&mut state)?
        };

        // after mutating our event emit an updated tab event
        self.emit_app_state()?;

        Ok(result)
    }

    fn emit_app_state(&self) -> anyhow::Result<()> {
        let json = self.app_state(AppState::state_json)?;
        self.emit_str(BACKEND_STATE_EVENT, json)?;
        Ok(())
    }
}
