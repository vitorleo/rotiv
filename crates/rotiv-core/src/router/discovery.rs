use std::path::{Path, PathBuf};

use crate::error::RotivError;

/// A discovered route entry mapping a file to an HTTP path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteEntry {
    /// Absolute path to the route file (`.tsx` or `.ts`).
    pub file_path: PathBuf,
    /// HTTP path, e.g. `/`, `/about`, `/users/:id`.
    pub route_path: String,
    /// True if this is an API-only route (`.ts` extension or path starts with `/api/`).
    pub is_api_only: bool,
}

/// Walk `routes_dir` recursively and return all discovered routes, sorted
/// so exact segments always come before parameterized ones.
pub fn discover_routes(routes_dir: &Path) -> Result<Vec<RouteEntry>, RotivError> {
    if !routes_dir.exists() {
        return Err(RotivError::new(
            "E_ROUTES_DIR_NOT_FOUND",
            format!("routes directory not found: {}", routes_dir.display()),
        )
        .with_suggestion("Make sure you are inside a Rotiv project with an app/routes/ directory"));
    }

    let mut entries: Vec<RouteEntry> = Vec::new();
    collect_routes(routes_dir, routes_dir, &mut entries)?;
    entries.sort_by(|a, b| sort_key(&a.route_path).cmp(&sort_key(&b.route_path)));
    Ok(entries)
}

fn collect_routes(
    base: &Path,
    dir: &Path,
    entries: &mut Vec<RouteEntry>,
) -> Result<(), RotivError> {
    let read_dir = std::fs::read_dir(dir).map_err(|e| {
        RotivError::new("E_IO", e.to_string()).with_file(dir.display().to_string(), None)
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| RotivError::new("E_IO", e.to_string()))?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Skip hidden files, underscore-prefixed files, and .rotiv dir
        if file_name.starts_with('.') || file_name.starts_with('_') {
            continue;
        }

        if path.is_dir() {
            collect_routes(base, &path, entries)?;
            continue;
        }

        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if ext != "tsx" && ext != "ts" {
            continue;
        }

        let route_path = file_to_route_path(base, &path);
        let is_api_only = ext == "ts" || route_path.starts_with("/api/");

        entries.push(RouteEntry {
            file_path: path.canonicalize().unwrap_or(path),
            route_path,
            is_api_only,
        });
    }

    Ok(())
}

/// Convert an absolute file path to an HTTP route path.
///
/// Examples:
/// - `routes/index.tsx`       → `/`
/// - `routes/about.tsx`       → `/about`
/// - `routes/users/index.tsx` → `/users`
/// - `routes/users/[id].tsx`  → `/users/:id`
pub fn file_to_route_path(base: &Path, file: &Path) -> String {
    // Strip the base prefix and extension
    let relative = file.strip_prefix(base).unwrap_or(file);
    let without_ext = relative.with_extension("");

    // Build segments from path components
    let mut segments: Vec<String> = without_ext
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    // Drop trailing "index" segment
    if segments.last().map(|s| s.as_str()) == Some("index") {
        segments.pop();
    }

    if segments.is_empty() {
        return "/".to_string();
    }

    // Convert [param] to :param
    let segments: Vec<String> = segments
        .into_iter()
        .map(|s| {
            if s.starts_with('[') && s.ends_with(']') {
                format!(":{}", &s[1..s.len() - 1])
            } else {
                s
            }
        })
        .collect();

    format!("/{}", segments.join("/"))
}

/// Sort key that places exact segments before parameterized ones.
/// `:id` → `~id` (tilde sorts after all letters).
pub fn sort_key(route_path: &str) -> String {
    route_path.replace(':', "~")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_route_file(dir: &Path, rel_path: &str) {
        let full = dir.join(rel_path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full, "export default {};").unwrap();
    }

    #[test]
    fn index_tsx_maps_to_root() {
        let tmp = TempDir::new().unwrap();
        make_route_file(tmp.path(), "index.tsx");
        let routes = discover_routes(tmp.path()).unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].route_path, "/");
        assert!(!routes[0].is_api_only);
    }

    #[test]
    fn about_tsx_maps_to_about() {
        let tmp = TempDir::new().unwrap();
        make_route_file(tmp.path(), "about.tsx");
        let routes = discover_routes(tmp.path()).unwrap();
        assert_eq!(routes[0].route_path, "/about");
    }

    #[test]
    fn dynamic_param_maps_to_colon_syntax() {
        let tmp = TempDir::new().unwrap();
        make_route_file(tmp.path(), "users/[id].tsx");
        let routes = discover_routes(tmp.path()).unwrap();
        assert_eq!(routes[0].route_path, "/users/:id");
    }

    #[test]
    fn users_index_maps_to_users() {
        let tmp = TempDir::new().unwrap();
        make_route_file(tmp.path(), "users/index.tsx");
        let routes = discover_routes(tmp.path()).unwrap();
        assert_eq!(routes[0].route_path, "/users");
    }

    #[test]
    fn empty_dir_returns_empty_vec() {
        let tmp = TempDir::new().unwrap();
        let routes = discover_routes(tmp.path()).unwrap();
        assert!(routes.is_empty());
    }

    #[test]
    fn ts_extension_is_api_only() {
        let tmp = TempDir::new().unwrap();
        make_route_file(tmp.path(), "api/users.ts");
        let routes = discover_routes(tmp.path()).unwrap();
        assert!(routes[0].is_api_only);
    }

    #[test]
    fn underscore_files_are_skipped() {
        let tmp = TempDir::new().unwrap();
        make_route_file(tmp.path(), "_helpers.tsx");
        make_route_file(tmp.path(), "index.tsx");
        let routes = discover_routes(tmp.path()).unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].route_path, "/");
    }

    #[test]
    fn exact_routes_sort_before_parameterized() {
        let tmp = TempDir::new().unwrap();
        make_route_file(tmp.path(), "users/[id].tsx");
        make_route_file(tmp.path(), "users/index.tsx");
        let routes = discover_routes(tmp.path()).unwrap();
        assert_eq!(routes[0].route_path, "/users");
        assert_eq!(routes[1].route_path, "/users/:id");
    }

    #[test]
    fn nonexistent_dir_returns_error() {
        let result = discover_routes(Path::new("/nonexistent/routes"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "E_ROUTES_DIR_NOT_FOUND");
    }
}
