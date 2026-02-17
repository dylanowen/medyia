use crate::media_sources::MediaSource;
use crate::{window_size, BackendState, EnhancedManager};
use anyhow::anyhow;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{LogicalPosition, LogicalSize, Manager, Runtime, Webview, WebviewBuilder, WebviewUrl};
use url::Url;
use crate::osx_utils::{enable_swipe_navigation, title_bar_height};

pub const TAB_BAR_HEIGHT: f64 = 56.0;
pub const MEDIA_SOURCE_BAR_WIDTH: f64 = 76.;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TabStatus {
    Active,
    Background,
    Unloaded,
}

pub type TabKey = String;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    pub key: TabKey,
    pub source: MediaSource,
    pub url: Url,
    pub status: TabStatus,
    pub is_playing: bool,
    pub display_name: String,
    #[serde(skip)]
    pub last_interaction: Instant,
}

pub struct TabsState {
    pub tabs: HashMap<TabKey, TabState>,
    pub active_tab_key: Option<TabKey>,
    pub playing_tab_key: Option<TabKey>,
    pub tab_order: Vec<TabKey>,
}

impl TabsState {
    pub fn new() -> Mutex<Self> {
        Mutex::new(Self {
            tabs: HashMap::new(),
            active_tab_key: None,
            playing_tab_key: None,
            tab_order: Vec::new(),
        })
    }

    pub fn show_tab<M, R>(&mut self, key: &str, app: &M) -> anyhow::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        for tab in self.tabs.values_mut() {
            if tab.status == TabStatus::Active {
                tab.status = TabStatus::Background;
            }
            tab.hide(app)?;
        }

        let tab = self
            .tabs
            .get_mut(key)
            .ok_or_else(|| anyhow!("Tab not found: {key:?}"))?;

        tab.show(app)?;

        self.active_tab_key = Some(key.to_string());

        Ok(())
    }

    pub fn create_tab<M, R>(&mut self, tab: TabState, app: &M) -> anyhow::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        if self.can_create_tab(tab.source) {
            Ok(())
        } else {
            Err(anyhow!("Maximum instances reached for {:?}", tab.source))
        }?;

        let webview = tab.create_webview(app)?;
        webview.hide()?;

        self.tab_order.push(tab.key.clone());
        self.tabs.insert(tab.key.clone(), tab);

        Ok(())
    }

    pub fn close_tab<M, R>(&mut self, key: &str, app: &M) -> anyhow::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        if let Some(mut tab) = self.tabs.remove(key) {
            tab.unload_tab(app)?;

            let mut next_i = self
                .tab_order
                .iter()
                .position(|key| key == &tab.key)
                .map(|i| {
                    self.tab_order.remove(i);
                    i + 1
                })
                .unwrap_or(usize::MAX);

            // we were the active tab so focus on something else if there is anything else
            if Some(&tab.key) == self.active_tab_key.as_ref() && !self.tab_order.is_empty() {
                // try to find a previous matching source tab
                for (i, other_tab) in self
                    .tab_order
                    .iter()
                    .enumerate()
                    .take(next_i)
                    .rev()
                    .map(|(i, tab_key)| (i, self.tabs.get(tab_key).unwrap()))
                {
                    if tab.source == other_tab.source {
                        next_i = i;
                        break;
                    }
                }

                // try to find a next matching source tab
                for (i, other_tab) in self
                    .tab_order
                    .iter()
                    .enumerate()
                    .skip(next_i)
                    .map(|(i, tab_key)| (i, self.tabs.get(tab_key).unwrap()))
                {
                    if tab.source == other_tab.source {
                        next_i = i;
                        break;
                    }
                }

                // if we didn't find anything, and we're past the end, just choose the last
                if next_i >= self.tab_order.len() {
                    next_i = self.tab_order.len() - 1;
                }

                let new_active_tab_key = &self.tab_order[next_i].clone();
                self.show_tab(new_active_tab_key, app)?;
            }

            if Some(&tab.key) == self.playing_tab_key.as_ref() {
                self.playing_tab_key = None;
            }
        }

        Ok(())
    }

    pub fn relayout<M, R>(&self, app: &M) -> anyhow::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        match window_size(app) {
            Ok(window_size) => {
                let title_height = title_bar_height(&app.main_window());
                for tab in self.tabs.values() {
                    tab.relayout(window_size, title_height, app)?;
                }
            }
            Err(e) => {
                error!("Failed to get window logical size: {e:?}");
            }
        }

        Ok(())
    }

    pub fn get_ordered_tabs(&self) -> Vec<TabState> {
        self.tab_order
            .iter()
            .filter_map(|label| self.tabs.get(label))
            .cloned()
            .collect()
    }

    pub fn state(&self) -> BackendState {
        BackendState {
            active_tab: self.active_tab_key.clone(),
            tabs: self
                .tab_order
                .iter()
                .filter_map(|label| self.tabs.get(label))
                .cloned()
                .collect(),
        }
    }

    fn can_create_tab(&self, source: MediaSource) -> bool {
        source.multi_instance()
            || !self
                .tabs
                .values().any(|t| t.source == source)
    }
}

