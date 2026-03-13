use rotiv_core::find_project_root;
use rotiv_orm::{MigrationOptions, run_migrations};

use crate::error::CliError;
use crate::output::OutputMode;

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

    let options = MigrationOptions {
        project_dir,
        generate_only,
        check_only: check,
        json_output: matches!(mode, OutputMode::Json),
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
