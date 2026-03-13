use std::path::PathBuf;

use rotiv_compiler::{compile_project, CompileOptions};
use rotiv_core::find_project_root;

use crate::error::CliError;
use crate::output::OutputMode;

pub fn run(out: Option<PathBuf>, minify: bool, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;
    let out_dir = out.unwrap_or_else(|| project_dir.join("dist"));

    let options = CompileOptions {
        project_dir: project_dir.clone(),
        out_dir: out_dir.clone(),
        minify,
        source_maps: !minify,
    };

    match mode {
        OutputMode::Human => {
            println!("Building {}...", project_dir.display());
        }
        OutputMode::Json => {}
    }

    let result = compile_project(options).map_err(|e| {
        CliError::Other(format!("build failed: {e}"))
    })?;

    match mode {
        OutputMode::Human => {
            println!(
                "Built {} file(s) to {} in {}ms",
                result.files_written.len(),
                out_dir.display(),
                result.duration_ms,
            );
            for warn in &result.warnings {
                eprintln!("warning: {warn}");
            }
        }
        OutputMode::Json => {
            let files: Vec<_> = result
                .files_written
                .iter()
                .map(|p| p.to_string_lossy())
                .collect();
            println!(
                "{}",
                serde_json::json!({
                    "files": files,
                    "warnings": result.warnings,
                    "duration_ms": result.duration_ms,
                })
            );
        }
    }

    Ok(())
}
