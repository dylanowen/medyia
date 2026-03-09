// use std::sync::Mutex;
// use tauri::AppHandle;
//
// use crate::state::EnhancerAppStateManagerEmitter;
// use crate::EnhancedResult;
// use souvlaki::{
//     MediaControlEvent, MediaControls, MediaMetadata as SouvlakiMetadata, MediaPlayback,
//     PlatformConfig,
// };
//
// static MEDIA_CONTROLS: Mutex<Option<MediaControls>> = Mutex::new(None);
//
// pub fn setup_media_keys(app: &AppHandle) -> anyhow::Result<()> {
//     // let config = PlatformConfig {
//     //     dbus_name: &app.config().identifier,
//     //     display_name: app
//     //         .config()
//     //         .product_name
//     //         .as_ref()
//     //         .map(String::as_str)
//     //         .unwrap_or_else(|| env!("CARGO_PKG_NAME")),
//     //     hwnd: None,
//     // };
//     //
//     // let controls = MediaControls::new(config)?;
//     //
//     // *MEDIA_CONTROLS.lock().unwrap() = Some(controls);
//     //
//     // let app_handle = app.clone();
//     // let mut guard = MEDIA_CONTROLS.lock().unwrap();
//     // if let Some(ref mut controls) = *guard {
//     //     controls.attach(move |event: MediaControlEvent| {
//     //         handle_media_event(&app_handle, event).log_error();
//     //     })?;
//     // }
//
//     Ok(())
// }
//
// fn handle_media_event(app: &AppHandle, event: MediaControlEvent) -> anyhow::Result<()> {
//     app.app_state_mut(|state| {
//         match event {
//             MediaControlEvent::Play => state.play_active_tab(app)?,
//             MediaControlEvent::Pause => {
//                 state.pause_playing_tab();
//             }
//             MediaControlEvent::Toggle => state.toggle_playing(app)?,
//             MediaControlEvent::Next => {
//                 if let Some(tab) = state.playing_tab_mut() {
//                     tab.next();
//                 }
//             }
//             MediaControlEvent::Previous => {
//                 if let Some(tab) = state.playing_tab_mut() {
//                     tab.previous();
//                 }
//             }
//             _ => (),
//         }
//
//         Ok(())
//     })
// }
//
// pub fn update_now_playing(title: Option<&str>, artist: Option<&str>, playing: bool) {
//     let mut guard = MEDIA_CONTROLS.lock().unwrap();
//     if let Some(ref mut controls) = *guard {
//         let _ = controls.set_metadata(SouvlakiMetadata {
//             title,
//             artist,
//             album: None,
//             cover_url: None,
//             duration: None,
//         });
//
//         let playback = if playing {
//             MediaPlayback::Playing { progress: None }
//         } else {
//             MediaPlayback::Paused { progress: None }
//         };
//         let _ = controls.set_playback(playback);
//     }
// }