impl TabState {
    pub fn show<M, R>(&mut self, app: &M) -> anyhow::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        let webview = match self.status {
            TabStatus::Active | TabStatus::Background => self
                .webview(app)
                .ok_or_else(|| anyhow!("Couldn't find webview for tab: {}", self.key))?,
            TabStatus::Unloaded => self.create_webview(app)?,
        };

        webview.show()?;
        self.status = TabStatus::Active;

        Ok(())
    }

    pub fn hide<M, R>(&mut self, app: &M) -> tauri::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        if let Some(webview) = self.webview(app) {
            webview.hide()?;
        }

        self.status = match self.status {
            TabStatus::Active | TabStatus::Background => TabStatus::Background,
            TabStatus::Unloaded => TabStatus::Unloaded,
        };

        Ok(())
    }

    pub fn unload_tab<M, R>(&mut self, app: &M) -> tauri::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        if let Some(webview) = self.webview(app) {
            if let Ok(url) = webview.url() {
                self.url = url;
            }

            webview.close()?;
        }
        self.status = TabStatus::Unloaded;

        Ok(())
    }

    pub fn create_webview<M, R>(&self, app: &M) -> tauri::Result<Webview<R>>
    where
        M: Manager<R>,
        R: Runtime,
    {
        let init_script = self.source.init_script(&self.key);

        let window = app.main_window();
        let web_view = window.add_child(
            WebviewBuilder::new(&self.key, WebviewUrl::External(self.url.clone()))
                .initialization_script(&init_script),
            self.position(title_bar_height(&window)),
            self.size(window_size(app)?),
        )?;

        enable_swipe_navigation(&web_view);

        Ok(web_view)
    }

    fn relayout<M, R>(&self, window_size: LogicalSize<f64>, title_bar_height: f64, app: &M) -> tauri::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        if let Some(webview) = self.webview(app)
            && self.status == TabStatus::Active
        {
            webview.set_size(self.size(window_size))?;
            webview.set_position(self.position(title_bar_height))?;
        }
        Ok(())
    }

    fn webview<M, R>(&self, app: &M) -> Option<Webview<R>>
    where
        M: Manager<R>,
        R: Runtime,
    {
        app.get_webview(&self.key)
    }

    fn size(&self, LogicalSize { width: windowWidth, height: windowHeight }: LogicalSize<f64>) -> LogicalSize<f64> {
        #[cfg(debug_assertions)]
        const DEV_TOOLS_OFFSET: f64 = 400.;
        #[cfg(not(debug_assertions))]
        const DEV_TOOLS_OFFSET: f64 = 0.;

        if self.source.multi_instance() {
            LogicalSize::new(
                windowWidth - MEDIA_SOURCE_BAR_WIDTH,
                windowHeight - TAB_BAR_HEIGHT - DEV_TOOLS_OFFSET,
            )
        } else {
            LogicalSize::new(windowWidth - MEDIA_SOURCE_BAR_WIDTH, windowHeight - DEV_TOOLS_OFFSET)
        }
    }

    fn position(&self, title_bar_height: f64) -> LogicalPosition<f64> {
        if self.source.multi_instance() {
            LogicalPosition::new(MEDIA_SOURCE_BAR_WIDTH, TAB_BAR_HEIGHT + title_bar_height)
        } else {
            LogicalPosition::new(MEDIA_SOURCE_BAR_WIDTH,  title_bar_height)
        }
    }
}



