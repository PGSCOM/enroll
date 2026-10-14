// SPDX-License-Identifier: MPL-2.0
use ashpd::desktop::settings::{ColorScheme, Settings};
use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use cosmic::{Theme, theme};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::error;

// AppTheme is directly copied from https://github.com/cosmic-utils/camera
/// Application theme preference
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Hash)]
pub enum AppTheme {
    /// Follow system theme (dark or light based on system setting)
    #[default]
    System,
    /// Always use dark theme
    Dark,
    /// Always use light theme
    Light,
}

impl AppTheme {
    /// Get the COSMIC theme for this app theme preference.
    ///
    /// On non-COSMIC desktops, `system_dark()`/`system_light()`/`system_preference()`
    /// read broken defaults from cosmic_config, so we use built-in themes instead,
    /// following `portal_is_dark` in `System` mode.
    pub fn theme(&self, portal_is_dark: bool) -> Theme {
        if is_cosmic_desktop() {
            match self {
                Self::Dark => {
                    let mut t = theme::system_dark();
                    t.theme_type.prefer_dark(Some(true));
                    t
                }
                Self::Light => {
                    let mut t = theme::system_light();
                    t.theme_type.prefer_dark(Some(false));
                    t
                }
                Self::System => theme::system_preference(),
            }
        } else {
            let is_dark = match self {
                Self::System => portal_is_dark,
                Self::Dark => true,
                Self::Light => false,
            };
            if is_dark {
                Theme::dark()
            } else {
                Theme::light()
            }
        }
    }
}

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 2]
pub struct Config {
    pub app_theme: AppTheme,
    pub experimental_ui: bool,
}

/// Whether we're running on the COSMIC desktop (cached for process lifetime).
pub fn is_cosmic_desktop() -> bool {
    static IS_COSMIC: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| {
        std::env::var("XDG_CURRENT_DESKTOP")
            .map(|d| d.to_ascii_uppercase().contains("COSMIC"))
            .unwrap_or(false)
    });
    *IS_COSMIC
}

/// Whether the XDG portal color scheme is dark.
///
/// `NoPreference` counts as light, as it is what light desktops report by default (e.g. GNOME).
pub fn is_dark_scheme(scheme: ColorScheme) -> bool {
    scheme == ColorScheme::PreferDark
}

/// Reads the XDG portal color scheme, blocking for at most one second.
///
/// Must be called outside of an async runtime.
pub fn read_portal_is_dark() -> bool {
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return false;
    };
    runtime.block_on(async {
        // Own connection: ashpd's shared one would die with this runtime
        let read = async {
            let connection = zbus::Connection::session().await?;
            Settings::with_connection(connection)
                .await?
                .color_scheme()
                .await
        };
        matches!(
            tokio::time::timeout(Duration::from_secs(1), read).await,
            Ok(Ok(scheme)) if is_dark_scheme(scheme)
        )
    })
}

/// Attempts to read current version of a COSMIC Config or create one
pub fn read_config(app_id: &str) -> (Option<cosmic_config::Config>, Config) {
    match cosmic_config::Config::new(app_id, Config::VERSION) {
        Ok(handler) => {
            let config = match Config::get_entry(&handler) {
                Ok(config) => config,
                Err((errors, config)) => {
                    error!(?errors, "Errors loading config");
                    config
                }
            };
            (Some(handler), config)
        }
        Err(err) => {
            error!(%err, "Failed to create config handler");
            (None, Config::default())
        }
    }
}
