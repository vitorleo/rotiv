use std::path::PathBuf;

use rotiv_core::{DevServer, DevServerConfig, RotivError};

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

/// Walk up from `cwd` until `.rotiv/spec.json` is found.
fn find_project_root() -> Result<PathBuf, RotivError> {
    let mut dir = std::env::current_dir().map_err(|e| {
        RotivError::new("E_IO", format!("cannot read current directory: {e}"))
    })?;

    loop {
        if dir.join(".rotiv").join("spec.json").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(
                RotivError::new("E_NOT_A_PROJECT", "not inside a Rotiv project")
                    .with_suggestion(
                        "Run `rotiv new <name>` to create a project, then `cd <name> && rotiv dev`",
                    ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_project_root_fails_outside_project() {
        // /tmp (or equiv) is not a Rotiv project
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(std::env::temp_dir()).unwrap();
        let result = find_project_root();
        std::env::set_current_dir(original).unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "E_NOT_A_PROJECT");
    }
}
