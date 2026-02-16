use log::{debug, error, info, warn};
use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Listener, Manager};

use crate::media_bridge;
use crate::tabs_state::TabsState;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackEvent {
    pub label: String,
    pub playing: bool,
    pub title: Option<String>,
    pub artist: Option<String>,
    #[allow(dead_code)]
    pub artwork_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct TitleChangedEvent {
    label: String,
    title: String,
}

pub fn setup_playback_listener(app: &AppHandle) {
    info!("Setting up playback listener");

    let app_handle = app.clone();
    app.listen("playback-state", move |event| {
        let payload = event.payload();
        debug!("Received playback-state event: {}", payload);
        match serde_json::from_str::<PlaybackEvent>(payload) {
            Ok(evt) => handle_playback_event(&app_handle, evt),
            Err(e) => error!(
                "Failed to parse playback event: {} — payload: {}",
                e, payload
            ),
        }
    });

    let app_handle = app.clone();
    app.listen("tab-title-changed", move |event| {
        let payload = event.payload();
        debug!("Received tab-title-changed event: {}", payload);
        if let Ok(evt) = serde_json::from_str::<TitleChangedEvent>(payload) {
            handle_title_changed(&app_handle, evt);
        }
    });
}

fn handle_playback_event(app: &AppHandle, event: PlaybackEvent) {
    info!(
        "Playback event: label={}, playing={}, title={:?}",
        event.label, event.playing, event.title
    );
    let state_mutex = app.state::<Mutex<TabsState>>();

    if event.playing {
        // Pause all other tabs
        let labels_to_pause: Vec<String> = {
            let state = state_mutex.lock().unwrap();
            state
                .tabs
                .keys()
                .filter(|l| **l != event.label)
                .cloned()
                .collect()
        };

        for label in &labels_to_pause {
            debug!("Pausing tab: {}", label);
            if let Some(wv) = app.get_webview(label) {
                let _ = wv.eval(
                    "document.querySelectorAll('video, audio').forEach(el => { if (!el.paused) el.pause(); });",
                );
            }
        }

        // Update state
        {
            let mut state = state_mutex.lock().unwrap();
            for (label, tab) in state.tabs.iter_mut() {
                tab.is_playing = *label == event.label;
            }
            state.playing_tab_key = Some(event.label.clone());
            debug!("State updated: playing_tab={}", event.label);
        }

        // Update system now-playing
        media_bridge::update_now_playing(event.title.as_deref(), event.artist.as_deref(), true);
    } else {
        // Paused
        {
            let mut state = state_mutex.lock().unwrap();
            if let Some(tab) = state.tabs.get_mut(&event.label) {
                tab.is_playing = false;
            }
            if state.playing_tab_key.as_deref() == Some(&event.label) {
                state.playing_tab_key = None;
            }
            debug!("State updated: {} paused", event.label);
        }

        media_bridge::update_now_playing(event.title.as_deref(), event.artist.as_deref(), false);
    }

    // Notify shell webview to refresh tab UI
    notify_shell(app);
}

fn handle_title_changed(app: &AppHandle, event: TitleChangedEvent) {
    let state_mutex = app.state::<Mutex<TabsState>>();
    let mut state = state_mutex.lock().unwrap();

    if let Some(tab) = state.tabs.get_mut(&event.label) {
        // Only update display name for multi-instance sources
        if tab.source.multi_instance() {
            let clean = clean_page_title(&event.title, &tab.source.name());
            if !clean.is_empty() {
                debug!("Tab {} title changed: {}", event.label, clean);
                tab.display_name = clean;
            }
        }
    }
    drop(state);
    notify_shell(app);
}

fn clean_page_title(title: &str, source_name: &str) -> String {
    // Strip common suffixes like " - YouTube", " | SoundCloud"
    let cleaned = title
        .trim_end_matches(&format!(" - {}", source_name))
        .trim_end_matches(&format!(" | {}", source_name))
        .trim_end_matches(&format!(" — {}", source_name))
        .trim();
    cleaned.to_string()
}

fn notify_shell(app: &AppHandle) {
    debug!("Emitting playback-changed to shell");
    match app.emit("playback-changed", ()) {
        Ok(_) => debug!("playback-changed emitted successfully"),
        Err(e) => warn!("Failed to emit playback-changed: {}", e),
    }
}
