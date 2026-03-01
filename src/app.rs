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
    /// Whether the About dialog is currently open.
    show_about: bool,
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
        let (saved_light, saved_dark) = load_paths();
        let mut app = App {
            core,
            light_wp: if saved_light.is_empty() { default.clone() } else { saved_light },
            dark_wp: if saved_dark.is_empty() { default } else { saved_dark },
            is_dark: None,
            light_wp_error: None,
            dark_wp_error: None,
            window_id: Some(id),
            show_about: false,
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

        let about_btn = widget::button::icon(
            widget::icon::from_name("help-about-symbolic"),
        )
        .on_press(Message::AboutOpen);

        let content = widget::column()
            .push(
                widget::row()
                    .push(widget::text::title4("Current Theme:"))
                    .push(widget::text::title4(theme_label))
                    .push(widget::horizontal_space())
                    .push(about_btn)
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
            .padding(24);

        let base = widget::layer_container(content)
            .layer(cosmic::cosmic_theme::Layer::Background)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill);

        if self.show_about {
            let version = build_version();
            let about_content = widget::column()
                .push(widget::text::title3("COSMIC Background Sync"))
                .push(widget::text(format!("Version: {}", version)))
                .push(widget::divider::horizontal::default())
                .push(
                    widget::row()
                        .push(widget::text::body("Author:"))
                        .push(widget::text::body("Renato Nagy"))
                        .spacing(6),
                )
                .push(
                    widget::row()
                        .push(widget::text::body("E-mail:"))
                        .push(widget::text::body("nagy.renato@hotmail.com"))
                        .spacing(6),
                )
                .push(
                    widget::row()
                        .push(widget::text::body("License:"))
                        .push(widget::text::body("MIT"))
                        .spacing(6),
                )
                .spacing(10)
                .padding(8);

            let dialog = widget::dialog()
                .title("About")
                .control(about_content)
                .primary_action(
                    widget::button::suggested("Close")
                        .on_press(Message::AboutClose),
                );

            cosmic::iced::widget::stack![base, dialog].into()
        } else {
            base.into()
        }
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
                if self.light_wp_error.is_none() {
                    save_paths(&self.light_wp, &self.dark_wp);
                    if self.is_dark == Some(false) {
                        let wp = self.light_wp.clone();
                        self.apply_wallpaper(&wp, false);
                    }
                }
            }
            Message::DarkWpChanged(p) => {
                self.dark_wp = p;
                self.dark_wp_error = validate_image_path(&self.dark_wp);
                if self.dark_wp_error.is_none() {
                    save_paths(&self.light_wp, &self.dark_wp);
                    if self.is_dark == Some(true) {
                        let wp = self.dark_wp.clone();
                        self.apply_wallpaper(&wp, true);
                    }
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
            Message::AboutOpen => {
                self.show_about = true;
            }
            Message::AboutClose => {
                self.show_about = false;
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

fn config_path() -> std::path::PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            std::path::PathBuf::from(home).join(".config")
        });
    base.join("cosmic").join("io.github.nagyrenato.CosmicWallShift")
}

fn save_paths(light: &str, dark: &str) {
    let dir = config_path();
    let _ = std::fs::create_dir_all(&dir);
    let content = format!("{light}\n{dark}\n");
    let _ = std::fs::write(dir.join("paths"), content);
}

fn load_paths() -> (String, String) {
    let file = config_path().join("paths");
    let Ok(content) = std::fs::read_to_string(&file) else {
        return (String::new(), String::new());
    };
    let mut lines = content.lines();
    let light = lines.next().unwrap_or_default().to_string();
    let dark = lines.next().unwrap_or_default().to_string();
    (light, dark)
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

/// Returns a version string like "0.1.2026.03.01-1-12",
/// combining the Cargo package version with the build timestamp.
fn build_version() -> String {
    let pkg = env!("CARGO_PKG_VERSION"); // e.g. "0.1.0"
    let dt = env!("BUILD_DATE_TIME");    // e.g. "2026.03.01-1-12"
    let mut parts = pkg.splitn(3, '.');
    let major = parts.next().unwrap_or("0");
    let minor = parts.next().unwrap_or("1");
    format!("{}.{}.{}", major, minor, dt)
}

