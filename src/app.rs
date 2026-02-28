use cosmic::app::{Core, Task};
use cosmic::iced::{event, Event, Subscription};
use cosmic::{executor, ApplicationExt, Element};

use crate::message::Message;
use crate::{tray, wallpaper, watcher};
pub struct App {
    pub core: Core,
    pub light_wp: String,
    pub dark_wp: String,
    pub is_dark: Option<bool>,
    light_wp_error: Option<String>,
    dark_wp_error: Option<String>,
    /// Id of the currently open window, if any.
    window_id: Option<cosmic::iced::window::Id>,
}

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "io.github.nagyrenato.CosmicWallShift";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        let id = core.main_window_id().unwrap();
        let default = find_default_wallpaper();
        let mut app = App {
            core,
            light_wp: default.clone(),
            dark_wp: default,
            is_dark: None,
            light_wp_error: None,
            dark_wp_error: None,
            window_id: Some(id),
        };
        app.set_header_title("COSMIC Background Sync".into());
        let cmd = app.set_window_title("COSMIC Background Sync".into(), id);
        (app, cmd)
    }

    fn view(&self) -> Element<'_, Message> {
        use cosmic::widget;

        let theme_label = match self.is_dark {
            Some(true) => "Dark",
            Some(false) => "Light",
            None => "Detecting...",
        };

        widget::column()
            .push(
                widget::row()
                    .push(widget::text::title4("Current Theme:"))
                    .push(widget::text::title4(theme_label))
                    .spacing(8),
            )
            .push(widget::divider::horizontal::default())
            .push(widget::text("Light Wallpaper Path:"))
            .push(
                widget::text_input("e.g. /home/user/Light.png", &self.light_wp)
                    .on_input(Message::LightWpChanged),
            )
            .push_maybe(
                self.light_wp_error.as_deref().map(|e| widget::text(e).size(13)),
            )
            .push(widget::text("Dark Wallpaper Path:"))
            .push(
                widget::text_input("e.g. /home/user/Dark.png", &self.dark_wp)
                    .on_input(Message::DarkWpChanged),
            )
            .push_maybe(
                self.dark_wp_error.as_deref().map(|e| widget::text(e).size(13)),
            )
            .spacing(12)
            .padding(24)
            .into()
    }

    /// Delegate all dynamically-opened windows to the same view as the main window,
    /// since we only ever have one window type.
    fn view_window(&self, _id: cosmic::iced::window::Id) -> Element<'_, Message> {
        self.view()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeChanged(is_dark) => {
                if self.is_dark != Some(is_dark) {
                    self.is_dark = Some(is_dark);
                    let target_wp = if is_dark {
                        self.dark_wp.clone()
                    } else {
                        self.light_wp.clone()
                    };
                    if validate_image_path(&target_wp).is_none() {
                        self.apply_wallpaper(&target_wp, is_dark);
                    }
                }
            }
            Message::LightWpChanged(p) => {
                self.light_wp = p;
                self.light_wp_error = validate_image_path(&self.light_wp);
                if self.light_wp_error.is_none() && self.is_dark == Some(false) {
                    let wp = self.light_wp.clone();
                    self.apply_wallpaper(&wp, false);
                }
            }
            Message::DarkWpChanged(p) => {
                self.dark_wp = p;
                self.dark_wp_error = validate_image_path(&self.dark_wp);
                if self.dark_wp_error.is_none() && self.is_dark == Some(true) {
                    let wp = self.dark_wp.clone();
                    self.apply_wallpaper(&wp, true);
                }
            }
            Message::TrayShow => {
                if self.window_id.is_none() {
                    // Window was closed — create a fresh one.
                    // On Wayland, set_visible(false→true) is unreliable;
                    // close + reopen is the only guaranteed path.
                    let (new_id, open_task) = cosmic::iced::window::open(
                        cosmic::iced::window::Settings {
                            size: cosmic::iced::Size::new(560.0, 520.0),
                            exit_on_close_request: false,
                            ..Default::default()
                        },
                    );
                    self.window_id = Some(new_id);
                    // Set title once iced confirms the window is open.
                    let title = "COSMIC Background Sync".to_string();
                    self.set_header_title(title.clone());
                    let title_task = self.set_window_title(title, new_id);
                    return Task::batch(vec![
                        open_task.discard(),
                        title_task,
                        cosmic::iced_runtime::window::gain_focus(new_id),
                    ]);
                } else if let Some(id) = self.window_id {
                    return cosmic::iced_runtime::window::gain_focus(id);
                }
            }
            Message::WindowCloseRequested(id) => {
                // exit_on_close_request: false means this fires instead of auto-closing.
                // Explicitly close the surface; wait for Closed to clear tracking.
                return cosmic::iced::window::close(id);
            }
            Message::WindowClosed(id) => {
                // Window is fully gone — safe to clear the tracked id now.
                if self.window_id == Some(id) {
                    self.window_id = None;
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let close_events = event::listen_with(|e, _status, id| match e {
            Event::Window(cosmic::iced::window::Event::CloseRequested) => {
                Some(Message::WindowCloseRequested(id))
            }
            Event::Window(cosmic::iced::window::Event::Closed) => {
                Some(Message::WindowClosed(id))
            }
            _ => None,
        });

        Subscription::batch(vec![
            watcher::theme_watcher(),
            tray::subscription(),
            close_events,
        ])
    }
}

fn find_default_wallpaper() -> String {
    let search_dirs = [
        "/usr/share/backgrounds/cosmic",
        "/usr/share/backgrounds",
        "/usr/share/wallpapers",
    ];
    for dir in &search_dirs {
        if let Ok(mut entries) = std::fs::read_dir(dir) {
            if let Some(Ok(entry)) = entries.find(|e| {
                e.as_ref().ok().map_or(false, |e| {
                    validate_image_path(&e.path().to_string_lossy()).is_none()
                        && !e.path().to_string_lossy().is_empty()
                })
            }) {
                return entry.path().to_string_lossy().to_string();
            }
        }
    }
    String::new()
}

fn validate_image_path(path: &str) -> Option<String> {
    if path.trim().is_empty() {
        return None;
    }
    let p = std::path::Path::new(path);
    let valid_ext = matches!(
        p.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref(),
        Some("jpg") | Some("jpeg") | Some("png") | Some("webp")
    );
    if !valid_ext {
        return Some("Unsupported file type. Use jpg, jpeg, png or webp.".into());
    }
    if !p.exists() {
        return Some("File not found.".into());
    }
    None
}

impl App {
    fn apply_wallpaper(&mut self, path: &str, is_dark: bool) {
        let _ = wallpaper::apply(path, is_dark);
    }
}
