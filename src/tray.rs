use cosmic::iced::futures::SinkExt;
use cosmic::iced::Subscription;
use cosmic::iced_futures::stream;
use ksni::TrayMethods;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

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

            // Retry spawning the tray icon with backoff. This handles the case
            // where the app autostarts before the StatusNotifierWatcher service
            // is ready on the D-Bus.
            let mut delay_secs = 1u64;
            let _handle = loop {
                match (AppTray { sender: event_tx.clone() }).spawn().await {
                    Ok(handle) => break Some(handle),
                    Err(e) => {
                        eprintln!(
                            "System tray unavailable (retrying in {delay_secs}s): {e}"
                        );
                        if delay_secs >= 30 {
                            eprintln!("System tray unavailable: giving up after retries.");
                            break None;
                        }
                        sleep(Duration::from_secs(delay_secs)).await;
                        delay_secs = (delay_secs * 2).min(30);
                    }
                }
            };

            while let Some(event) = event_rx.recv().await {
                let msg = match event {
                    TrayEvent::Show => Message::TrayShow,
                };
                let _ = tx.send(msg).await;
            }
        }),
    )
}
