use std::io::Write;
use std::path::PathBuf;

use rotiv_compiler::{compile_project, CompileOptions};
use rotiv_core::{find_project_root, worker::resolve_tsx_loader, worker::path_to_file_url_or_bare};

use crate::error::CliError;
use crate::output::OutputMode;

// Embed the build-script TypeScript source files at compile time.
const BUILD_INDEX: &str = include_str!("../../../../packages/@rotiv/build-script/src/index.ts");
const BUILD_COMPILER: &str = include_str!("../../../../packages/@rotiv/build-script/src/compiler.ts");
const BUILD_MANIFEST: &str = include_str!("../../../../packages/@rotiv/build-script/src/manifest.ts");

/// Write the embedded build-script source to a temp directory and return the
/// path to `index.ts`. The `TempDir` must be kept alive for the build run.
fn write_embedded_build_script() -> Result<(tempfile::TempDir, PathBuf), CliError> {
    let dir = tempfile::tempdir()
        .map_err(|e| CliError::Other(format!("failed to create temp dir for build-script: {e}")))?;
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src)
        .map_err(|e| CliError::Other(format!("failed to create build-script src dir: {e}")))?;

    let files = [
        ("index.ts", BUILD_INDEX),
        ("compiler.ts", BUILD_COMPILER),
        ("manifest.ts", BUILD_MANIFEST),
    ];
    for (name, content) in &files {
        let mut f = std::fs::File::create(src.join(name))
            .map_err(|e| CliError::Other(format!("failed to write build-script/{name}: {e}")))?;
        f.write_all(content.as_bytes())
            .map_err(|e| CliError::Other(format!("failed to write build-script/{name}: {e}")))?;
    }

    // Required so that Node.js/tsx treats the temp dir as ESM.
    std::fs::write(dir.path().join("package.json"), r#"{"type":"module"}"#)
        .map_err(|e| CliError::Other(format!("failed to write build-script package.json: {e}")))?;

    let entry = src.join("index.ts");
    Ok((dir, entry))
}

pub fn run(out: Option<PathBuf>, minify: bool, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;
    let out_dir = out.unwrap_or_else(|| project_dir.join("dist"));

    match mode {
        OutputMode::Human => {
            println!("Building {}...", project_dir.display());
        }
        OutputMode::Json => {}
    }

    // Write embedded build-script to temp dir; keep alive for the duration of the build.
    let (_script_dir, script_entry) = write_embedded_build_script()?;

    // Resolve tsx loader and convert to file:// URL for node --import.
    let tsx_raw = resolve_tsx_loader(&project_dir);
    let tsx_loader = path_to_file_url_or_bare(&tsx_raw);

    let options = CompileOptions {
        project_dir: project_dir.clone(),
        out_dir: out_dir.clone(),
        minify,
        source_maps: !minify,
        script_path: Some(script_entry),
        tsx_loader,
    };

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
