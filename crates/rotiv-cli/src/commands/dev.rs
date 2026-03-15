use std::io::Write;

use rotiv_core::{find_project_root, DevServer, DevServerConfig};

use crate::error::CliError;
use crate::output::OutputMode;

// Embed the route-worker TypeScript source files at compile time.
// These are written to a temp directory at runtime so the standalone binary
// can serve routes without requiring the Rotiv monorepo to be present.
const WORKER_INDEX: &str = include_str!("../../../../packages/@rotiv/route-worker/src/index.ts");
const WORKER_INVOKE: &str = include_str!("../../../../packages/@rotiv/route-worker/src/invoke.ts");
const WORKER_DB: &str = include_str!("../../../../packages/@rotiv/route-worker/src/db.ts");
const WORKER_ERRORS: &str = include_str!("../../../../packages/@rotiv/route-worker/src/errors.ts");
const WORKER_RENDER: &str = include_str!("../../../../packages/@rotiv/route-worker/src/render.ts");
const WORKER_TRANSFORM: &str = include_str!("../../../../packages/@rotiv/route-worker/src/transform.ts");

/// Write the embedded route-worker source to a temp directory and return the
/// path to `index.ts`. The `TempDir` is returned alongside so the caller can
/// keep it alive for the lifetime of the dev server.
fn write_embedded_worker() -> Result<(tempfile::TempDir, std::path::PathBuf), CliError> {
    let dir = tempfile::tempdir()
        .map_err(|e| CliError::Other(format!("failed to create temp dir for worker: {e}")))?;
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src)
        .map_err(|e| CliError::Other(format!("failed to create worker src dir: {e}")))?;

    let files = [
        ("index.ts", WORKER_INDEX),
        ("invoke.ts", WORKER_INVOKE),
        ("db.ts", WORKER_DB),
        ("errors.ts", WORKER_ERRORS),
        ("render.ts", WORKER_RENDER),
        ("transform.ts", WORKER_TRANSFORM),
    ];
    for (name, content) in &files {
        let mut f = std::fs::File::create(src.join(name))
            .map_err(|e| CliError::Other(format!("failed to write worker/{name}: {e}")))?;
        f.write_all(content.as_bytes())
            .map_err(|e| CliError::Other(format!("failed to write worker/{name}: {e}")))?;
    }

    let entry = src.join("index.ts");
    Ok((dir, entry))
}

pub fn run(port: u16, host: &str, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(|e| {
        // Print the error immediately before returning, so the caller can re-use
        // the error for both human and JSON output modes via main.rs.
        CliError::Rotiv(e)
    })?;

    match mode {
        OutputMode::Human => {
            println!();
        }
        OutputMode::Json => {}
    }

    // Write embedded worker source to a temp directory. Keep `_worker_dir`
    // alive for the duration of the server so the temp files aren't deleted.
    let (_worker_dir, worker_entry) = write_embedded_worker()?;

    let config = DevServerConfig {
        port,
        host: host.to_string(),
        project_dir,
        worker_port: port + 1,
        json_output: matches!(mode, OutputMode::Json),
        worker_path: Some(worker_entry),
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Other(format!("failed to start async runtime: {e}")))?;

    rt.block_on(async {
        DevServer::new(config)
            .start()
            .await
            .map_err(CliError::Rotiv)
    })
}

