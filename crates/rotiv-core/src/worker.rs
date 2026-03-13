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
    pub fn new(project_dir: PathBuf, port: u16) -> Result<Self, RotivError> {
        let worker_path = resolve_worker_path()?;
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

        let child = tokio::process::Command::new("node")
            .arg("--import")
            .arg("tsx")
            .arg(&self.worker_path)
            .env("ROTIV_WORKER_PORT", self.port.to_string())
            .env("ROTIV_PROJECT_DIR", self.project_dir.display().to_string())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| {
                RotivError::new("E_WORKER_SPAWN", format!("failed to start route worker: {e}"))
                    .with_suggestion(
                        "Make sure Node.js is installed and available in your PATH",
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

/// Resolve the path to the route-worker entry point.
///
/// Resolution order:
/// 1. `ROTIV_WORKER_PATH` env var
/// 2. `<binary_dir>/../../packages/@rotiv/route-worker/src/index.ts` (dev layout)
/// 3. `<binary_dir>/route-worker/index.ts` (production layout)
pub fn resolve_worker_path() -> Result<PathBuf, RotivError> {
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

    // 2. Dev layout: binary is at target/debug/rotiv, worker is at packages/@rotiv/route-worker/src/index.ts
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

        // 3. Production layout: worker bundled alongside binary
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
        let result = resolve_worker_path();
        std::env::remove_var("ROTIV_WORKER_PATH");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "E_WORKER_NOT_FOUND");
    }
}
