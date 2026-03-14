use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Serialize;

use crate::RotivError;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// A single static-analysis finding.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// Diagnostic code, e.g. "V001"
    pub code: String,
    pub severity: DiagnosticSeverity,
    /// Relative file path
    pub file: String,
    /// 1-based line number, if applicable
    pub line: Option<u32>,
    pub message: String,
    pub suggestion: String,
    /// Replacement content for auto-fixable issues; None if not fixable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_fix: Option<String>,
}

/// Run all 10 static-analysis checks against the project.
///
/// Scans `app/routes/**/*.tsx`, `app/models/**/*.ts`, and `app/modules/**/*.json`.
pub fn run_diagnostics(project_dir: &Path) -> Result<Vec<Diagnostic>, RotivError> {
    let mut diags: Vec<Diagnostic> = Vec::new();

    let routes_dir = project_dir.join("app").join("routes");
    let models_dir = project_dir.join("app").join("models");
    let modules_dir = project_dir.join("app").join("modules");

    // --- Route file checks (V001, V002, V005, V006, V007) ---
    if routes_dir.exists() {
        walk_routes(&routes_dir, &routes_dir, project_dir, &mut diags);
    }

    // --- Model file checks (V003, V004) ---
    if models_dir.exists() {
        walk_models(&models_dir, project_dir, &mut diags);
    }

    // --- Module checks (V008, V009, V010) ---
    if modules_dir.exists() {
        walk_modules(&modules_dir, project_dir, &mut diags);
    }

    Ok(diags)
}

fn walk_routes(
    dir: &Path,
    routes_root: &Path,
    project_dir: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_routes(&path, routes_root, project_dir, diags);
        } else if path.extension().and_then(|e| e.to_str()) == Some("tsx") {
            check_route_file(&path, project_dir, diags);
        }
    }
}

fn check_route_file(file: &Path, project_dir: &Path, diags: &mut Vec<Diagnostic>) {
    let rel = file
        .strip_prefix(project_dir)
        .unwrap_or(file)
        .display()
        .to_string()
        .replace('\\', "/");

    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => return,
    };

    // V001: Route file missing `export default defineRoute`
    if !content.contains("export default defineRoute") {
        // V005 check: has `export default {` (raw object)?
        let has_raw_default = content.contains("export default {");

        if has_raw_default {
            // V005: raw object default export instead of defineRoute
            let line = find_line(&content, "export default {");
            diags.push(Diagnostic {
                code: "V005".to_string(),
                severity: DiagnosticSeverity::Error,
                file: rel.clone(),
                line,
                message: "Route has `export default { ... }` instead of `defineRoute({ ... })`"
                    .to_string(),
                suggestion: "Wrap the object in `defineRoute()`: `export default defineRoute({ path: \"...\", ... })`".to_string(),
                auto_fix: None,
            });
        } else {
            // V001: missing export default defineRoute entirely
            let auto_fix = Some(format!(
                "export default defineRoute({{\n{}}}\n)",
                content.trim()
            ));
            diags.push(Diagnostic {
                code: "V001".to_string(),
                severity: DiagnosticSeverity::Error,
                file: rel.clone(),
                line: None,
                message: "Route file missing `export default defineRoute(...)`".to_string(),
                suggestion: "The default export must be `defineRoute({ path, loader?, action?, component? })`".to_string(),
                auto_fix,
            });
        }
    }

    // V002: defineRoute missing `component` field
    if content.contains("defineRoute(") && !content.contains("component(") && !content.contains("component:") {
        let line = find_line(&content, "defineRoute(");
        diags.push(Diagnostic {
            code: "V002".to_string(),
            severity: DiagnosticSeverity::Warning,
            file: rel.clone(),
            line,
            message: "`defineRoute()` call is missing a `component` field".to_string(),
            suggestion: "Add a `component({ data }) { return <JSX />; }` field to defineRoute()".to_string(),
            auto_fix: None,
        });
    }

    // V006: loader uses ctx.db but no model import
    // Only trigger on non-comment lines to avoid matching template FRAMEWORK comments
    let uses_ctx_db = content.lines().any(|l| {
        let trimmed = l.trim();
        !trimmed.starts_with("//") && trimmed.contains("ctx.db")
    });
    if uses_ctx_db {
        // Accept any relative import that ends with /models/ (handles ../../models/ too)
        let has_model_import = content.contains("/models/");
        if !has_model_import {
            let line = find_line(&content, "ctx.db");
            diags.push(Diagnostic {
                code: "V006".to_string(),
                severity: DiagnosticSeverity::Warning,
                file: rel.clone(),
                line,
                message: "Route loader uses `ctx.db` but has no model import".to_string(),
                suggestion: "Import your model: `import { users } from \"../models/user.js\"`".to_string(),
                auto_fix: None,
            });
        }
    }

    // V007: filename has [param] but path string uses [param] instead of :param
    let filename = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if filename.contains('[') {
        // Check if any path: "..." line in the file doesn't use :param notation
        for (i, line_str) in content.lines().enumerate() {
            let trimmed = line_str.trim();
            if trimmed.starts_with("path:") && trimmed.contains('[') {
                diags.push(Diagnostic {
                    code: "V007".to_string(),
                    severity: DiagnosticSeverity::Error,
                    file: rel.clone(),
                    line: Some(i as u32 + 1),
                    message: "Route path uses `[param]` bracket notation; should use `:param`".to_string(),
                    suggestion: "Change `path: \"/users/[id]\"` to `path: \"/users/:id\"`".to_string(),
                    auto_fix: None,
                });
            }
        }
    }
}

