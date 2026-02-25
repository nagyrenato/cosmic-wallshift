use cosmic::iced::futures::SinkExt;
use cosmic::iced::Subscription;
use cosmic::iced_futures::stream;
use notify::Watcher;

use crate::message::Message;

/// Returns a [`Subscription`] that watches the COSMIC theme-mode file with inotify
/// and emits a [`Message::ThemeChanged`] whenever the file changes.
pub fn theme_watcher() -> Subscription<Message> {
    Subscription::run_with_id(
        "theme-watcher",
        stream::channel(4, |mut tx| async move {
            let user = std::env::var("USER").unwrap_or_default();
            let theme_file = format!(
                "/home/{}/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark",
                user
            );

            // Bridge notify's sync callback into an async channel.
            let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel::<()>(8);
            let mut watcher =
                notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    if let Ok(event) = res {
                        use notify::EventKind::*;
                        if matches!(event.kind, Modify(_) | Create(_) | Remove(_)) {
                            let _ = notify_tx.blocking_send(());
                        }
                    }
                })
                .expect("failed to create watcher");

            watcher
                .watch(
                    std::path::Path::new(&theme_file),
                    notify::RecursiveMode::NonRecursive,
                )
                .unwrap_or_else(|e| eprintln!("watch error: {e}"));

            // Emit initial state.
            let mut last = String::new();
            if let Ok(content) = tokio::fs::read_to_string(&theme_file).await {
                last = content.trim().to_string();
                let _ = tx.send(Message::ThemeChanged(last == "true")).await;
            }

            // React to filesystem events instead of polling.
            while notify_rx.recv().await.is_some() {
                if let Ok(content) = tokio::fs::read_to_string(&theme_file).await {
                    let current = content.trim().to_string();
                    if current != last {
                        let _ = tx.send(Message::ThemeChanged(current == "true")).await;
                        last = current;
                    }
                }
            }
        }),
    )
}
