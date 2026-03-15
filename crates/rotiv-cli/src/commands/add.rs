use std::path::Path;

use rotiv_core::{RotivError, find_project_root};
use serde::Serialize;

use crate::error::CliError;
use crate::output::{OutputMode, human, json};

const ROUTE_TEMPLATE: &str = include_str!("../templates/add/route.tsx");
const MODEL_TEMPLATE: &str = include_str!("../templates/add/model.ts");
const MODULE_MANIFEST_TEMPLATE: &str = include_str!("../templates/add/module_manifest.json");
const MODULE_INDEX_TEMPLATE: &str = include_str!("../templates/add/module_index.ts");
const MODULE_TEST_TEMPLATE: &str = include_str!("../templates/add/module_test.ts");

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
        let corrected = to_pascal_case(name);
        let err = RotivError::new(
            "E011",
            format!("invalid model name '{}': must be PascalCase (e.g. Post, UserProfile)", name),
        )
        .with_expected("PascalCase name (e.g. Post)", name)
        .with_suggestion(format!("Did you mean '{}'?", corrected))
        .with_corrected_code(corrected);
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

/// `rotiv add module <name>` — scaffold a module directory with manifest, entry, and test.
pub fn run_add_module(name: &str, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;

    // Validate: lowercase alphanumeric + hyphens
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        let corrected = to_kebab_case(name);
        let err = RotivError::new(
            "E012",
            format!(
                "invalid module name '{}': must be lowercase alphanumeric with hyphens (e.g. auth, my-module)",
                name
            ),
        )
        .with_expected("lowercase-hyphen-name (e.g. auth, file-uploads)", name)
        .with_suggestion(format!("Did you mean '{}'?", corrected))
        .with_corrected_code(corrected);
        return Err(CliError::Rotiv(err));
    }

    let module_dir = project_dir.join("app").join("modules").join(name);

    if module_dir.exists() {
        let err = RotivError::new(
            "E010",
            format!("module already exists: app/modules/{}", name),
        )
        .with_suggestion("Use a different name or delete the existing module directory first.");
        return Err(CliError::Rotiv(err));
    }

    std::fs::create_dir_all(&module_dir)?;

    // Check if this is a first-party module
    let (manifest_content, index_content, test_content) = first_party_module(name).unwrap_or_else(|| {
        (
            MODULE_MANIFEST_TEMPLATE.replace("{{module_name}}", name),
            MODULE_INDEX_TEMPLATE.replace("{{module_name}}", name),
            MODULE_TEST_TEMPLATE.replace("{{module_name}}", name),
        )
    });

    std::fs::write(module_dir.join("module.json"), &manifest_content)?;
    std::fs::write(module_dir.join("index.ts"), &index_content)?;
    std::fs::write(module_dir.join("module.test.ts"), &test_content)?;

    // Update .rotiv/spec.json modules array
    add_module_to_spec(&project_dir, name, &manifest_content)?;

    let rel = format!("app/modules/{}/", name);
    match mode {
        OutputMode::Human => {
            human::print_success(&format!("created {}", rel));
            human::print_info("files", "module.json, index.ts, module.test.ts");
            println!();
            println!("  Next steps:");
            println!("    Import and use in a route:");
            println!(
                "    import {{ {}Middleware }} from \"../modules/{}/index.js\";",
                to_camel(name),
                name
            );
        }
        OutputMode::Json => json::print_success(&AddSuccess {
            ok: true,
            kind: "module".to_string(),
            file: rel,
        }),
    }
    Ok(())
}

/// Update spec.json to include the new module entry.
fn add_module_to_spec(project_dir: &Path, name: &str, manifest_json: &str) -> Result<(), CliError> {
    let spec_path = project_dir.join(".rotiv").join("spec.json");
    if !spec_path.exists() {
        return Ok(());
    }

    let raw = std::fs::read_to_string(&spec_path)?;
    let mut spec: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or(serde_json::json!({"version": "1"}));

    // Parse manifest to get version
    let version = serde_json::from_str::<serde_json::Value>(manifest_json)
        .ok()
        .and_then(|v| v["version"].as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "0.1.0".to_string());

    // Add to modules array (avoid duplicates)
    let modules = spec["modules"].as_array_mut();
    if let Some(arr) = modules {
        let already_present = arr.iter().any(|m| m["name"].as_str() == Some(name));
        if !already_present {
            arr.push(serde_json::json!({ "name": name, "version": version }));
        }
    } else {
        spec["modules"] = serde_json::json!([{ "name": name, "version": version }]);
    }

    let json_str = serde_json::to_string_pretty(&spec).unwrap_or_default();
    std::fs::write(&spec_path, json_str)?;
    Ok(())
}

/// Convert any string to PascalCase. "user_profile" / "user-profile" → "UserProfile"
fn to_pascal_case(name: &str) -> String {
    name.split(|c: char| c == '_' || c == '-' || c == ' ')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}

/// Convert any string to kebab-case. "MyModule" / "my_module" → "my-module"
fn to_kebab_case(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('-');
            out.push(ch.to_lowercase().next().unwrap_or(ch));
        } else if ch == '_' || ch == ' ' {
            out.push('-');
        } else {
            out.push(ch.to_lowercase().next().unwrap_or(ch));
        }
    }
    out
}

/// Convert hyphen-case to camelCase for use in import statements.
/// "file-uploads" → "fileUploads"
fn to_camel(name: &str) -> String {
    let mut out = String::new();
    let mut capitalize_next = false;
    for ch in name.chars() {
        if ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            out.push(ch.to_uppercase().next().unwrap_or(ch));
            capitalize_next = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Returns first-party module content (manifest, index, test) for known module names.
fn first_party_module(name: &str) -> Option<(String, String, String)> {
    match name {
        "sessions" => Some((
            include_str!("../modules/sessions/module.json").to_string(),
            include_str!("../modules/sessions/index.ts").to_string(),
            include_str!("../modules/sessions/module.test.ts").to_string(),
        )),
        "auth" => Some((
            include_str!("../modules/auth/module.json").to_string(),
            include_str!("../modules/auth/index.ts").to_string(),
            include_str!("../modules/auth/module.test.ts").to_string(),
        )),
        "file-uploads" => Some((
            include_str!("../modules/file-uploads/module.json").to_string(),
            include_str!("../modules/file-uploads/index.ts").to_string(),
            include_str!("../modules/file-uploads/module.test.ts").to_string(),
        )),
        _ => None,
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
