use cosmic::iced::window;

#[derive(Clone, Debug)]
pub enum Message {
    ThemeChanged(bool),
    LightWpChanged(String),
    DarkWpChanged(String),
    TrayShow,
    WindowCloseRequested(window::Id),
    WindowClosed(window::Id),
    AboutOpen,
    AboutClose,
}
