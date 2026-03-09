use crate::media_sources::MediaSource;
use crate::state::tabs_state::TabState;
use crate::state::{TabKey, TabKeyRef};
use enum_dispatch::enum_dispatch;
use log::debug;
use serde::Serialize;
use std::error::Error;
use std::time::Duration;
use tauri::{LogicalSize, Manager, Runtime};

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase", bound = "")]
#[enum_dispatch(MediaStateInternal<R>)]
pub(super) enum MediaState<R: Runtime> {
    Single(SingleMediaState<R>),
    Multi(MultiMediaState<R>),
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase", bound = "")]
pub(super) struct SingleMediaState<R: Runtime> {
    source: MediaSource,
    tab: Option<TabState<R>>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase", bound = "")]
pub(super) struct MultiMediaState<R: Runtime> {
    source: MediaSource,
    tabs: Vec<TabState<R>>,
    #[serde(skip)]
    last_active: Option<TabKey>,
}
impl<R: Runtime> MediaState<R> {
    pub(super) fn new(source: MediaSource) -> Self {
        if source.multi_instance() {
            Self::Multi(MultiMediaState::new(source))
        } else {
            Self::Single(SingleMediaState::new(source))
        }
    }
}

// impl<R: Runtime> MediaState<R> {
//     pub fn new(source: MediaSource) -> Self {
//         if source.multi_instance() {
//             Self::Multi(MultiMediaState::new(source))
//         } else {
//             Self::Single(SingleMediaState::new(source))
//         }
//     }
//
//     pub fn create_tab_advanced(&mut self, tab: TabState<R>) -> anyhow::Result<()> {
//         match self {
//             Self::Single(state) => state.set_tab(tab),
//             Self::Multi(state) => state.create_tab_advanced(tab),
//         }
//
//         Ok(())
//     }
//
//     pub(super) fn close_tab(
//         &mut self,
//         tab: TabKeyRef,
//         app: &impl Manager<R>,
//     ) -> anyhow::Result<TabCloseState> {
//         match self {
//             Self::Single(state) => state.close_tab(tab),
//             Self::Multi(state) => state.close_tab(tab, app),
//         }
//     }
//
//     pub(super) fn show_tab(
//         &mut self,
//         key: TabKeyRef,
//         app: &impl Manager<R>,
//     ) -> anyhow::Result<bool> {
//         match self {
//             Self::Single(state) => state.show_tab(key, app),
//             Self::Multi(state) => state.show_tab(key, app),
//         }
//     }
//
//     pub fn unload_inactive(&mut self, max_age: Duration) -> tauri::Result<()> {
//         match self {
//             Self::Single(state) => state.unload_inactive(key, app),
//             Self::Multi(state) => state.unload_inactive(key, app),
//         }
//     }
//
//     pub(super) fn source(&self) -> MediaSource {
//         match self {
//             MediaState::Single(state) => state.source,
//             MediaState::Multi(state) => state.source,
//         }
//     }
//
//     pub(super) fn is_active(&self) -> bool {
//         match self {
//             MediaState::Single(state) => state.is_active(),
//             MediaState::Multi(state) => state.is_active(),
//         }
//     }
//
//     pub(super) fn is_playing(&self) -> bool {
//         match self {
//             MediaState::Single(state) => state.is_playing(),
//             MediaState::Multi(state) => state.is_playing(),
//         }
//     }
//
//     // pub(super) fn play_pause_tab(
//     //     &mut self,
//     //     key: TabKeyRef,
//     //     play: bool,
//     //     app: &impl Manager<R>,
//     // ) -> anyhow::Result<bool>
//     // where
//     //     M: Manager<R>,
//     //     R: Runtime,
//     // {
//     //     self.playing_tab = None;
//     //
//     //     if let Some(tab) = self.tab_mut(key) {
//     //         let wv = tab.load_tab(app)?;
//     //
//     //         let js = if play {
//     //             "document.querySelector('video, audio')?.play();"
//     //         } else {
//     //             "document.querySelector('video, audio')?.pause();"
//     //         };
//     //
//     //         wv.eval(js).log_error();
//     //
//     //         if play {
//     //             self.playing_tab = Some(tab.key.clone());
//     //         }
//     //         Ok(true)
//     //     } else {
//     //         Ok(false)
//     //     }
//     // }
//     //
//     // pub(super) fn relayout_advanced(
//     //     &self,
//     //     window_size: LogicalSize<f64>,
//     //     title_bar_height: f64,
//     //     app: &impl Manager<R>,
//     // ) -> tauri::Result<()>
//     // {
//     //     if let Some(active_tab) = self.active_tab() {
//     //         active_tab.relayout_advanced(window_size, title_bar_height, app)?;
//     //     }
//     //
//     //     Ok(())
//     // }
// }

#[enum_dispatch]
pub trait MediaStateInternal<R: Runtime> {
    fn create_tab(&mut self, url_override: Option<String>) -> anyhow::Result<TabKey> {
        let tab = match url_override {
            Some(url) => TabState::with_url(url.parse()?, self.source()),
            None => TabState::new(self.source()),
        };
        let tab_key = tab.key.clone();

        self.create_tab_advanced(tab)?;

        Ok(tab_key)
    }

