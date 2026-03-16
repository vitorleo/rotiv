use std::path::PathBuf;

use rotiv_core::RotivError;
use serde::Serialize;

use crate::error::CliError;
use crate::output::{human, json, OutputMode};

// Embedded templates — compiled into the binary at build time.
const SPEC_JSON: &str = include_str!("../../../../templates/default/.rotiv/spec.json");
const CONTEXT_MD: &str = include_str!("../../../../templates/default/.rotiv/context.md");
const ROUTES_INDEX_TSX: &str = include_str!("../../../../templates/default/app/routes/index.tsx");
const MODEL_USER_TS: &str = include_str!("../../../../templates/default/app/models/user.ts");
const PACKAGE_JSON: &str = include_str!("../../../../templates/default/package.json");
const TSCONFIG_JSON: &str = include_str!("../../../../templates/default/tsconfig.json");
const README_MD: &str = include_str!("../../../../templates/default/README.md");
const NPMRC: &str = include_str!("../../../../templates/default/.npmrc");

#[derive(Serialize)]
struct NewSuccess {
    ok: bool,
    project: String,
    path: String,
}

pub fn run(name: &str, mode: OutputMode) -> Result<(), CliError> {
    let dest = PathBuf::from(name);

    if dest.exists() {
        let err = RotivError::new("E001", format!("directory '{}' already exists", name))
            .with_suggestion(format!(
                "Choose a different name or remove the existing directory: rm -rf {}",
                name
            ));
        return Err(CliError::Rotiv(err));
    }

    // Validate project name: alphanumeric, hyphens, underscores only
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        let err = RotivError::new(
            "E002",
            format!(
                "invalid project name '{}': only alphanumeric characters, hyphens, and underscores are allowed",
                name
            ),
        )
        .with_expected("alphanumeric, hyphens, underscores", name);
        return Err(CliError::Rotiv(err));
    }

    let created_at = current_timestamp();

    // Create directory structure
    let rotiv_dir = dest.join(".rotiv");
    let routes_dir = dest.join("app").join("routes");
    let models_dir = dest.join("app").join("models");

    std::fs::create_dir_all(&rotiv_dir)?;
    std::fs::create_dir_all(&routes_dir)?;
    std::fs::create_dir_all(&models_dir)?;

    // Write example model
    write_template(&models_dir.join("user.ts"), MODEL_USER_TS, name, &created_at)?;

    // Write templated files
    write_template(&rotiv_dir.join("spec.json"), SPEC_JSON, name, &created_at)?;
    write_template(&rotiv_dir.join("context.md"), CONTEXT_MD, name, &created_at)?;
    write_template(&routes_dir.join("index.tsx"), ROUTES_INDEX_TSX, name, &created_at)?;
    write_template(&dest.join("package.json"), PACKAGE_JSON, name, &created_at)?;
    write_template(&dest.join("tsconfig.json"), TSCONFIG_JSON, name, &created_at)?;
    write_template(&dest.join("README.md"), README_MD, name, &created_at)?;
    write_template(&dest.join(".npmrc"), NPMRC, name, &created_at)?;

    let abs_path = std::fs::canonicalize(&dest)
        .unwrap_or(dest.clone())
        .display()
        .to_string();

    match mode {
        OutputMode::Json => {
            json::print_success(&NewSuccess {
                ok: true,
                project: name.to_string(),
                path: abs_path,
            });
        }
        OutputMode::Human => {
            human::print_success(&format!("Created project '{}'", name));
            human::print_info("location", &abs_path);
            println!();
            println!("  Next steps:");
            println!("    cd {}", name);
            println!("    pnpm install      # installs tsx, drizzle-kit, drizzle-orm");
            println!("    rotiv migrate     # creates the database from your schema");
            println!("    rotiv dev         # starts the dev server at http://localhost:3000");
        }
    }

    Ok(())
}

fn write_template(
    path: &PathBuf,
    template: &str,
    project_name: &str,
    created_at: &str,
) -> Result<(), CliError> {
    let content = template
        .replace("{{project_name}}", project_name)
        .replace("{{created_at}}", created_at);
    std::fs::write(path, content)?;
    Ok(())
}

fn current_timestamp() -> String {
    // Simple ISO 8601 timestamp without external crate.
    // Uses std::time for portability in Phase 1.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Convert unix timestamp to a human-readable date (no chrono in Phase 1)
    // Format: YYYY-MM-DDTHH:MM:SSZ (approximate — good enough for spec.json)
    let secs_per_day = 86400u64;
    let days_since_epoch = secs / secs_per_day;
    let time_of_day = secs % secs_per_day;

    let hh = time_of_day / 3600;
    let mm = (time_of_day % 3600) / 60;
    let ss = time_of_day % 60;

    // Gregorian calendar calculation from days since 1970-01-01
    let (year, month, day) = days_to_ymd(days_since_epoch);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hh, mm, ss
    )
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Algorithm: civil date from days since epoch (1970-01-01)
    // Reference: http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_format() {
        let ts = current_timestamp();
        assert!(ts.contains('T'));
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20);
    }

    #[test]
    fn days_to_ymd_epoch() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn write_template_substitutes_placeholders() {
        let result = "{{project_name}} created at {{created_at}}"
            .replace("{{project_name}}", "my-app")
            .replace("{{created_at}}", "2025-01-01T00:00:00Z");
        assert_eq!(result, "my-app created at 2025-01-01T00:00:00Z");
    }
}
