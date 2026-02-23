use crate::media_sources::MediaSource;
use crate::tabs_state::{TabKey, TabKeyRef, TabState, TabStatus};
use anyhow::anyhow;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use log::error;
use tauri::{LogicalSize, Manager, Runtime};
use crate::EnhancedManager;
use crate::osx_utils::title_bar_height;
use crate::utils::EnhancedWindow;

#[derive(Serialize, Default, Clone, Debug)]
pub struct AppState {
    pub media: HashMap<MediaSource, MediaState>,
    pub active_source: Option<MediaSource>,
    pub playing_source: Option<MediaSource>,
}

#[derive(Serialize, Clone, Debug)]
pub struct MediaState {
    source: MediaSource,
    tabs: Vec<TabState>,
    active_tab: Option<TabKey>,
    playing_tab: Option<TabKey>,
}

impl AppState {
    pub fn new() -> Mutex<Self> {
        Mutex::new(Self::default())
    }

    pub fn create_tab(&mut self, tab: TabState) -> anyhow::Result<()> {
        self.source_mut(tab.source).create_tab(tab)
    }

    pub fn show_source<M, R>(&mut self, source: MediaSource, app: &M) -> anyhow::Result<bool>     where
        M: Manager<R>,
        R: Runtime,
    {
        let source = self.source_mut(source);
        if let Some(tab) = source.active_tab.clone() {
            source.show_tab(&tab, app)?;
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    pub fn show_tab<M, R>(&mut self, key: TabKeyRef, app: &M) -> anyhow::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        for media in self.media.values_mut() {
            if media.show_tab(key, app)? {
                self.active_source = Some(media.source);
            }
        }

        Ok(())
    }

    pub fn close_tab<M, R>(&mut self, key: TabKeyRef, app: &M) -> anyhow::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        for media in self.media.values_mut() {
            match media.close_tab(key) {
                TabCloseState::Closed => {
                    break;
                }
                TabCloseState::ClosedActive(Some(next_active)) => {
                    self.show_tab(&next_active, app)?;
                    break;
                }
                TabCloseState::ClosedActive(None) => {
                    // we closed our tab but the source didn't have any other tabs so choose another one
                    self.active_source = None;
                    for source in MediaSource::ALL {
                        if self.show_source(*source, app)? {
                            // we could show this source, so we're done
                            break;
                        }
                    }
                    break;
                }
                TabCloseState::NotClosed => (),
            }
        }

        Ok(())
    }

    pub fn relayout<M, R>(&self, app: &M) -> anyhow::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        match app.main_window().available_size() {
            Ok(window_size) => {
                let title_height = title_bar_height(&app.main_window());

                if let Some(source) = self.active_source() {
                    source.relayout_advanced(window_size, title_height, app)?;
                }
            }
            Err(e) => {
                error!("Failed to get window logical size: {e:?}");
            }
        }

        Ok(())
    }

    fn can_create_tab(&self, source: MediaSource) -> bool {
        self.source(source)
            .is_none_or(|m| source.multi_instance() || m.tabs.is_empty())
    }

    fn active_source(&self) -> Option<& MediaState> {
        self.active_source.and_then(|s| self.source(s))
    }

    fn active_source_mut(&mut self) -> Option<&mut MediaState> {
        self.active_source.map(|s| self.source_mut(s))
    }

    fn source(&self, source: MediaSource) -> Option<&MediaState> {
        self.media.get(&source)
    }

    fn source_mut(&mut self, source: MediaSource) -> &mut MediaState {
        self.media
            .entry(source)
            .or_insert_with(|| MediaState::new(source))
    }
}

impl MediaState {
    pub fn new(source: MediaSource) -> Self {
        Self {
            source,
            tabs: Vec::default(),
            active_tab: None,
            playing_tab: None,
        }
    }

    pub fn create_tab(&mut self, mut tab: TabState) -> anyhow::Result<()> {
        assert_eq!(self.source, tab.source);
        if self.can_create_tab() {
            tab.status = TabStatus::Unloaded;
            self.tabs.push(tab);
            Ok(())
        } else {
            Err(anyhow!("Maximum instances reached for {:?}", tab.source))
        }
    }

    fn close_tab(
        &mut self,
        tab: TabKeyRef,
    ) -> TabCloseState
    {
        if let Some(i) = self.tabs.iter().position(|t| t.key == tab) {
            self.tabs.remove(i);

            if self.playing_tab.as_ref().is_some_and(|key| key == &tab) {
                self.playing_tab = None;
            }

            if self.active_tab.as_ref().is_some_and(|key| key == &tab) {
                let next_active_i = if i < self.tabs.len() {
                    Some(i)
                } else if i > 0 {
                    Some(i - 1)
                } else {
                    None
                };

                if let Some(next_active_i) = next_active_i {
                    let next_active = self.tabs[next_active_i].key.clone();

                    TabCloseState::ClosedActive(Some(next_active))
                } else {
                    TabCloseState::ClosedActive(None)
                }
            } else {
                TabCloseState::Closed
            }
        } else {
            TabCloseState::NotClosed
        }
    }

    fn show_tab<M, R>(&mut self, key: TabKeyRef, app: &M) -> anyhow::Result<bool>
    where
        M: Manager<R>,
        R: Runtime,
    {
        for tab in self.tabs.iter_mut() {
            if tab.status == TabStatus::Active {
                tab.status = TabStatus::Background;
            }
            tab.hide(app)?;
        }

        if let Some(tab) = self.tab_mut(key) {
            tab.show(app)?;
            self.active_tab = Some(tab.key.clone());
            Ok(true)
        }
        else {
            Ok(false)
        }
    }

    fn relayout_advanced<M, R>(
        &self,
        window_size: LogicalSize<f64>,
        title_bar_height: f64,
        app: &M,
    ) -> tauri::Result<()>
    where
        M: Manager<R>,
        R: Runtime,
    {
        if let Some(active_tab) = self.active_tab() {
            active_tab.relayout_advanced(window_size, title_bar_height, app)?;
        }

        Ok(())
    }

    fn active_tab(&self) -> Option<&TabState> {
        self.active_tab.as_ref().map(|key| self.tab(key).unwrap())
    }

    fn tab(&self, key: TabKeyRef) -> Option<&TabState> {
        self.tabs.iter().find(|t| t.key == key)
    }

    fn tab_mut(&mut self, key: TabKeyRef) -> Option<&mut TabState> {
        self.tabs.iter_mut().find(|t| t.key == key)
    }

    fn can_create_tab(&self) -> bool {
        self.source.multi_instance() || self.tabs.is_empty()
    }
}

enum TabCloseState {
    Closed,
    ClosedActive(Option<TabKey>),
    NotClosed,
}