    fn create_tab_advanced(&mut self, tab: TabState<R>) -> anyhow::Result<()>;

    fn show_source(&mut self, app: &impl Manager<R>) -> anyhow::Result<()>;

    fn hide_source(&mut self) -> anyhow::Result<()> {
        for tab in self.tabs_mut() {
            tab.hide()?;
        }

        Ok(())
    }

    fn show_tab(&mut self, key: TabKeyRef, app: &impl Manager<R>) -> anyhow::Result<bool>;

    fn close_tab(&mut self, tab: TabKeyRef, app: &impl Manager<R>)
    -> anyhow::Result<TabCloseState>;

    fn unload_inactive(&mut self, max_age: Duration) -> tauri::Result<()>;

    fn relayout_advanced(
        &self,
        window_size: LogicalSize<f64>,
        title_bar_height: f64,
    ) -> tauri::Result<()> {
        for tab in self.tabs() {
            tab.relayout_advanced(window_size, title_bar_height)?;
        }

        Ok(())
    }

    fn source(&self) -> MediaSource;

    fn is_active(&self) -> bool;

    fn is_playing(&self) -> bool;

    fn tabs(&self) -> Box<dyn Iterator<Item = &TabState<R>> + '_>;

    fn tabs_mut(&mut self) -> Box<dyn Iterator<Item = &mut TabState<R>> + '_>;
}

impl<R: Runtime> SingleMediaState<R> {
    fn new(source: MediaSource) -> Self {
        Self { source, tab: None }
    }
}

impl<R: Runtime> MediaStateInternal<R> for SingleMediaState<R> {
    fn create_tab_advanced(&mut self, tab: TabState<R>) -> anyhow::Result<()> {
        assert_eq!(self.source, tab.source);

        debug!("Creating tab: {}", tab.key);

        self.tab = Some(tab);

        Ok(())
    }

    fn show_source(&mut self, app: &impl Manager<R>) -> anyhow::Result<()> {
        debug!("Showing source: {:?}", self.source);

        let tab_key = if let Some(tab) = &self.tab {
            tab.key.clone()
        } else {
            let tab = TabState::new(self.source);
            let new_key = tab.key.clone();
            self.create_tab_advanced(tab)?;

            new_key
        };

        self.show_tab(&tab_key, app)?;

        Ok(())
    }

    fn hide_source(&mut self) -> anyhow::Result<()> {
        if let Some(tab) = &mut self.tab {
            tab.hide()?;
        }

        Ok(())
    }

    fn show_tab(&mut self, key: TabKeyRef, app: &impl Manager<R>) -> anyhow::Result<bool> {
        if let Some(tab) = &mut self.tab
            && tab.key == key
        {
            tab.show(app)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn close_tab(
        &mut self,
        key: TabKeyRef,
        _app: &impl Manager<R>,
    ) -> anyhow::Result<TabCloseState> {
        if let Some(mut tab) = self.tab.take_if(|t| t.key == key) {
            debug!("Closing Tab: {}", tab.key);
            if tab.unload()? {
                Ok(TabCloseState::ClosedActive(false))
            } else {
                Ok(TabCloseState::Closed)
            }
        } else {
            Ok(TabCloseState::NotClosed)
        }
    }

    fn unload_inactive(&mut self, max_age: Duration) -> tauri::Result<()> {
        if let Some(tab) = &mut self.tab {
            tab.try_unload_inactive(max_age)?;
        }

        Ok(())
    }

    fn source(&self) -> MediaSource {
        self.source
    }

    fn is_active(&self) -> bool {
        matches!(&self.tab, Some(tab) if tab.is_active)
    }

    fn is_playing(&self) -> bool {
        matches!(&self.tab, Some(tab) if tab.is_playing)
    }

    fn tabs(&self) -> Box<dyn Iterator<Item = &TabState<R>> + '_> {
        Box::new(self.tab.iter())
    }

    fn tabs_mut(&mut self) -> Box<dyn Iterator<Item = &mut TabState<R>> + '_> {
        Box::new(self.tab.iter_mut())
    }
}

impl<R: Runtime> MultiMediaState<R> {
    fn new(source: MediaSource) -> Self {
        Self {
            source,
            tabs: Vec::default(),
            last_active: None,
        }
    }
}

impl<R: Runtime> MediaStateInternal<R> for MultiMediaState<R> {
    fn create_tab_advanced(&mut self, tab: TabState<R>) -> anyhow::Result<()> {
        assert_eq!(self.source, tab.source);

        debug!("Creating tab: {}", tab.key);

        self.tabs.push(tab);

        Ok(())
    }

