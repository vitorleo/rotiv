pub mod error;

pub use error::CompilerError;

use std::path::PathBuf;
use std::time::Instant;

use serde::Deserialize;

/// Options for compiling a Rotiv project.
pub struct CompileOptions {
    /// Root directory of the Rotiv project (contains `.rotiv/spec.json`).
    pub project_dir: PathBuf,
    /// Output directory (defaults to `<project_dir>/dist`).
    pub out_dir: PathBuf,
    /// Enable minification (production builds).
    pub minify: bool,
    /// Emit inline source maps (dev builds).
    pub source_maps: bool,
}

/// Result of a successful compilation.
pub struct CompileResult {
    /// Paths of all files written to `out_dir`.
    pub files_written: Vec<PathBuf>,
    /// Non-fatal warnings produced during compilation.
    pub warnings: Vec<String>,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

/// JSON structure emitted by the build script on stdout.
#[derive(Deserialize)]
struct BuildScriptOutput {
    files: Vec<String>,
    warnings: Vec<String>,
    duration_ms: u64,
}

/// Compile a Rotiv project by invoking the Node.js build script.
///
/// The build script is located via (in priority order):
///   1. `ROTIV_BUILD_SCRIPT_PATH` environment variable
///   2. `<binary_dir>/../../packages/@rotiv/build-script/src/index.ts` (dev monorepo layout)
///   3. `<binary_dir>/build-script/index.ts` (production layout)
pub fn compile_project(options: CompileOptions) -> Result<CompileResult, CompilerError> {
    let script_path = resolve_build_script_path()?;

    let started = Instant::now();

    let mut cmd = std::process::Command::new("node");
    cmd.arg("--import")
        .arg("tsx")
        .arg(&script_path)
        .arg("--project")
        .arg(&options.project_dir)
        .arg("--out")
        .arg(&options.out_dir);

    if options.minify {
        cmd.arg("--minify");
    }

    let output = cmd
        .output()
        .map_err(|e| CompilerError::SpawnFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(CompilerError::BuildFailed(stderr));
    }

    let parsed: BuildScriptOutput = serde_json::from_slice(&output.stdout).map_err(|e| {
        let raw = String::from_utf8_lossy(&output.stdout);
        CompilerError::ParseFailed(format!("{e}: {raw}"))
    })?;

    let elapsed = started.elapsed().as_millis() as u64;

    Ok(CompileResult {
        files_written: parsed.files.into_iter().map(PathBuf::from).collect(),
        warnings: parsed.warnings,
        duration_ms: elapsed.max(parsed.duration_ms),
    })
}

/// Resolve the path to the Node.js build script.
pub fn resolve_build_script_path() -> Result<PathBuf, CompilerError> {
    // 1. Environment variable override
    if let Ok(path) = std::env::var("ROTIV_BUILD_SCRIPT_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
        return Err(CompilerError::ScriptNotFound(format!(
            "ROTIV_BUILD_SCRIPT_PATH={path} does not exist"
        )));
    }

    if let Ok(exe) = std::env::current_exe() {
        // 2. Dev monorepo layout: binary at target/debug/rotiv.exe
        let dev_path = exe
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|repo_root| {
                repo_root
                    .join("packages")
                    .join("@rotiv")
                    .join("build-script")
                    .join("src")
                    .join("index.ts")
            });

        if let Some(p) = dev_path {
            if p.exists() {
                return Ok(p);
            }
        }

        // 3. Production layout: script shipped alongside binary
        let prod_path = exe
            .parent()
            .map(|dir| dir.join("build-script").join("index.ts"));

        if let Some(p) = prod_path {
            if p.exists() {
                return Ok(p);
            }
        }
    }

    Err(CompilerError::ScriptNotFound(
        "Set ROTIV_BUILD_SCRIPT_PATH or run from the Rotiv monorepo".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_error_stub() {
        let err = CompilerError::NotImplemented("tsx transform".to_string());
        assert!(err.to_string().contains("Not implemented"));
    }

    #[test]
    fn compiler_error_spawn_failed() {
        let err = CompilerError::SpawnFailed("node not found".to_string());
        assert!(err.to_string().contains("spawn"));
    }

    #[test]
    fn compiler_error_build_failed() {
        let err = CompilerError::BuildFailed("syntax error in index.tsx".to_string());
        assert!(err.to_string().contains("Build failed"));
    }

    #[test]
    fn compile_options_construction() {
        let opts = CompileOptions {
            project_dir: PathBuf::from("/tmp/my-project"),
            out_dir: PathBuf::from("/tmp/my-project/dist"),
            minify: false,
            source_maps: true,
        };
        assert!(!opts.minify);
        assert!(opts.source_maps);
    }
}
