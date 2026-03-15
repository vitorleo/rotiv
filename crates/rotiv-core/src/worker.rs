use std::path::PathBuf;
use std::time::Duration;

use tokio::process::Child;

use crate::error::RotivError;

/// Manages the lifecycle of the Node.js route-worker child process.
pub struct RouteWorker {
    process: Option<Child>,
    pub port: u16,
    project_dir: PathBuf,
    worker_path: PathBuf,
}

impl RouteWorker {
    /// Create a new worker manager. Does not start the process.
    ///
    /// `embedded_path` — if provided, use this path directly (highest priority after env var).
    /// This is set by the CLI when it has written the embedded worker source to a temp directory.
    pub fn new(project_dir: PathBuf, port: u16, embedded_path: Option<PathBuf>) -> Result<Self, RotivError> {
        let worker_path = resolve_worker_path(embedded_path)?;
        Ok(Self {
            process: None,
            port,
            project_dir,
            worker_path,
        })
    }

    /// Start the Node.js route-worker process.
    pub async fn start(&mut self) -> Result<(), RotivError> {
        if self.process.is_some() {
            return Ok(()); // already running
        }

        // Run from the project directory so that user's node_modules are available.
        // When running from a temp dir (embedded worker), fall back to the parent
        // of the worker file so relative imports within the worker still resolve.
        let worker_package_dir = {
            let parent = self.worker_path.parent().and_then(|p| p.parent()); // src/ -> package root
            // Prefer project_dir if it has node_modules (standalone install case)
            if self.project_dir.join("node_modules").exists() {
                self.project_dir.clone()
            } else if let Some(p) = parent {
                p.to_path_buf()
            } else {
                self.project_dir.clone()
            }
        };

        // Expose node_modules via NODE_PATH so that compiled .mjs files loaded
        // from the OS temp cache can resolve @rotiv/* packages.
        let node_modules = worker_package_dir.join("node_modules");

        // Resolve tsx loader — prefer project node_modules, fall back to binary-adjacent copy.
        let tsx_loader = resolve_tsx_loader(&self.project_dir);

        let child = tokio::process::Command::new("node")
            .arg("--import")
            .arg(&tsx_loader)
            .arg(&self.worker_path)
            .current_dir(&worker_package_dir)
            .env("ROTIV_WORKER_PORT", self.port.to_string())
            .env("ROTIV_PROJECT_DIR", self.project_dir.display().to_string())
            .env("NODE_PATH", node_modules)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| {
                RotivError::new("E_WORKER_SPAWN", format!("failed to start route worker: {e}"))
                    .with_suggestion(
                        "Make sure Node.js is installed and available in your PATH. \
                         Install tsx with: npm install -g tsx",
                    )
            })?;

        self.process = Some(child);
        Ok(())
    }

    /// Wait for the worker to be ready by polling the health endpoint.
    pub async fn wait_ready(&self, timeout: Duration) -> Result<(), RotivError> {
        let url = format!("http://127.0.0.1:{}/_rotiv/health", self.port);
        let client = reqwest::Client::new();
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            if tokio::time::Instant::now() >= deadline {
                return Err(RotivError::new(
                    "E_WORKER_TIMEOUT",
                    format!(
                        "route worker did not become ready within {}s",
                        timeout.as_secs()
                    ),
                )
                .with_suggestion(
                    "Check that Node.js and tsx are installed: node --version && npx tsx --version",
                ));
            }

            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                _ => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Stop the route-worker process.
    pub async fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill().await;
        }
    }

    /// Returns true if the worker process is currently running.
    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }
}

impl Drop for RouteWorker {
    fn drop(&mut self) {
        // Best-effort synchronous kill on drop (e.g., on Ctrl+C).
        if let Some(mut child) = self.process.take() {
            let _ = child.start_kill();
        }
    }
}

