mod commands;
mod media_bridge;
mod media_sources;
mod memory;
mod osx_utils;
mod playback;
mod run;
mod session;
mod state;
mod utils;
mod webview_manager;

use log::{error, warn};
use std::fmt::Debug;
use std::panic;
use tauri::{Manager, Runtime, Webview, Window};

pub use run::*;

pub const MAIN_WINDOW: &str = "MAIN_WINDOW";
pub const MAIN_WEBVIEW: &str = "MAIN_WINDOW";

pub const BACKEND_STATE_EVENT: &str = "BACKEND_STATE_EVENT";

trait EnhancedManager<R: Runtime> {
    fn main_window(&self) -> Window<R>;
    fn main_webview(&self) -> Webview<R>;
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
}

trait EnhancedResult {
    fn log_warn(&self);
    fn log_error(&self);
}

impl<T, E: Debug> EnhancedResult for Result<T, E> {
    #[track_caller]
    fn log_warn(&self) {
        if let Err(e) = self {
            warn!("{e:?}");
        }
    }

    #[track_caller]
    fn log_error(&self) {
        if let Err(e) = self {
            let loc = panic::Location::caller();
            error!("[{}:{}] {e:?}", loc.file(), loc.line());
        }
    }
}
