use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaDefinition {
    id: MediaSource,
    name: &'static str,
    icon_url: &'static str,
    default_url: &'static str,
    multi_instance: bool,
}

macro_rules! define_sources {
    (
        $(
            $variant:ident {
                name: $name:expr,
                icon_url: $icon_url:expr,
                default_url: $default_url:expr,
                allowed_origins: [$($origin:expr),* $(,)?],
                multi_instance: $multi:expr,
                init_script: $init_script:expr,
                next_selector: $next_selector:expr,
                previous_selector: $previous_selector:expr,
            }
        ),* $(,)?
    ) => {
        #[derive(Deserialize, Serialize, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
        pub enum MediaSource {
            $($variant,)*
        }

        impl MediaSource {
            pub const ALL: &'static [MediaSource] = &[
                $(MediaSource::$variant,)*
            ];

            pub fn source_id(self) -> String {
                 match serde_json::to_value(&self) {
                     Ok(Value::String(value)) => value,
                     e => unreachable!("Invalid json value: {e:?}"),
                 }
            }

            pub fn name(self) -> &'static str {
                match self {
                    $(MediaSource::$variant => $name,)*
                }
            }

            pub fn icon_url(&self) -> &'static str {
                match self {
                    $(MediaSource::$variant => $icon_url,)*
                }
            }

            pub fn default_url(self) -> &'static str {
                match self {
                    $(MediaSource::$variant => $default_url,)*
                }
            }

            pub fn allowed_origins(self) -> &'static [&'static str] {
                match self {
                    $(MediaSource::$variant => &[$($origin,)*],)*
                }
            }

            pub fn multi_instance(self) -> bool {
                match self {
                    $(MediaSource::$variant => $multi,)*
                }
            }

            pub fn init_script(self, tab_label: &str) -> String {
                const BASE_SCRIPT: &str = include_str!("../scripts/base_monitor.js");
                let metadata = match self {
                    $(MediaSource::$variant => $init_script,)*
                };

                format!(
                    r#"(() => {{
                        const TAB_LABEL = '{tab_label}';
                        const SOURCE_ID = '{}';
                        {BASE_SCRIPT}
                        {metadata}
                    }})();"#,
                    self.source_id()
                )
            }

            pub fn next_selector(self) -> &'static str {
                match self {
                    $(MediaSource::$variant => $next_selector,)*
                }
            }

            pub fn previous_selector(self) -> &'static str {
                match self {
                    $(MediaSource::$variant => $previous_selector,)*
                }
            }

            pub fn definition(&self) -> MediaDefinition {
                MediaDefinition {
                    id: *self,
                    name: self.name(),
                    icon_url: self.icon_url(),
                    default_url: self.default_url(),
                    multi_instance: self.multi_instance(),
                }
            }

            pub fn next_tab_key(self) -> String {
                format!("{}-{}", self.source_id(), Alphanumeric.sample_string(&mut rand::rng(), 6))
            }
        }
    };
}

define_sources! {
    AppleMusic {
        name: "Apple Music",
        icon_url: "https://music.apple.com/assets/favicon/favicon-180.png",
        default_url: "https://music.apple.com",
        allowed_origins: [
            "https://music.apple.com",
            "https://appleid.apple.com",
            "https://idmsa.apple.com",
        ],
        multi_instance: false,
        init_script: include_str!("../scripts/apple-music_metadata.js"),
        next_selector: "button[aria-label=\"Next\"], .web-chrome-playback-controls__next",
        previous_selector: "button[aria-label=\"Previous\"], .web-chrome-playback-controls__previous",
    },
    YouTube {
        name: "YouTube",
        icon_url: "https://www.youtube.com/img/favicon_144.png",
        default_url: "https://www.youtube.com",
        allowed_origins: [
            "https://www.youtube.com",
            "https://youtube.com",
            "https://accounts.google.com",
            "https://accounts.youtube.com",
            "https://consent.youtube.com",
            "https://consent.google.com",
            "https://myaccount.google.com",
        ],
        multi_instance: true,
        init_script: include_str!("../scripts/youtube_metadata.js"),
        next_selector: "button.ytp-next-button, a.ytp-next-button, .ytp-next-button",
        previous_selector: "button.ytp-prev-button, a.ytp-prev-button, .ytp-prev-button",
    },
    SoundCloud {
        name: "SoundCloud",
        icon_url: "https://a-v2.sndcdn.com/assets/images/sc-icons/favicon-2cadd14bdb.ico",
        default_url: "https://soundcloud.com",
        allowed_origins: [
            "https://soundcloud.com",
            "https://secure.soundcloud.com",
            "https://api-v2.soundcloud.com",
            "https://accounts.google.com",
        ],
        multi_instance: false,
        init_script: include_str!("../scripts/soundcloud_metadata.js"),
        next_selector: "button.skipControl__next, button[aria-label=\"Next\"]",
        previous_selector: "button.skipControl__previous, button[aria-label=\"Previous\"]",
    },
}
