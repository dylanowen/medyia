use crate::media_sources::MediaSource;
use crate::osx_utils::enable_swipe_navigation;
use crate::state::TabKey;
use crate::utils::EnhancedWindow;
use crate::{EnhancedManager, EnhancedResult};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::time::{Duration, Instant};
use std::{fmt, mem};
use tauri::{LogicalPosition, LogicalSize, Manager, Runtime, Webview, WebviewBuilder, WebviewUrl};
use tokio::time::sleep;
use url::Url;

pub const TAB_BAR_HEIGHT: f64 = 56.0;
pub const MEDIA_SOURCE_BAR_WIDTH: f64 = 76.;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", bound = "")]
pub struct TabState<R: Runtime> {
    pub key: TabKey,
    pub source: MediaSource,
    pub url: Url,
    pub(super) is_active: bool,
    pub(super) is_playing: bool,
    pub display_name: String,
    #[serde(skip, default = "Instant::now")]
    pub last_interaction: Instant,
    #[serde(skip)]
    webview: Option<Webview<R>>,
}

impl<R: Runtime> TabState<R> {
    pub fn new(source: MediaSource) -> Self {
        Self::with_url(Url::parse(source.default_url()).unwrap(), source)
    }

    pub fn with_url(url: Url, source: MediaSource) -> Self {
        Self {
            key: source.next_tab_key(),
            source,
            url,
            is_active: false,
            is_playing: false,
            display_name: source.name().to_string(),
            last_interaction: Instant::now(),
            webview: None,
        }
    }

    pub fn show(&mut self, app: &impl Manager<R>) -> anyhow::Result<()> {
        debug!("Showing Tab: {}", self.key);
        self.load_tab(app)?.show()?;
        self.relayout(app)?;

        self.is_active = true;

        Ok(())
    }

    pub fn hide(&mut self) -> tauri::Result<()> {
        if let Some(webview) = &self.webview {
            debug!("Hiding Tab: {}", self.key);
            webview.hide()?;
        }

        self.is_active = false;

        Ok(())
    }

    #[deprecated]
    pub fn play(&mut self, app: &impl Manager<R>) -> anyhow::Result<()> {
        self.load_tab(app)?;
        self.maybe_eval("document.querySelector('video, audio')?.play();");
        self.is_playing = true;

        debug!("{} Tab -> Play", self.key);

        Ok(())
    }

    #[deprecated]
    pub fn pause(&mut self) {
        self.maybe_eval("document.querySelector('video, audio')?.pause();");
        self.is_playing = false;

        debug!("{} Tab -> Pause", self.key);
    }

    #[deprecated]
    pub fn next(&self) {
        if self.maybe_eval(format!(
            "document.querySelector('{}')?.click();",
            self.source.next_selector()
        )) {
            debug!("{} Tab -> Next", self.key);
        }
    }

        #[deprecated]
    pub fn previous(&self) {
        if self.maybe_eval(format!(
            "document.querySelector('{}')?.click();",
            self.source.previous_selector()
        )) {
            debug!("{} Tab -> Previous", self.key);
        }
    }

    pub fn try_unload_inactive(&mut self, max_age: Duration) -> tauri::Result<()> {
        if !self.is_active
            && !self.is_playing
            && Instant::now().duration_since(self.last_interaction) >= max_age
        {
            self.unload()?;
        }

        Ok(())
    }

    pub fn unload(&mut self) -> tauri::Result<bool> {
        let was_active = self.is_active;
        self.hide()?;
        if let Some(webview) = mem::take(&mut self.webview) {
            if let Ok(url) = webview.url() {
                self.url = url;
            }

            webview.navigate(Url::parse("about:blank").unwrap())?;
            tauri::async_runtime::spawn(async move {
                // give our webview time to navigate to about:blank
                sleep(Duration::from_millis(100)).await;
                if let Err(e) = webview.close() {
                    error!("Couldn't close webview: {e:?}");
                }
            });
        }
        self.is_active = false;
        self.is_playing = false;

        Ok(was_active)
    }

    fn load_tab(&mut self, app: &impl Manager<R>) -> tauri::Result<Webview<R>> {
        if let Some(webview) = &self.webview {
            Ok(webview.clone())
        } else {
            let init_script = self.source.init_script(&self.key);

            let window = app.main_window();
            let webview = window.add_child(
                WebviewBuilder::new(&self.key, WebviewUrl::External(self.url.clone()))
                    .initialization_script(&init_script),
                self.position(window.title_bar_height()),
                self.size(window.available_size()?),
            )?;

            enable_swipe_navigation(&webview);

            self.webview = Some(webview.clone());

            Ok(webview)
        }
    }

    fn relayout(&self, app: &impl Manager<R>) -> tauri::Result<()> {
        let window = app.main_window();
        self.relayout_advanced(window.available_size()?, window.title_bar_height())?;
        Ok(())
    }

    pub fn relayout_advanced(
        &self,
        window_size: LogicalSize<f64>,
        title_bar_height: f64,
    ) -> tauri::Result<()> {
        if let Some(webview) = &self.webview
            && self.is_active
        {
            webview.set_size(self.size(window_size))?;
            webview.set_position(self.position(title_bar_height))?;
        }
        Ok(())
    }

    fn maybe_eval(&self, js: impl Into<String>) -> bool {
        if let Some(webview) = &self.webview {
            webview.eval(js).log_error();
            true
        } else {
            false
        }
    }

    fn size(
        &self,
        LogicalSize {
            width: window_width,
            height: window_height,
        }: LogicalSize<f64>,
    ) -> LogicalSize<f64> {
        if self.source.multi_instance() {
            LogicalSize::new(
                window_width - MEDIA_SOURCE_BAR_WIDTH,
                window_height - TAB_BAR_HEIGHT,
            )
        } else {
            LogicalSize::new(window_width - MEDIA_SOURCE_BAR_WIDTH, window_height)
        }
    }

    fn position(&self, title_bar_height: f64) -> LogicalPosition<f64> {
        if self.source.multi_instance() {
            LogicalPosition::new(MEDIA_SOURCE_BAR_WIDTH, TAB_BAR_HEIGHT + title_bar_height)
        } else {
            LogicalPosition::new(MEDIA_SOURCE_BAR_WIDTH, title_bar_height)
        }
    }
}

impl<R: Runtime> Clone for TabState<R> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            source: self.source,
            url: self.url.clone(),
            is_active: self.is_active,
            is_playing: self.is_playing,
            display_name: self.display_name.clone(),
            last_interaction: self.last_interaction,
            webview: self.webview.clone(),
        }
    }
}

impl<R: Runtime> Debug for TabState<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TabState")
            .field("key", &self.key)
            .field("source", &self.source)
            .field("url", &self.url.to_string())
            .field("is_active", &self.is_active)
            .field("is_playing", &self.is_playing)
            .field("last_interaction", &self.last_interaction)
            .finish()
    }
}

impl<R: Runtime> Drop for TabState<R> {
    fn drop(&mut self) {
        self.unload().log_error();
    }
}
