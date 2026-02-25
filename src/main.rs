mod app;
mod message;
mod tray;
mod wallpaper;
mod watcher;

use cosmic::app::Settings;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // exit_on_close(false): clicking × minimizes to tray instead of quitting.
    let settings = Settings::default()
        .size(cosmic::iced::Size::new(560.0, 520.0))
        .exit_on_close(false);
    cosmic::app::run::<app::App>(settings, ())?;
    Ok(())
}