fn walk_models(dir: &Path, project_dir: &Path, diags: &mut Vec<Diagnostic>) {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    for entry in read.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ts") {
            check_model_file(&path, project_dir, diags);
        }
    }
}

fn check_model_file(file: &Path, project_dir: &Path, diags: &mut Vec<Diagnostic>) {
    let rel = file
        .strip_prefix(project_dir)
        .unwrap_or(file)
        .display()
        .to_string()
        .replace('\\', "/");

    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => return,
    };

    // V003: model file lacks raw table export (sqliteTable or pgTable)
    if !content.contains("sqliteTable(") && !content.contains("pgTable(") {
        diags.push(Diagnostic {
            code: "V003".to_string(),
            severity: DiagnosticSeverity::Error,
            file: rel.clone(),
            line: None,
            message: "Model file missing a raw table export (`sqliteTable()` or `pgTable()`)".to_string(),
            suggestion: "Export a raw Drizzle table: `export const users = sqliteTable(\"users\", { ... })`\ndrizzle-kit requires this for migration generation.".to_string(),
            auto_fix: None,
        });
    }

    // V004: model file lacks defineModel call
    if !content.contains("defineModel(") {
        diags.push(Diagnostic {
            code: "V004".to_string(),
            severity: DiagnosticSeverity::Error,
            file: rel.clone(),
            line: None,
            message: "Model file missing `defineModel()` call".to_string(),
            suggestion: "Add: `export const UserModel = defineModel(\"User\", users)`\nThis registers the model in Rotiv's runtime registry.".to_string(),
            auto_fix: None,
        });
    }
}

fn walk_modules(dir: &Path, project_dir: &Path, diags: &mut Vec<Diagnostic>) {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            check_module_dir(&path, project_dir, diags);
        }
    }
}

fn check_module_dir(module_dir: &Path, _project_dir: &Path, diags: &mut Vec<Diagnostic>) {
    let name = module_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let rel_dir = format!("app/modules/{}", name);

    // V008: module.json missing
    let manifest_path = module_dir.join("module.json");
    if !manifest_path.exists() {
        diags.push(Diagnostic {
            code: "V008".to_string(),
            severity: DiagnosticSeverity::Error,
            file: format!("{}/module.json", rel_dir),
            line: None,
            message: format!("Module '{}' is missing module.json manifest", name),
            suggestion: "Run `rotiv add module <name>` to scaffold a module with the correct structure.".to_string(),
            auto_fix: None,
        });
        return; // can't do further checks without the manifest
    }

    // V009: module.json missing required fields (name, version, provides)
    let manifest_content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let manifest: serde_json::Value = match serde_json::from_str(&manifest_content) {
        Ok(v) => v,
        Err(e) => {
            diags.push(Diagnostic {
                code: "V009".to_string(),
                severity: DiagnosticSeverity::Error,
                file: format!("{}/module.json", rel_dir),
                line: None,
                message: format!("module.json in '{}' is not valid JSON: {}", name, e),
                suggestion: "Fix the JSON syntax in module.json.".to_string(),
                auto_fix: None,
            });
            return;
        }
    };

    let missing_fields: Vec<&str> = ["name", "version", "provides"]
        .iter()
        .filter(|&&f| manifest.get(f).is_none())
        .copied()
        .collect();

    if !missing_fields.is_empty() {
        diags.push(Diagnostic {
            code: "V009".to_string(),
            severity: DiagnosticSeverity::Error,
            file: format!("{}/module.json", rel_dir),
            line: None,
            message: format!(
                "module.json in '{}' is missing required fields: {}",
                name,
                missing_fields.join(", ")
            ),
            suggestion: "Add the missing fields to module.json. Required: name, version, provides.".to_string(),
            auto_fix: None,
        });
    }

    // V010: module requires a capability not provided by any other module
    // (simple check: if "requires" lists items, ensure index.ts exists as entry point)
    let entry_path = module_dir.join("index.ts");
    if !entry_path.exists() {
        diags.push(Diagnostic {
            code: "V010".to_string(),
            severity: DiagnosticSeverity::Error,
            file: format!("{}/index.ts", rel_dir),
            line: None,
            message: format!("Module '{}' is missing its entry file index.ts", name),
            suggestion: "Run `rotiv add module <name>` to scaffold a proper module structure with index.ts.".to_string(),
            auto_fix: None,
        });
    }
}

/// Apply auto-fixes to files that have fixable diagnostics.
pub fn apply_fixes(diagnostics: &[Diagnostic], project_dir: &Path) -> Result<usize, RotivError> {
    let mut fixed = 0;

    for diag in diagnostics {
        if let Some(fix) = &diag.auto_fix {
            let abs_path = project_dir.join(&diag.file);
            fs::write(&abs_path, fix)
                .map_err(|e| RotivError::new("E_IO", format!("failed to write fix to {}: {}", diag.file, e)))?;
            fixed += 1;
        }
    }

    Ok(fixed)
}

/// Return the 1-based line number of the first line containing `needle`.
fn find_line(content: &str, needle: &str) -> Option<u32> {
    let reader = BufReader::new(content.as_bytes());
    for (i, line) in reader.lines().enumerate() {
        if let Ok(l) = line {
            if l.contains(needle) {
                return Some(i as u32 + 1);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_line_finds_needle() {
        let content = "line one\nline two\ndefineRoute(\nline four";
        assert_eq!(find_line(content, "defineRoute("), Some(3));
    }

    #[test]
    fn find_line_missing() {
        let content = "nothing here";
        assert_eq!(find_line(content, "defineRoute("), None);
    }
}
