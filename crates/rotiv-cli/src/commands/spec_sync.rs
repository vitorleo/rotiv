use std::fs;
use std::path::Path;

use rotiv_core::find_project_root;

use crate::error::CliError;
use crate::output::{OutputMode, human};

pub fn run(mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;
    run_for_project(&project_dir, mode)
}

pub fn run_for_project(project_dir: &Path, mode: OutputMode) -> Result<(), CliError> {
    let spec_path = project_dir.join(".rotiv").join("spec.json");

    // Read existing spec or start minimal
    let mut spec: serde_json::Value = if spec_path.exists() {
        let raw = fs::read_to_string(&spec_path)?;
        serde_json::from_str(&raw)
            .unwrap_or_else(|_| serde_json::json!({"version": "1"}))
    } else {
        serde_json::json!({"version": "1"})
    };

    // Discover routes
    let routes_dir = project_dir.join("app").join("routes");
    let routes = discover_routes(&routes_dir, project_dir);

    // Discover models via rotiv_orm
    let models = discover_models_entries(project_dir);

    // Update spec
    spec["routes"] = serde_json::Value::Array(routes.clone());
    spec["models"] = serde_json::Value::Array(models.clone());

    // Write back
    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json_str = serde_json::to_string_pretty(&spec).unwrap_or_default();
    fs::write(&spec_path, json_str)?;

    match mode {
        OutputMode::Human => {
            human::print_success(&format!(
                "synced {} route(s), {} model(s) → .rotiv/spec.json",
                routes.len(),
                models.len()
            ));
        }
        OutputMode::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "routes": routes.len(),
                    "models": models.len(),
                    "spec": ".rotiv/spec.json"
                })
            );
        }
    }

    Ok(())
}

fn discover_routes(routes_dir: &Path, project_dir: &Path) -> Vec<serde_json::Value> {
    let mut entries = Vec::new();

    if !routes_dir.exists() {
        return entries;
    }

    walk_routes(routes_dir, routes_dir, project_dir, &mut entries);
    entries.sort_by(|a, b| {
        a["path"]
            .as_str()
            .unwrap_or("")
            .cmp(b["path"].as_str().unwrap_or(""))
    });
    entries
}

fn walk_routes(
    dir: &Path,
    routes_root: &Path,
    project_dir: &Path,
    entries: &mut Vec<serde_json::Value>,
) {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_routes(&path, routes_root, project_dir, entries);
        } else if path.extension().and_then(|e| e.to_str()) == Some("tsx") {
            if let Some(entry) = build_route_entry(&path, routes_root, project_dir) {
                entries.push(entry);
            }
        }
    }
}

fn build_route_entry(
    file: &Path,
    routes_root: &Path,
    project_dir: &Path,
) -> Option<serde_json::Value> {
    let rel_to_routes = file.strip_prefix(routes_root).ok()?;
    let rel_to_project = file.strip_prefix(project_dir).ok()?;

    // Derive route path from filename
    let route_path = file_to_route_path(rel_to_routes);

    // Scan non-comment lines for presence flags
    let content = fs::read_to_string(file).unwrap_or_default();
    let code_lines: String = content
        .lines()
        .filter(|l| !l.trim().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let has_loader = code_lines.contains("loader(") || code_lines.contains("loader:");
    let has_action = code_lines.contains("action(") || code_lines.contains("action:");
    let has_component = code_lines.contains("component(") || code_lines.contains("component:");

    Some(serde_json::json!({
        "path": route_path,
        "file": rel_to_project.display().to_string().replace('\\', "/"),
        "has_loader": has_loader,
        "has_action": has_action,
        "has_component": has_component,
    }))
}

/// Convert a relative route file path to a URL route path.
/// app/routes/users/[id].tsx → /users/:id
/// app/routes/index.tsx → /
pub fn file_to_route_path(rel: &Path) -> String {
    let stem = rel.with_extension("");
    let parts: Vec<&str> = stem
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    let mut segments: Vec<String> = Vec::new();
    for part in &parts {
        if *part == "index" || *part == "layout" {
            continue;
        }
        if part.starts_with('[') && part.ends_with(']') {
            segments.push(format!(":{}", &part[1..part.len() - 1]));
        } else {
            segments.push(part.to_string());
        }
    }

    if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    }
}

fn discover_models_entries(project_dir: &Path) -> Vec<serde_json::Value> {
    use rotiv_orm::discover_models;

    let model_entries = match discover_models(project_dir) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    model_entries
        .into_iter()
        .map(|m| {
            let file_rel = m
                .file
                .strip_prefix(project_dir)
                .unwrap_or(&m.file)
                .display()
                .to_string()
                .replace('\\', "/");
            // Derive table name: snake_case plural of model name
            let table = to_snake_plural(&m.name);
            serde_json::json!({
                "name": m.name,
                "file": file_rel,
                "table": table,
            })
        })
        .collect()
}

fn to_snake_plural(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    if out.ends_with('s') || out.ends_with('x') || out.ends_with('z') {
        format!("{}es", out)
    } else {
        format!("{}s", out)
    }
}

pub fn read_project_name(project_dir: &Path) -> String {
    let spec_path = project_dir.join(".rotiv").join("spec.json");
    if let Ok(raw) = fs::read_to_string(&spec_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            // Try both spec formats
            if let Some(name) = v["project"]["name"].as_str() {
                return name.to_string();
            }
            if let Some(name) = v["project_name"].as_str() {
                return name.to_string();
            }
        }
    }
    project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string()
}

pub fn discover_routes_for_project(project_dir: &Path) -> Vec<serde_json::Value> {
    discover_routes(&project_dir.join("app").join("routes"), project_dir)
}

pub fn discover_models_entries_pub(project_dir: &Path) -> Vec<serde_json::Value> {
    discover_models_entries(project_dir)
}
