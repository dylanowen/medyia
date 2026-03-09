use crate::media_sources::MediaSource;
use crate::osx_utils::title_bar_height;
use crate::state::media_state::MediaStateInternal;
use crate::state::media_state::{MediaState, TabCloseState};
use crate::state::tabs_state::TabState;
use crate::state::{TabKey, TabKeyRef};
use crate::utils::EnhancedWindow;
use crate::EnhancedManager;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Manager, Runtime};

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase", bound = "")]
pub struct AppState<R: Runtime> {
    media: HashMap<MediaSource, MediaState<R>>,
}

impl<R: Runtime> Default for AppState<R> {
    fn default() -> Self {
        Self {
            media: HashMap::default(),
        }
    }
}

impl<R: Runtime> AppState<R> {
    pub fn new() -> Mutex<Self> {
        Mutex::new(Self::default())
    }

    pub fn create_tab(
        &mut self,
        source: MediaSource,
        url_override: Option<String>,
    ) -> anyhow::Result<TabKey> {
        self.state_mut(source).create_tab(url_override)
    }

    pub fn show_source(
        &mut self,
        source: MediaSource,
        app: &impl Manager<R>,
    ) -> anyhow::Result<()> {
        for state in self.media.values_mut() {
            if state.source() != source {
                state.hide_source()?
            }
        }

        self.state_mut(source).show_source(app)
    }

    pub fn show_tab(&mut self, key: TabKeyRef, app: &impl Manager<R>) -> anyhow::Result<()> {
        for media in self.media.values_mut() {
            media.show_tab(key, app)?;
        }

        Ok(())
    }

    pub fn play_active_tab(&mut self, app: &impl Manager<R>) -> anyhow::Result<()> {
        if let Some(key) = self.active_tab_key() {
            if let Some(tab) = self.playing_tab_mut()
                && tab.key != key
            {
                tab.pause();
            }

            self.tab_mut(&key).unwrap().play(app)?;
        }

        Ok(())
    }

    pub fn pause_playing_tab(&mut self) -> bool {
        if let Some(tab) = self.playing_tab_mut() {
            tab.pause();
            true
        } else {
            false
        }
    }

    pub fn toggle_playing(&mut self, app: &impl Manager<R>) -> anyhow::Result<()> {
        if self.pause_playing_tab() {
            Ok(())
        } else {
            self.play_active_tab(app)
        }
    }

    pub fn close_active_tab(&mut self, app: &impl Manager<R>) -> anyhow::Result<()> {
        if let Some(active_tab) = self.active_tab_key() {
            self.close_tab(&active_tab, app)?;
        }

        Ok(())
    }

    pub fn close_tab(&mut self, key: TabKeyRef, app: &impl Manager<R>) -> anyhow::Result<()> {
        for media in self.media.values_mut() {
            match media.close_tab(key, app)? {
                TabCloseState::Closed | TabCloseState::ClosedActive(true) => break,
                TabCloseState::ClosedActive(false) => {
                    // we closed our tab but the source didn't have any other tabs so choose another one
                    if let Some(tab) = self.tabs_mut().next() {
                        tab.show(app)?;
                    }
                    break;
                }
                TabCloseState::NotClosed => (),
            }
        }

        Ok(())
    }

    pub fn relayout(&self, app: &impl Manager<R>) -> anyhow::Result<()> {
        match app.main_window().available_size() {
            Ok(window_size) => {
                let title_height = title_bar_height(&app.main_window());

                for source in self.media.values() {
                    source.relayout_advanced(window_size, title_height)?;
                }
            }
            Err(e) => {
                error!("Failed to get window logical size: {e:?}");
            }
        }

        Ok(())
    }

    pub fn read_session(&self) -> AppStateSession<R> {
        AppStateSession {
            tabs: self.tabs().cloned().collect(),
        }
    }

    pub fn restore_session(
        &mut self,
        AppStateSession { tabs }: AppStateSession<R>,
        app: &impl Manager<R>,
    ) -> anyhow::Result<()> {
        for tab in tabs {
            let is_active = tab.is_active.then(|| tab.key.clone());
            self.state_mut(tab.source).create_tab_advanced(tab)?;

            if let Some(key) = is_active {
                self.show_tab(&key, app)?;
            }
        }

        Ok(())
    }

    pub fn active_tab_key(&self) -> Option<TabKey> {
        self.active_tab().map(|t| t.key.clone())
    }

    fn active_tab(&self) -> Option<&TabState<R>> {
        self.tabs().find(|t| t.is_active)
    }

    //  fn active_tab_mut(&mut self) -> Option<&mut TabState<R>> {
    //     self.tabs_mut().find(|t| t.is_active)
    // }

    pub fn playing_tab(&self) -> Option<&TabState<R>> {
        self.tabs().find(|t| t.is_playing)
    }

    pub fn playing_tab_mut(&mut self) -> Option<&mut TabState<R>> {
        self.tabs_mut().find(|t| t.is_playing)
    }

    pub fn tab_mut(&mut self, key: TabKeyRef) -> Option<&mut TabState<R>> {
        self.tabs_mut().find(|t| t.key == key)
    }

    pub fn tabs(&self) -> impl Iterator<Item = &TabState<R>> {
        self.media.values().flat_map(|s| s.tabs())
    }

    pub fn tabs_mut(&mut self) -> impl Iterator<Item = &mut TabState<R>> {
        self.media.values_mut().flat_map(|s| s.tabs_mut())
    }

    pub fn state_json(&self) -> anyhow::Result<String> {
        let active_sources = self
            .media
            .values()
            .filter(|s| s.is_active())
            .collect::<Vec<_>>();
        let playing_sources = self
            .media
            .values()
            .filter(|s| s.is_playing())
            .collect::<Vec<_>>();
        assert!(
            active_sources.len() <= 1,
            "Found multiple active media: {active_sources:#?}"
        );
        assert!(
            playing_sources.len() <= 1,
            "Found multiple playing media: {playing_sources:#?}"
        );

        Ok(serde_json::to_string(&self)?)
    }

    fn maybe_state_mut(&mut self, source: MediaSource) -> Option<&mut MediaState<R>> {
        self.media.get_mut(&source)
    }

    fn state_mut(&mut self, source: MediaSource) -> &mut MediaState<R> {
        self.media
            .entry(source)
            .or_insert_with(|| MediaState::new(source))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound = "")]
pub struct AppStateSession<R: Runtime> {
    tabs: Vec<TabState<R>>,
}
