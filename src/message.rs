use cosmic::iced::window;

#[derive(Clone, Debug)]
pub enum Message {
    /// The system theme changed; `true` = dark mode.
    ThemeChanged(bool),
    /// The user edited the light wallpaper path.
    LightWpChanged(String),
    /// The user edited the dark wallpaper path.
    DarkWpChanged(String),
    /// "Show Window" was clicked in the system tray.
    TrayShow,
    /// The window × button was pressed — tell iced to close it.
    WindowCloseRequested(window::Id),
    /// The window surface has been fully destroyed — clear tracking.
    WindowClosed(window::Id),
    /// Open the About dialog.
    AboutOpen,
    /// Close the About dialog.
    AboutClose,
}
