use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use crate::error::RotivError;

/// A file system change event filtered to `.tsx`/`.ts` files.
#[derive(Debug, Clone)]
pub enum WatchEvent {
    Changed(PathBuf),
    Created(PathBuf),
    Deleted(PathBuf),
}

/// File watcher for the routes directory.
///
/// Uses `notify_debouncer_mini` with a 200ms debounce.
/// Falls back to polling when `ROTIV_FORCE_POLL=1` is set or on Windows.
pub struct FileWatcher {
    receiver: mpsc::Receiver<WatchEvent>,
    // Keep the watcher alive for the lifetime of this struct.
    _watcher: Box<dyn std::any::Any + Send>,
}

impl FileWatcher {
    /// Create a watcher for the given directory.
    pub fn new(dir: &Path) -> Result<Self, RotivError> {
        let use_polling = std::env::var("ROTIV_FORCE_POLL").is_ok()
            || cfg!(target_os = "windows");

        if use_polling {
            Self::new_poll(dir)
        } else {
            // Try recommended watcher first; fall back to poll on error.
            Self::new_recommended(dir).or_else(|_| Self::new_poll(dir))
        }
    }

    fn new_recommended(dir: &Path) -> Result<Self, RotivError> {
        #[allow(unused_imports)]
        use notify::Watcher;

        let (tx, rx) = mpsc::channel::<WatchEvent>();
        let dir_owned = dir.to_path_buf();

        let mut debouncer = notify_debouncer_mini::new_debouncer(
            Duration::from_millis(200),
            move |events: notify_debouncer_mini::DebounceEventResult| {
                if let Ok(events) = events {
                    for event in events {
                        let path = event.path;
                        if !is_route_file(&path) {
                            continue;
                        }
                        // Infer create/delete from file existence since debouncer
                        // does not expose fine-grained event kinds.
                        let watch_event = if !path.exists() {
                            WatchEvent::Deleted(path)
                        } else {
                            WatchEvent::Changed(path)
                        };
                        let _ = tx.send(watch_event);
                    }
                }
            },
        )
        .map_err(|e| RotivError::new("E_WATCHER", format!("failed to create file watcher: {e}")))?;

        debouncer
            .watcher()
            .watch(&dir_owned, notify::RecursiveMode::Recursive)
            .map_err(|e| {
                RotivError::new("E_WATCHER", format!("failed to watch directory: {e}"))
            })?;

        Ok(Self {
            receiver: rx,
            _watcher: Box::new(debouncer),
        })
    }

    fn new_poll(dir: &Path) -> Result<Self, RotivError> {
        use notify::{PollWatcher, RecursiveMode, Watcher};

        let (tx, rx) = mpsc::channel::<WatchEvent>();
        let dir_owned = dir.to_path_buf();

        let config = notify::Config::default()
            .with_poll_interval(Duration::from_millis(500));

        let mut watcher = PollWatcher::new(
            move |result: notify::Result<notify::Event>| {
                if let Ok(event) = result {
                    for path in event.paths {
                        if !is_route_file(&path) {
                            continue;
                        }
                        let watch_event = match event.kind {
                            notify::EventKind::Create(_) => WatchEvent::Created(path),
                            notify::EventKind::Remove(_) => WatchEvent::Deleted(path),
                            _ => WatchEvent::Changed(path),
                        };
                        let _ = tx.send(watch_event);
                    }
                }
            },
            config,
        )
        .map_err(|e| {
            RotivError::new("E_WATCHER", format!("failed to create poll watcher: {e}"))
        })?;

        watcher
            .watch(&dir_owned, RecursiveMode::Recursive)
            .map_err(|e| {
                RotivError::new("E_WATCHER", format!("failed to watch directory: {e}"))
            })?;

        Ok(Self {
            receiver: rx,
            _watcher: Box::new(watcher),
        })
    }

    /// Try to receive an event, returning `None` if no event is available.
    pub fn try_recv(&self) -> Option<WatchEvent> {
        self.receiver.try_recv().ok()
    }

    /// Block waiting for an event, up to `timeout`.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<WatchEvent> {
        self.receiver.recv_timeout(timeout).ok()
    }
}

fn is_route_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains(".rotiv") {
        return false;
    }
    matches!(path.extension().and_then(|e| e.to_str()), Some("tsx") | Some("ts"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn watcher_constructs_without_error() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("ROTIV_FORCE_POLL", "1");
        let result = FileWatcher::new(tmp.path());
        std::env::remove_var("ROTIV_FORCE_POLL");
        assert!(result.is_ok(), "FileWatcher::new should succeed: {:?}", result.err());
    }

    #[test]
    fn is_route_file_filters_correctly() {
        assert!(is_route_file(Path::new("app/routes/index.tsx")));
        assert!(is_route_file(Path::new("app/routes/api/users.ts")));
        assert!(!is_route_file(Path::new("app/routes/style.css")));
        assert!(!is_route_file(Path::new(".rotiv/spec.json")));
        assert!(!is_route_file(Path::new("app/.rotiv/context.md")));
    }
}
