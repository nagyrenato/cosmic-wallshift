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
            let config_home = std::env::var("XDG_CONFIG_HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_default();
                    std::path::PathBuf::from(home).join(".config")
                });
            let theme_file = config_home
                .join("cosmic/com.system76.CosmicTheme.Mode/v1/is_dark")
                .to_string_lossy()
                .to_string();
            // Watch the parent directory so we keep getting events even when the
            // file is replaced atomically (delete + recreate), which would otherwise
            // silently break an inotify watch on the file's inode.
            let theme_dir = std::path::Path::new(&theme_file)
                .parent()
                .expect("theme file has no parent dir")
                .to_path_buf();
            let target_name = std::path::Path::new(&theme_file)
                .file_name()
                .expect("theme file has no name")
                .to_os_string();

            // Bridge notify's sync callback into an async channel.
            let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel::<()>(8);
            let target_name_cb = target_name.clone();
            let mut watcher =
                notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    if let Ok(event) = res {
                        use notify::EventKind::*;
                        if matches!(event.kind, Modify(_) | Create(_) | Remove(_)) {
                            let relevant = event.paths.iter().any(|p| {
                                p.file_name().map_or(false, |n| n == target_name_cb)
                            });
                            if relevant {
                                let _ = notify_tx.blocking_send(());
                            }
                        }
                    }
                })
                .expect("failed to create watcher");

            watcher
                .watch(&theme_dir, notify::RecursiveMode::NonRecursive)
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
