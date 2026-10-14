// SPDX-License-Identifier: MPL-2.0

mod accounts_dbus;
mod app;
mod config;
mod fprint_dbus;
mod i18n;

const WINDOW_MIN_WIDTH: f32 = 360.0;
const WINDOW_MIN_HEIGHT: f32 = 600.0;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(WINDOW_MIN_WIDTH)
            .min_height(WINDOW_MIN_HEIGHT),
    );

    let portal_is_dark = !config::is_cosmic_desktop() && config::read_portal_is_dark();

    // Starts the application's event loop with the portal color scheme as flags.
    cosmic::app::run::<app::AppModel>(settings, portal_is_dark)
}
