use std::path::Path;

use rotiv_core::{RotivError, find_project_root};
use serde::Serialize;

use crate::error::CliError;
use crate::output::{OutputMode, human, json};

const ROUTE_TEMPLATE: &str = include_str!("../templates/add/route.tsx");
const MODEL_TEMPLATE: &str = include_str!("../templates/add/model.ts");

#[derive(Serialize)]
struct AddSuccess {
    ok: bool,
    kind: String,
    file: String,
}

/// `rotiv add route <path>` — scaffold an annotated route file.
pub fn run_add_route(path: &str, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;
    let (route_path, file_path) = derive_route_paths(path);
    let dest = project_dir.join("app").join("routes").join(&file_path);

    if dest.exists() {
        let err = RotivError::new(
            "E010",
            format!("file already exists: {}", dest.display()),
        )
        .with_suggestion("Use a different path or delete the existing file first.");
        return Err(CliError::Rotiv(err));
    }

    // Create parent directories if needed (e.g. app/routes/users/)
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = ROUTE_TEMPLATE
        .replace("{{route_path}}", &route_path)
        .replace("{{route_file_path}}", path);

    std::fs::write(&dest, content)?;

    let rel = format!("app/routes/{}", file_path);
    match mode {
        OutputMode::Human => {
            human::print_success(&format!("created {}", rel));
            human::print_info("path", &route_path);
        }
        OutputMode::Json => json::print_success(&AddSuccess {
            ok: true,
            kind: "route".to_string(),
            file: rel,
        }),
    }
    Ok(())
}

/// `rotiv add model <Name>` — scaffold an annotated model file.
pub fn run_add_model(name: &str, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;

    // Validate PascalCase: first char uppercase, rest alphanumeric
    if name.is_empty()
        || !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
        || !name.chars().all(|c| c.is_alphanumeric())
    {
        let err = RotivError::new(
            "E011",
            format!("invalid model name '{}': must be PascalCase (e.g. Post, UserProfile)", name),
        )
        .with_expected("PascalCase name (e.g. Post)", name);
        return Err(CliError::Rotiv(err));
    }

    let table_name = to_snake_plural(name);
    let file_name = format!("{}.ts", to_snake(name));
    let dest = project_dir.join("app").join("models").join(&file_name);

    if dest.exists() {
        let err = RotivError::new(
            "E010",
            format!("file already exists: {}", dest.display()),
        )
        .with_suggestion("Use a different name or delete the existing file first.");
        return Err(CliError::Rotiv(err));
    }

    std::fs::create_dir_all(dest.parent().unwrap_or(Path::new(".")))?;

    let content = MODEL_TEMPLATE
        .replace("{{model_name}}", name)
        .replace("{{table_name}}", &table_name);

    std::fs::write(&dest, content)?;

    let rel = format!("app/models/{}", file_name);
    match mode {
        OutputMode::Human => {
            human::print_success(&format!("created {}", rel));
            human::print_info("table", &table_name);
            println!();
            println!("  Next steps:");
            println!("    rotiv migrate --generate-only");
            println!("    rotiv migrate");
        }
        OutputMode::Json => json::print_success(&AddSuccess {
            ok: true,
            kind: "model".to_string(),
            file: rel,
        }),
    }
    Ok(())
}

/// Derive the TypeScript route path and the file path from a user-supplied slug.
///
/// Input:  "users/[id]"
/// Output: ("/users/:id", "users/[id].tsx")
fn derive_route_paths(path: &str) -> (String, String) {
    let normalized = path.trim_start_matches('/');

    // Build file path: e.g. "users/[id]" → "users/[id].tsx"
    let file_path = if normalized.ends_with(".tsx") {
        normalized.to_string()
    } else {
        format!("{}.tsx", normalized)
    };

    // Build route path: replace [param] with :param, prepend /
    let route_path = format!(
        "/{}",
        normalized
            .trim_end_matches(".tsx")
            .split('/')
            .map(|segment| {
                if segment.starts_with('[') && segment.ends_with(']') {
                    format!(":{}", &segment[1..segment.len() - 1])
                } else if segment == "index" {
                    String::new()
                } else {
                    segment.to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("/")
    );

    // Handle root index: "/users/[]" → "/" not "//"; also plain "index" → "/"
    let route_path = if route_path == "/" || route_path.is_empty() {
        "/".to_string()
    } else {
        route_path.trim_end_matches('/').to_string()
    };

    (route_path, file_path)
}

/// Convert PascalCase to snake_case. "UserProfile" → "user_profile"
fn to_snake(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    out
}

/// Convert PascalCase to snake_case plural. "Post" → "posts", "UserProfile" → "user_profiles"
fn to_snake_plural(name: &str) -> String {
    let snake = to_snake(name);
    // Simple English pluralization: append 's' (sufficient for model table names)
    if snake.ends_with('s') || snake.ends_with('x') || snake.ends_with('z') {
        format!("{}es", snake)
    } else {
        format!("{}s", snake)
    }
}

/// Build the absolute project file path for a route file, used by spec_sync.
#[allow(dead_code)]
pub fn route_file_to_path(file: &Path, routes_dir: &Path) -> String {
    let rel = file.strip_prefix(routes_dir).unwrap_or(file);
    let stem = rel.with_extension("");
    let parts: Vec<&str> = stem
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    let mut segments: Vec<String> = Vec::new();
    for part in &parts {
        if *part == "index" {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_simple_route() {
        let (route, file) = derive_route_paths("users");
        assert_eq!(route, "/users");
        assert_eq!(file, "users.tsx");
    }

    #[test]
    fn derive_dynamic_route() {
        let (route, file) = derive_route_paths("users/[id]");
        assert_eq!(route, "/users/:id");
        assert_eq!(file, "users/[id].tsx");
    }

    #[test]
    fn derive_index_route() {
        let (route, file) = derive_route_paths("index");
        assert_eq!(route, "/");
        assert_eq!(file, "index.tsx");
    }

    #[test]
    fn to_snake_pascal() {
        assert_eq!(to_snake("UserProfile"), "user_profile");
        assert_eq!(to_snake("Post"), "post");
        assert_eq!(to_snake("User"), "user");
    }

    #[test]
    fn to_snake_plural_simple() {
        assert_eq!(to_snake_plural("Post"), "posts");
        assert_eq!(to_snake_plural("User"), "users");
        assert_eq!(to_snake_plural("UserProfile"), "user_profiles");
    }
}
