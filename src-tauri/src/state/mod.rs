mod app_state;

use crate::tabs_state::TabsState;
use crate::{
    EnhancedManager, EnhancedManagerEmitter, BACKEND_STATE_EVENT,
};
pub use app_state::*;
use std::sync::Mutex;
use tauri::{Emitter, Manager, Runtime};

trait EnhancerAppStateManager<R> {
    fn app_state<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&AppState) -> V;
}

impl<M, R> EnhancerAppStateManager<R> for M
where
    M: Manager<R>,
    R: Runtime,
{
    fn app_state<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&AppState) -> V,
    {
        let mutex = self.state::<Mutex<AppState>>();
        let mut state = mutex.lock().unwrap();
        f(&mut state)
    }
}

trait EnhancerAppStateManagerEmitter<R> {
    fn app_state_mut<F, V>(&self, f: F) -> anyhow::Result<V>
    where
        F: FnOnce(&mut AppState) -> anyhow::Result<V>;
}

impl<M, R> EnhancerAppStateManagerEmitter<R> for M
where
    M: Manager<R> + Emitter<R>,
    R: Runtime,
{
    fn app_state_mut<F, V>(&self, f: F) -> anyhow::Result<V>
    where
        F: FnOnce(&mut AppState) -> anyhow::Result<V>,
    {
        let mutex = self.state::<Mutex<AppState>>();
        let mut state = mutex.lock().unwrap();
        let result = f(&mut state)?;

        // after mutating our event emit an updated tab event
        self.emit(BACKEND_STATE_EVENT, state.clone())?;

        Ok(result)
    }
}
