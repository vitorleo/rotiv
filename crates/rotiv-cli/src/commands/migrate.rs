use std::io::Write;

use rotiv_core::find_project_root;
use rotiv_orm::{MigrationOptions, run_migrations};

use crate::error::CliError;
use crate::output::OutputMode;

// Embed the migrate-script TypeScript source files at compile time.
const MIGRATE_INDEX: &str = include_str!("../../../../packages/@rotiv/migrate-script/src/index.ts");
const MIGRATE_RUNNER: &str = include_str!("../../../../packages/@rotiv/migrate-script/src/runner.ts");
const MIGRATE_CONFIG: &str = include_str!("../../../../packages/@rotiv/migrate-script/src/drizzle-config.ts");

/// Write the embedded migrate-script source to a temp directory and return the
/// path to `index.ts`. The `TempDir` must be kept alive for the migration run.
fn write_embedded_migrate_script() -> Result<(tempfile::TempDir, std::path::PathBuf), CliError> {
    let dir = tempfile::tempdir()
        .map_err(|e| CliError::Other(format!("failed to create temp dir for migrate-script: {e}")))?;
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src)
        .map_err(|e| CliError::Other(format!("failed to create migrate-script src dir: {e}")))?;

    let files = [
        ("index.ts", MIGRATE_INDEX),
        ("runner.ts", MIGRATE_RUNNER),
        ("drizzle-config.ts", MIGRATE_CONFIG),
    ];
    for (name, content) in &files {
        let mut f = std::fs::File::create(src.join(name))
            .map_err(|e| CliError::Other(format!("failed to write migrate-script/{name}: {e}")))?;
        f.write_all(content.as_bytes())
            .map_err(|e| CliError::Other(format!("failed to write migrate-script/{name}: {e}")))?;
    }

    // Required so that Node.js/tsx treats the temp dir as ESM.
    std::fs::write(dir.path().join("package.json"), r#"{"type":"module"}"#)
        .map_err(|e| CliError::Other(format!("failed to write migrate-script package.json: {e}")))?;

    let entry = src.join("index.ts");
    Ok((dir, entry))
}

pub fn run(generate_only: bool, check: bool, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;

    let models_dir = project_dir.join("app").join("models");
    if !models_dir.exists() {
        match mode {
            OutputMode::Human => println!("  migrate   no models directory — nothing to do"),
            OutputMode::Json => {
                println!(r#"{{"ok":true,"message":"no models directory","migrations_applied":0}}"#)
            }
        }
        return Ok(());
    }

    // Write embedded migrate-script to temp dir; keep alive for the duration of the run.
    let (_script_dir, script_entry) = write_embedded_migrate_script()?;

    let options = MigrationOptions {
        project_dir,
        generate_only,
        check_only: check,
        json_output: matches!(mode, OutputMode::Json),
        script_path: Some(script_entry),
    };

    let result = run_migrations(options)
        .map_err(|e| CliError::Other(format!("migration error: {e}")))?;

    match mode {
        OutputMode::Human => {
            if check {
                println!(
                    "  migrate   {} pending migration(s)",
                    result.migrations_applied
                );
            } else if generate_only {
                println!("  migrate   migration files generated");
            } else {
                println!(
                    "  migrate   {} migration(s) applied in {}ms",
                    result.migrations_applied, result.duration_ms
                );
            }
            for w in &result.warnings {
                eprintln!("  [migrate] warning: {w}");
            }
        }
        OutputMode::Json => {
            let files: Vec<String> = result
                .migration_files
                .iter()
                .map(|p| p.display().to_string())
                .collect();
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "migrations_applied": result.migrations_applied,
                    "migration_files": files,
                    "warnings": result.warnings,
                    "duration_ms": result.duration_ms,
                })
            );
        }
    }

    Ok(())
}