    fn show_source(&mut self, app: &impl Manager<R>) -> anyhow::Result<()> {
        debug!("Showing source: {:?}", self.source);

        let tab_key = if let Some(last_active) = &self.last_active
            && self.tab(last_active).is_some()
        {
            last_active.clone()
        } else if !self.tabs.is_empty() {
            self.tabs[0].key.clone()
        } else {
            let tab = TabState::new(self.source);
            let new_key = tab.key.clone();
            self.create_tab_advanced(tab)?;

            new_key
        };

        self.show_tab(&tab_key, app)?;

        Ok(())
    }

    fn show_tab(&mut self, key: TabKeyRef, app: &impl Manager<R>) -> anyhow::Result<bool> {
        self.hide_source()?;

        if let Some(tab) = self.tab_mut(key) {
            let key = tab.key.clone();
            tab.show(app)?;
            self.last_active = Some(key);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn close_tab(
        &mut self,
        tab: TabKeyRef,
        app: &impl Manager<R>,
    ) -> anyhow::Result<TabCloseState> {
        if let Some(i) = self.tabs.iter().position(|t| t.key == tab) {
            debug!("Closing Tab: {tab}");
            let closed_tab = self.tabs.remove(i);

            if closed_tab.is_active {
                let next_active_i = if i < self.tabs.len() {
                    Some(i)
                } else if i > 0 {
                    Some(i - 1)
                } else {
                    None
                };

                if let Some(next_active_i) = next_active_i {
                    self.tabs[next_active_i].show(app)?;

                    Ok(TabCloseState::ClosedActive(true))
                } else {
                    Ok(TabCloseState::ClosedActive(false))
                }
            } else {
                Ok(TabCloseState::Closed)
            }
        } else {
            Ok(TabCloseState::NotClosed)
        }
    }

    fn unload_inactive(&mut self, max_age: Duration) -> tauri::Result<()> {
        for tab in self.tabs.iter_mut() {
            tab.try_unload_inactive(max_age)?;
        }

        Ok(())
    }

    fn source(&self) -> MediaSource {
        self.source
    }

    fn is_active(&self) -> bool {
        match self.tabs.iter().filter(|t| t.is_active).count() {
            0 => false,
            1 => true,
            multiple => panic!("{multiple} tabs are active"),
        }
    }

    fn is_playing(&self) -> bool {
        match self.tabs.iter().filter(|t| t.is_playing).count() {
            0 => false,
            1 => true,
            multiple => panic!("{multiple} tabs are playing"),
        }
    }

    fn tabs(&self) -> Box<dyn Iterator<Item = &TabState<R>> + '_> {
        Box::new(self.tabs.iter())
    }

    fn tabs_mut(&mut self) -> Box<dyn Iterator<Item = &mut TabState<R>> + '_> {
        Box::new(self.tabs.iter_mut())
    }
}

impl<R: Runtime> MultiMediaState<R> {
    fn tab(&self, key: TabKeyRef) -> Option<&TabState<R>> {
        self.tabs.iter().find(|t| t.key == key)
    }