/// Resolve the `tsx` ESM loader specifier for use with `node --import`.
///
/// Resolution order:
/// 1. `project_dir/node_modules/tsx/dist/esm/index.cjs` (user has tsx installed)
/// 2. Binary-adjacent `node_modules/tsx/dist/esm/index.cjs` (bundled with rotiv install)
/// 3. Fall back to bare `"tsx"` (relies on NODE_PATH or global install)
pub fn resolve_tsx_loader(project_dir: &std::path::Path) -> String {
    let tsx_subpath = ["tsx", "dist", "esm", "index.cjs"].iter().collect::<std::path::PathBuf>();

    // 1. Project node_modules
    let project_tsx = project_dir.join("node_modules").join(&tsx_subpath);
    if project_tsx.exists() {
        return project_tsx.display().to_string();
    }

    // 2. Binary-adjacent node_modules (installed alongside rotiv binary)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            let adj_tsx = bin_dir.join("node_modules").join(&tsx_subpath);
            if adj_tsx.exists() {
                return adj_tsx.display().to_string();
            }
            // Also check one level up (e.g. .cargo/bin/../node_modules)
            if let Some(parent) = bin_dir.parent() {
                let up_tsx = parent.join("node_modules").join(&tsx_subpath);
                if up_tsx.exists() {
                    return up_tsx.display().to_string();
                }
            }
        }
    }

    // 3. Bare fallback — works if tsx is globally installed or in PATH
    "tsx".to_string()
}

/// Resolve the path to the route-worker entry point.
///
/// Resolution order:
/// 1. `ROTIV_WORKER_PATH` env var
/// 2. `embedded_path` — explicit path provided by the CLI (e.g. temp dir with embedded source)
/// 3. `<binary_dir>/../../packages/@rotiv/route-worker/src/index.ts` (dev monorepo layout)
/// 4. `<binary_dir>/route-worker/index.ts` (production binary-relative layout)
pub fn resolve_worker_path(embedded_path: Option<PathBuf>) -> Result<PathBuf, RotivError> {
    // 1. Environment variable override
    if let Ok(path) = std::env::var("ROTIV_WORKER_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
        return Err(
            RotivError::new(
                "E_WORKER_NOT_FOUND",
                format!("ROTIV_WORKER_PATH is set but file not found: {path}"),
            )
            .with_suggestion("Check that the path points to @rotiv/route-worker/src/index.ts"),
        );
    }

    // 2. Embedded path from CLI (written from include_str! source at startup)
    if let Some(p) = embedded_path {
        if p.exists() {
            return Ok(p);
        }
    }

    // 3. Dev layout: binary is at target/debug/rotiv, worker is at packages/@rotiv/route-worker/src/index.ts
    if let Ok(binary_dir) = std::env::current_exe().map(|p| p.parent().unwrap_or(&p).to_path_buf())
    {
        let dev_path = binary_dir
            .join("..") // target/debug -> target
            .join("..") // target -> rotiv/
            .join("packages")
            .join("@rotiv")
            .join("route-worker")
            .join("src")
            .join("index.ts");
        let canonical = dev_path.canonicalize().unwrap_or(dev_path);
        if canonical.exists() {
            return Ok(canonical);
        }

        // 4. Production layout: worker bundled alongside binary
        let prod_path = binary_dir.join("route-worker").join("index.ts");
        if prod_path.exists() {
            return Ok(prod_path);
        }
    }

    Err(
        RotivError::new(
            "E_WORKER_NOT_FOUND",
            "cannot locate @rotiv/route-worker entry point",
        )
        .with_suggestion(
            "Set ROTIV_WORKER_PATH=/path/to/packages/@rotiv/route-worker/src/index.ts",
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_worker_path_with_env_var() {
        // Point to a file that doesn't exist — should return E_WORKER_NOT_FOUND
        std::env::set_var("ROTIV_WORKER_PATH", "/nonexistent/index.ts");
        let result = resolve_worker_path(None);
        std::env::remove_var("ROTIV_WORKER_PATH");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "E_WORKER_NOT_FOUND");
    }

    #[test]
    fn resolve_worker_path_uses_embedded_when_provided() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("index.ts");
        std::fs::File::create(&p).unwrap().write_all(b"// stub").unwrap();
        let result = resolve_worker_path(Some(p.clone()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), p);
    }
}
