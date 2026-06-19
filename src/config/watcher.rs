use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::watch;

/// Watch a config file for changes and emit reload events on a watch channel.
pub fn watch_config(path: PathBuf) -> Result<watch::Receiver<()>> {
    let (tx, rx) = watch::channel(());

    let tx_clone = tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                let _ = tx_clone.send(());
            }
        }
    })
    .map_err(|e| anyhow::anyhow!("Failed to create file watcher: {}", e))?;

    let parent = path.parent().unwrap_or(Path::new("."));
    watcher
        .watch(parent, RecursiveMode::NonRecursive)
        .map_err(|e| anyhow::anyhow!("Failed to watch path: {}", e))?;

    // Store watcher in a leaked Box to keep it alive for the process lifetime
    let _leaked = Box::new(watcher);
    std::mem::forget(_leaked);

    Ok(rx)
}

/// Apply config changes from a reload event.
pub fn apply_reload(
    config: &mut super::Config,
    reload_rx: &mut watch::Receiver<()>,
) -> bool {
    if reload_rx.has_changed().unwrap_or(false) {
        tracing::info!("Config file changed, reloading...");
        // In production: re-read the config file and merge changes
        // For now: mark as consumed and signal that reload happened
        reload_rx.borrow_and_update();
        return true;
    }
    false
}
