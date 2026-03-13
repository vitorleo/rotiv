pub mod error;
pub mod models;
pub mod project;
pub mod proxy;
pub mod router;
pub mod server;
pub mod watcher;
pub mod worker;

pub use error::RotivError;
pub use models::{discover_models, ModelEntry};
pub use project::find_project_root;
pub use proxy::{InvokeRequest, InvokeResponse, invoke_route};
pub use router::{RouteEntry, RouteRegistry, SharedRegistry, new_shared_registry};
pub use server::{DevServer, DevServerConfig};
pub use watcher::{FileWatcher, WatchEvent};
pub use worker::{RouteWorker, resolve_worker_path};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotiv_error_creates_correctly() {
        let err = RotivError::new("E001", "Something went wrong");
        assert_eq!(err.code, "E001");
        assert_eq!(err.message, "Something went wrong");
        assert!(err.suggestion.is_none());
    }

    #[test]
    fn rotiv_error_with_suggestion() {
        let err = RotivError::new("E002", "Directory not found")
            .with_suggestion("Run `rotiv new <name>` to create a project");
        assert!(err.suggestion.is_some());
    }

    #[test]
    fn rotiv_error_serializes_to_json() {
        let err = RotivError::new("E001", "test error");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("E001"));
        assert!(json.contains("test error"));
    }
}
