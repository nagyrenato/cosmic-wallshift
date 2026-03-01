use cosmic::iced::futures::SinkExt;
use cosmic::iced::Subscription;
use cosmic::iced_futures::stream;
use ksni::TrayMethods;
use tokio::sync::mpsc;

use crate::message::Message;

// ── internal event ──────────────────────────────────────────────────────────

#[derive(Debug)]
enum TrayEvent {
    Show,
}

// ── ksni tray definition ─────────────────────────────────────────────────────

#[derive(Debug)]
struct AppTray {
    sender: mpsc::Sender<TrayEvent>,
}

impl ksni::Tray for AppTray {
    fn id(&self) -> String {
        "io.github.nagyrenato.CosmicWallShift".into()
    }

    fn icon_name(&self) -> String {
        "io.github.nagyrenato.CosmicWallShift".into()
    }

    fn title(&self) -> String {
        "COSMIC WallShift".into()
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Show Window".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.sender.try_send(TrayEvent::Show);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

// ── subscription ─────────────────────────────────────────────────────────────

/// Spawns the system-tray icon and emits [`Message::TrayShow`] when clicked.
pub fn subscription() -> Subscription<Message> {
    Subscription::run_with_id(
        "tray",
        stream::channel(4, |mut tx| async move {
            let (event_tx, mut event_rx) = mpsc::channel::<TrayEvent>(8);

            // _handle must stay alive so the tray remains registered.
            let _handle = AppTray { sender: event_tx }
                .spawn()
                .await
                .map_err(|e| eprintln!("System tray unavailable: {e}"))
                .ok();

            while let Some(event) = event_rx.recv().await {
                let msg = match event {
                    TrayEvent::Show => Message::TrayShow,
                };
                let _ = tx.send(msg).await;
            }
        }),
    )
}
