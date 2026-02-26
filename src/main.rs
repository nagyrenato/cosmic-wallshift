mod app;
mod message;
mod tray;
mod wallpaper;
mod watcher;

use cosmic::app::Settings;
use std::os::unix::net::UnixListener;

fn acquire_instance_lock() -> Option<UnixListener> {
    let uid = unsafe { libc::getuid() };
    let socket_path = format!("/run/user/{}/cosmic-wallshift.lock", uid);

    // If we can connect, a live instance is already listening — bail out.
    if std::os::unix::net::UnixStream::connect(&socket_path).is_ok() {
        return None;
    }

    // Connection failed → socket is stale or absent. Remove and (re)create it.
    let _ = std::fs::remove_file(&socket_path);

    match UnixListener::bind(&socket_path) {
        Ok(listener) => Some(listener),
        Err(_) => None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure only one instance runs at a time.
    let _lock = match acquire_instance_lock() {
        Some(l) => l,
        None => {
            eprintln!("cosmic-wallshift is already running.");
            std::process::exit(1);
        }
    };

    // exit_on_close(false): clicking × minimizes to tray instead of quitting.
    let settings = Settings::default()
        .size(cosmic::iced::Size::new(560.0, 520.0))
        .exit_on_close(false);
    cosmic::app::run::<app::App>(settings, ())?;
    Ok(())
}
