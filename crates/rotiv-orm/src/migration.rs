use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Deserialize;

use crate::OrmError;

/// Options controlling the migration run.
pub struct MigrationOptions {
    /// Root directory of the Rotiv project.
    pub project_dir: PathBuf,
    /// Only generate migration files; don't apply them.
    pub generate_only: bool,
    /// Only check for pending migrations; don't apply.
    pub check_only: bool,
    /// Emit JSON to stdout instead of human-readable output.
    pub json_output: bool,
}

/// Result of a successful migration run.
pub struct MigrationResult {
    /// Number of migration files applied.
    pub migrations_applied: u32,
    /// Paths of all migration files processed.
    pub migration_files: Vec<PathBuf>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

/// JSON structure emitted by the migrate script on stdout.
#[derive(Deserialize)]
struct MigrateScriptOutput {
    ok: bool,
    migrations_applied: Option<u32>,
    migration_files: Option<Vec<String>>,
    warnings: Option<Vec<String>>,
    duration_ms: u64,
    /// For --check mode
    pending: Option<u32>,
}

/// Run migrations by spawning the Node.js migrate script.
///
/// The script is located via (priority order):
///   1. `ROTIV_MIGRATE_SCRIPT_PATH` environment variable
///   2. `<binary>/../../packages/@rotiv/migrate-script/src/index.ts` (dev monorepo)
///   3. `<binary>/migrate-script/index.ts` (production)
pub fn run_migrations(options: MigrationOptions) -> Result<MigrationResult, OrmError> {
    let script_path = resolve_migrate_script_path()?;
    let started = Instant::now();

    let mode_flag = if options.generate_only {
        "--generate"
    } else if options.check_only {
        "--check"
    } else {
        "--migrate"
    };

    let output = std::process::Command::new("node")
        .arg("--import")
        .arg("tsx")
        .arg(&script_path)
        .arg("--project")
        .arg(&options.project_dir)
        .arg(mode_flag)
        .output()
        .map_err(|e| OrmError::SpawnFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(OrmError::MigrationFailed(stderr));
    }

    let parsed: MigrateScriptOutput =
        serde_json::from_slice(&output.stdout).map_err(|e| {
            let raw = String::from_utf8_lossy(&output.stdout);
            OrmError::ParseFailed(format!("{e}: {raw}"))
        })?;

    if !parsed.ok {
        return Err(OrmError::MigrationFailed(
            "migrate script reported ok=false".to_string(),
        ));
    }

    let elapsed = started.elapsed().as_millis() as u64;

    // For --check mode, the script emits `pending` instead of `migrations_applied`
    let migrations_applied = parsed
        .migrations_applied
        .or(parsed.pending)
        .unwrap_or(0);

    Ok(MigrationResult {
        migrations_applied,
        migration_files: parsed
            .migration_files
            .unwrap_or_default()
            .into_iter()
            .map(PathBuf::from)
            .collect(),
        warnings: parsed.warnings.unwrap_or_default(),
        duration_ms: elapsed.max(parsed.duration_ms),
    })
}

/// Auto-migrate helper: check pending (fast, no subprocess) then apply if needed.
pub fn auto_migrate(project_dir: &Path) -> Result<MigrationResult, OrmError> {
    // Fast check — reads journal JSON only
    let check_result = run_migrations(MigrationOptions {
        project_dir: project_dir.to_path_buf(),
        generate_only: false,
        check_only: true,
        json_output: false,
    })?;

    let pending = check_result.migrations_applied; // pending field maps here from check mode
    if pending == 0 {
        return Ok(check_result);
    }

    // Pending migrations found — apply them
    run_migrations(MigrationOptions {
        project_dir: project_dir.to_path_buf(),
        generate_only: false,
        check_only: false,
        json_output: false,
    })
}

/// Resolve the path to the Node.js migrate script.
pub fn resolve_migrate_script_path() -> Result<PathBuf, OrmError> {
    // 1. Environment variable override
    if let Ok(path) = std::env::var("ROTIV_MIGRATE_SCRIPT_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
        return Err(OrmError::ScriptNotFound(format!(
            "ROTIV_MIGRATE_SCRIPT_PATH={path} does not exist"
        )));
    }

    if let Ok(exe) = std::env::current_exe() {
        // 2. Dev monorepo layout
        let dev_path = exe
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|repo_root| {
                repo_root
                    .join("packages")
                    .join("@rotiv")
                    .join("migrate-script")
                    .join("src")
                    .join("index.ts")
            });

        if let Some(p) = dev_path {
            if p.exists() {
                return Ok(p);
            }
        }

        // 3. Production layout
        let prod_path = exe
            .parent()
            .map(|dir| dir.join("migrate-script").join("index.ts"));

        if let Some(p) = prod_path {
            if p.exists() {
                return Ok(p);
            }
        }
    }

    Err(OrmError::ScriptNotFound(
        "Set ROTIV_MIGRATE_SCRIPT_PATH or run from the Rotiv monorepo".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_options_construction() {
        let opts = MigrationOptions {
            project_dir: PathBuf::from("/tmp/project"),
            generate_only: false,
            check_only: true,
            json_output: false,
        };
        assert!(opts.check_only);
        assert!(!opts.generate_only);
    }

    #[test]
    fn resolve_script_env_override_missing() {
        std::env::set_var("ROTIV_MIGRATE_SCRIPT_PATH", "/nonexistent/path/index.ts");
        let result = resolve_migrate_script_path();
        std::env::remove_var("ROTIV_MIGRATE_SCRIPT_PATH");
        assert!(matches!(result, Err(OrmError::ScriptNotFound(_))));
    }
}
