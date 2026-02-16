use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata as SouvlakiMetadata, MediaPlayback,
    PlatformConfig,
};

use crate::tabs_state::TabsState;

static MEDIA_CONTROLS: Mutex<Option<MediaControls>> = Mutex::new(None);

pub fn setup_media_keys(app: &AppHandle) {
    let config = PlatformConfig {
        dbus_name: "medya",
        display_name: "Medya",
        hwnd: None,
    };

    let controls = match MediaControls::new(config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create media controls: {:?}", e);
            return;
        }
    };

    *MEDIA_CONTROLS.lock().unwrap() = Some(controls);

    let app_handle = app.clone();
    let mut guard = MEDIA_CONTROLS.lock().unwrap();
    if let Some(ref mut controls) = *guard {
        let _ = controls.attach(move |event: MediaControlEvent| {
            handle_media_event(&app_handle, event);
        });
    }
}

fn handle_media_event(app: &AppHandle, event: MediaControlEvent) {
    match event {
        MediaControlEvent::Play => forward_play_pause(app, true),
        MediaControlEvent::Pause => forward_play_pause(app, false),
        MediaControlEvent::Toggle => forward_toggle(app),
        MediaControlEvent::Next => forward_next_previous(app, true),
        MediaControlEvent::Previous => forward_next_previous(app, false),
        _ => {}
    }
}

fn forward_play_pause(app: &AppHandle, play: bool) {
    let state_mutex = app.state::<Mutex<TabsState>>();
    let label = {
        let state = state_mutex.lock().unwrap();
        state
            .playing_tab_key
            .clone()
            .or_else(|| state.active_tab_key.clone())
    };

    if let Some(label) = label {
        if let Some(wv) = app.get_webview(&label) {
            let js = if play {
                "document.querySelector('video, audio')?.play();"
            } else {
                "document.querySelector('video, audio')?.pause();"
            };
            let _ = wv.eval(js);
        }
    }
}

fn forward_toggle(app: &AppHandle) {
    let state_mutex = app.state::<Mutex<TabsState>>();
    let label = {
        let state = state_mutex.lock().unwrap();
        state
            .playing_tab_key
            .clone()
            .or_else(|| state.active_tab_key.clone())
    };

    if let Some(label) = label {
        if let Some(wv) = app.get_webview(&label) {
            let _ = wv.eval(
                "(() => { const el = document.querySelector('video, audio'); if (el) { el.paused ? el.play() : el.pause(); } })();",
            );
        }
    }
}

fn forward_next_previous(app: &AppHandle, next: bool) {
    let state_mutex = app.state::<Mutex<TabsState>>();
    let (label, source_id) = {
        let state = state_mutex.lock().unwrap();
        let l = state
            .playing_tab_key
            .clone()
            .or_else(|| state.active_tab_key.clone());
        let sid = l
            .as_ref()
            .and_then(|l| state.tabs.get(l))
            .map(|t| t.source.clone());
        (l, sid)
    };

    if let (Some(label), Some(source)) = (label, source_id) {
        if let Some(wv) = app.get_webview(&label) {
            let selector = if next {
                source.next_selector()
            } else {
                source.previous_selector()
            };
            let js = format!("document.querySelector('{}')?.click();", selector);
            let _ = wv.eval(&js);
        }
    }
}

pub fn update_now_playing(title: Option<&str>, artist: Option<&str>, playing: bool) {
    let mut guard = MEDIA_CONTROLS.lock().unwrap();
    if let Some(ref mut controls) = *guard {
        let _ = controls.set_metadata(SouvlakiMetadata {
            title,
            artist,
            album: None,
            cover_url: None,
            duration: None,
        });

        let playback = if playing {
            MediaPlayback::Playing { progress: None }
        } else {
            MediaPlayback::Paused { progress: None }
        };
        let _ = controls.set_playback(playback);
    }
}
