use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::error::RotivError;
use super::discovery::{RouteEntry, discover_routes};
use super::matcher::matches;

/// Thread-safe route registry. Wrap in `Arc<RwLock<RouteRegistry>>` for shared access.
pub struct RouteRegistry {
    routes_dir: PathBuf,
    entries: Vec<RouteEntry>,
}

impl RouteRegistry {
    pub fn new(routes_dir: PathBuf) -> Self {
        Self {
            routes_dir,
            entries: Vec::new(),
        }
    }

    /// Discover routes from the filesystem and populate the registry.
    pub fn load(&mut self) -> Result<(), RotivError> {
        self.entries = discover_routes(&self.routes_dir)?;
        Ok(())
    }

    /// Re-discover routes (called on file system changes).
    pub fn reload(&mut self) -> Result<(), RotivError> {
        self.load()
    }

    /// All registered route entries.
    pub fn entries(&self) -> &[RouteEntry] {
        &self.entries
    }

    /// Find the entry whose route_path matches `request_path`.
    /// Handles parameterized routes (`:id`).
    pub fn find_by_path(&self, request_path: &str) -> Option<&RouteEntry> {
        // First try exact match
        if let Some(entry) = self.entries.iter().find(|e| e.route_path == request_path) {
            return Some(entry);
        }
        // Then try parameterized match
        self.entries
            .iter()
            .find(|e| matches(&e.route_path, request_path).is_some())
    }

    /// Extract path params for a given request path against the matched route.
    pub fn extract_params(
        &self,
        entry: &RouteEntry,
        request_path: &str,
    ) -> std::collections::HashMap<String, String> {
        matches(&entry.route_path, request_path).unwrap_or_default()
    }
}

/// Convenience type alias for shared registry.
pub type SharedRegistry = Arc<RwLock<RouteRegistry>>;

/// Create a new shared registry.
pub fn new_shared_registry(routes_dir: PathBuf) -> SharedRegistry {
    Arc::new(RwLock::new(RouteRegistry::new(routes_dir)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_registry(files: &[&str]) -> (TempDir, RouteRegistry) {
        let tmp = TempDir::new().unwrap();
        for f in files {
            let path = tmp.path().join(f);
            if let Some(p) = path.parent() {
                fs::create_dir_all(p).unwrap();
            }
            fs::write(&path, "export default {};").unwrap();
        }
        let mut reg = RouteRegistry::new(tmp.path().to_path_buf());
        reg.load().unwrap();
        (tmp, reg)
    }

    #[test]
    fn find_exact_route() {
        let (_tmp, reg) = setup_registry(&["index.tsx", "about.tsx"]);
        let entry = reg.find_by_path("/about").unwrap();
        assert_eq!(entry.route_path, "/about");
    }

    #[test]
    fn find_root_route() {
        let (_tmp, reg) = setup_registry(&["index.tsx"]);
        let entry = reg.find_by_path("/").unwrap();
        assert_eq!(entry.route_path, "/");
    }

    #[test]
    fn find_parameterized_route() {
        let (_tmp, reg) = setup_registry(&["users/[id].tsx"]);
        let entry = reg.find_by_path("/users/42").unwrap();
        assert_eq!(entry.route_path, "/users/:id");
    }

    #[test]
    fn no_match_returns_none() {
        let (_tmp, reg) = setup_registry(&["index.tsx"]);
        assert!(reg.find_by_path("/missing").is_none());
    }

    #[test]
    fn exact_before_parameterized() {
        let (_tmp, reg) = setup_registry(&["users/index.tsx", "users/[id].tsx"]);
        let entry = reg.find_by_path("/users").unwrap();
        assert_eq!(entry.route_path, "/users");
    }
}
