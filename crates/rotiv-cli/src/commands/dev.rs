use rotiv_core::{find_project_root, DevServer, DevServerConfig};

use crate::error::CliError;
use crate::output::OutputMode;

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

    let config = DevServerConfig {
        port,
        host: host.to_string(),
        project_dir,
        worker_port: port + 1,
        json_output: matches!(mode, OutputMode::Json),
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

