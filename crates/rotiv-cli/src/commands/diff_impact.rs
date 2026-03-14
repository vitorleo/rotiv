use std::fs;
use std::path::Path;

use rotiv_core::find_project_root;
use serde::Serialize;

use crate::commands::spec_sync::file_to_route_path;
use crate::error::CliError;
use crate::output::{OutputMode, human};

#[derive(Serialize)]
struct AffectedRoute {
    path: String,
    file: String,
    import_line: String,
}

pub fn run(target_file: &str, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;

    // Derive the stem to search for: "app/models/user.ts" → "user"
    let target_path = Path::new(target_file);
    let stem = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(target_file);

    let routes_dir = project_dir.join("app").join("routes");
    let affected = find_affected_routes(&routes_dir, &project_dir, stem);

    let target_normalized = target_file.replace('\\', "/");

    match mode {
        OutputMode::Human => {
            if affected.is_empty() {
                human::print_info("diff-impact", &format!("no routes import '{}'", stem));
            } else {
                human::print_success(&format!(
                    "{} route(s) affected by changes to '{}'",
                    affected.len(),
                    target_normalized
                ));
                for route in &affected {
                    println!("  {} ({})", route.path, route.file);
                    println!("    import: {}", route.import_line);
                }
            }
        }
        OutputMode::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "target": target_normalized,
                    "affected_routes": affected,
                    "total": affected.len(),
                })
            );
        }
    }

    Ok(())
}

fn find_affected_routes(
    routes_dir: &Path,
    project_dir: &Path,
    stem: &str,
) -> Vec<AffectedRoute> {
    let mut results = Vec::new();

    if !routes_dir.exists() {
        return results;
    }

    walk_route_files(routes_dir, routes_dir, project_dir, stem, &mut results);
    results.sort_by(|a, b| a.path.cmp(&b.path));
    results
}

fn walk_route_files(
    dir: &Path,
    routes_root: &Path,
    project_dir: &Path,
    stem: &str,
    results: &mut Vec<AffectedRoute>,
) {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_route_files(&path, routes_root, project_dir, stem, results);
        } else if path.extension().and_then(|e| e.to_str()) == Some("tsx") {
            if let Some(hit) = check_file_for_import(&path, routes_root, project_dir, stem) {
                results.push(hit);
            }
        }
    }
}

fn check_file_for_import(
    file: &Path,
    routes_root: &Path,
    project_dir: &Path,
    stem: &str,
) -> Option<AffectedRoute> {
    let content = fs::read_to_string(file).ok()?;
    let rel_to_routes = file.strip_prefix(routes_root).ok()?;
    let rel_to_project = file
        .strip_prefix(project_dir)
        .unwrap_or(file)
        .display()
        .to_string()
        .replace('\\', "/");

    let route_path = file_to_route_path(rel_to_routes);

    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("import") {
            continue;
        }
        // Check if this import line references the stem
        if trimmed.contains(stem) {
            return Some(AffectedRoute {
                path: route_path,
                file: rel_to_project,
                import_line: trimmed.to_string(),
            });
        }
    }

    None
}