    fn tab_mut(&mut self, key: TabKeyRef) -> Option<&mut TabState<R>> {
        self.tabs.iter_mut().find(|t| t.key == key)
    }
}

// impl MediaState {
//     pub fn new(source: MediaSource) -> Self {
//         Self {
//             source,
//             tabs: Vec::default(),
//             active_tab: None,
//             playing_tab: None,
//         }
//     }
//
//     pub fn create_tab_advanced(&mut self, mut tab: TabState) -> anyhow::Result<()> {
//         assert_eq!(self.source, tab.source);
//         if self.can_create_tab_advanced() {
//             tab.is_active = TabStatus::Unloaded;
//             self.tabs.push(tab);
//             Ok(())
//         } else {
//             Err(anyhow!("Maximum instances reached for {:?}", tab.source))
//         }
//     }
//
//     pub(super) fn close_tab(
//         &mut self,
//         tab: TabKeyRef,
//         app: &impl Manager<R>,
//     ) -> anyhow::Result<TabCloseState>
//     where
//         M: Manager<R>,
//         R: Runtime,
//     {
//         if let Some(i) = self.tabs.iter().position(|t| t.key == tab) {
//             debug!("Closing Tab: {tab}");
//             self.tabs.remove(i).unload_tab(app).log_error();
//
//             if self.playing_tab.as_ref().is_some_and(|key| key == &tab) {
//                 self.playing_tab = None;
//             }
//
//             if self.active_tab.as_ref().is_some_and(|key| key == &tab) {
//                 self.active_tab = None;
//
//                 let next_active_i = if i < self.tabs.len() {
//                     Some(i)
//                 } else if i > 0 {
//                     Some(i - 1)
//                 } else {
//                     None
//                 };
//
//                 if let Some(next_active_i) = next_active_i {
//                     let next_key = self.tabs[next_active_i].key.clone();
//                     self.show_tab(&next_key, app)?;
//
//                     Ok(TabCloseState::ClosedActive(true))
//                 } else {
//                     Ok(TabCloseState::ClosedActive(false))
//                 }
//             } else {
//                 Ok(TabCloseState::Closed)
//             }
//         } else {
//             Ok(TabCloseState::NotClosed)
//         }
//     }
//
//     pub(super) fn show_tab(&mut self, key: TabKeyRef, app: &impl Manager<R>) -> anyhow::Result<bool>
//     where
//         M: Manager<R>,
//         R: Runtime,
//     {
//         for tab in self.tabs.iter_mut() {
//             if tab.is_active == TabStatus::Active {
//                 tab.is_active = TabStatus::Background;
//             }
//             tab.hide(app)?;
//         }
//
//         if let Some(tab) = self.tab_mut(key) {
//             tab.show(app)?;
//             self.active_tab = Some(tab.key.clone());
//             Ok(true)
//         } else {
//             Ok(false)
//         }
//     }
//
//     pub(super) fn play_pause_tab(
//         &mut self,
//         key: TabKeyRef,
//         play: bool,
//         app: &impl Manager<R>,
//     ) -> anyhow::Result<bool>
//     where
//         M: Manager<R>,
//         R: Runtime,
//     {
//         self.playing_tab = None;
//
//         if let Some(tab) = self.tab_mut(key) {
//             let wv = tab.load_tab(app)?;
//
//             let js = if play {
//                 "document.querySelector('video, audio')?.play();"
//             } else {
//                 "document.querySelector('video, audio')?.pause();"
//             };
//
//             wv.eval(js).log_error();
//
//             if play {
//                 self.playing_tab = Some(tab.key.clone());
//             }
//             Ok(true)
//         } else {
//             Ok(false)
//         }
//     }
//
//     pub(super) fn relayout_advanced(
//         &self,
//         window_size: LogicalSize<f64>,
//         title_bar_height: f64,
//         app: &impl Manager<R>,
//     ) -> tauri::Result<()>
//     where
//         M: Manager<R>,
//         R: Runtime,
//     {
//         if let Some(active_tab) = self.active_tab() {
//             active_tab.relayout_advanced(window_size, title_bar_height, app)?;
//         }
//
//         Ok(())
//     }
//
//     fn active_tab(&self) -> Option<&TabState> {
//         self.active_tab.as_ref().map(|key| self.tab(key).unwrap())
//     }
//
//     pub(super) fn tab(&self, key: TabKeyRef) -> Option<&TabState> {
//         self.tabs.iter().find(|t| t.key == key)
//     }
//
//     fn tab_mut(&mut self, key: TabKeyRef) -> Option<&mut TabState> {
//         self.tabs.iter_mut().find(|t| t.key == key)
//     }
//
//     fn can_create_tab_advanced(&self) -> bool {
//         self.source.multi_instance() || self.tabs.is_empty()
//     }
// }

pub(super) enum TabCloseState {
    /// We closed our tab but it wasn't active
    Closed,
    /// We closed our tab and we report whether we could find an alternative in our media
    ClosedActive(bool),
    /// We didn't have anything to close
    NotClosed,
}
