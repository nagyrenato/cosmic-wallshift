use cosmic::iced::window;

#[derive(Clone, Debug)]
pub enum Message {
    // System theme changed (true = dark mode).
    ThemeChanged(bool),
    // Light wallpaper path changed.
    LightWpChanged(String),
    // Dark wallpaper path changed.
    DarkWpChanged(String),
    // Show window requested.
    TrayShow,
    // Window close requested.
    WindowCloseRequested(window::Id),
    // Window closed.
    WindowClosed(window::Id),
    // Open about dialog.
    AboutOpen,
    // Close about dialog.
    AboutClose,
}
