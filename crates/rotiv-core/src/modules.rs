use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::RotivError;

/// Module capability tier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleTier {
    Primitive,
    Slot,
    EscapeHatch,
}

impl Default for ModuleTier {
    fn default() -> Self {
        ModuleTier::Slot
    }
}

/// Parsed module manifest (`app/modules/<name>/module.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub configures: Vec<String>,
    #[serde(default)]
    pub tier: ModuleTier,
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default)]
    pub test: Option<String>,
}

/// A conflict: two or more modules providing the same capability.
#[derive(Debug, Clone, Serialize)]
pub struct CapabilityConflict {
    pub capability: String,
    pub provided_by: Vec<String>,
}

/// A missing requirement: a module requires a capability no module provides.
#[derive(Debug, Clone, Serialize)]
pub struct MissingRequirement {
    pub module: String,
    pub requires: String,
}

/// Parse a single `module.json` file.
pub fn parse_manifest(path: &Path) -> Result<ModuleManifest, RotivError> {
    let raw = fs::read_to_string(path).map_err(|e| {
        RotivError::new("E030", format!("failed to read {}: {}", path.display(), e))
    })?;
    serde_json::from_str::<ModuleManifest>(&raw).map_err(|e| {
        RotivError::new(
            "E031",
            format!("invalid module.json at {}: {}", path.display(), e),
        )
        .with_suggestion("Check that module.json is valid JSON with required fields: name, version")
    })
}

/// Scan `<project_dir>/app/modules/*/module.json` and parse each manifest.
///
/// Returns an empty Vec if the modules directory does not exist.
pub fn discover_modules(project_dir: &Path) -> Result<Vec<ModuleManifest>, RotivError> {
    let modules_dir = project_dir.join("app").join("modules");

    if !modules_dir.exists() {
        return Ok(Vec::new());
    }

    let mut manifests = Vec::new();

    for entry in fs::read_dir(&modules_dir).map_err(|e| {
        RotivError::new("E032", format!("failed to read modules dir: {}", e))
    })? {
        let entry = entry.map_err(|e| {
            RotivError::new("E032", format!("failed to read dir entry: {}", e))
        })?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("module.json");
        if manifest_path.exists() {
            let manifest = parse_manifest(&manifest_path)?;
            manifests.push(manifest);
        }
    }

    manifests.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(manifests)
}

/// Check capability consistency across installed modules.
///
/// Returns:
/// - `conflicts`: capabilities provided by more than one module
/// - `missing`: capabilities required by a module but not provided by any
pub fn resolve_capabilities(
    modules: &[ModuleManifest],
) -> (Vec<CapabilityConflict>, Vec<MissingRequirement>) {
    // Build map: capability → list of provider module names
    let mut providers: HashMap<String, Vec<String>> = HashMap::new();
    for m in modules {
        for cap in &m.provides {
            providers
                .entry(cap.clone())
                .or_default()
                .push(m.name.clone());
        }
    }

    // Find conflicts: capability provided by more than one module
    let conflicts: Vec<CapabilityConflict> = providers
        .iter()
        .filter(|(_, names)| names.len() > 1)
        .map(|(cap, names)| CapabilityConflict {
            capability: cap.clone(),
            provided_by: names.clone(),
        })
        .collect();

    // Find missing: required capability not in providers map
    let mut missing = Vec::new();
    for m in modules {
        for req in &m.requires {
            if !providers.contains_key(req) {
                missing.push(MissingRequirement {
                    module: m.name.clone(),
                    requires: req.clone(),
                });
            }
        }
    }

    (conflicts, missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manifest(name: &str, provides: &[&str], requires: &[&str]) -> ModuleManifest {
        ModuleManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: None,
            provides: provides.iter().map(|s| s.to_string()).collect(),
            requires: requires.iter().map(|s| s.to_string()).collect(),
            configures: Vec::new(),
            tier: ModuleTier::Slot,
            entry: None,
            test: None,
        }
    }

    #[test]
    fn resolve_no_issues() {
        let modules = vec![
            make_manifest("sessions", &["sessions"], &[]),
            make_manifest("auth", &["auth"], &["sessions"]),
        ];
        let (conflicts, missing) = resolve_capabilities(&modules);
        assert!(conflicts.is_empty());
        assert!(missing.is_empty());
    }

    #[test]
    fn resolve_conflict() {
        let modules = vec![
            make_manifest("auth-a", &["auth"], &[]),
            make_manifest("auth-b", &["auth"], &[]),
        ];
        let (conflicts, missing) = resolve_capabilities(&modules);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].capability, "auth");
        assert!(missing.is_empty());
    }

    #[test]
    fn resolve_missing_requirement() {
        let modules = vec![
            make_manifest("auth", &["auth"], &["sessions"]),
        ];
        let (conflicts, missing) = resolve_capabilities(&modules);
        assert!(conflicts.is_empty());
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].module, "auth");
        assert_eq!(missing[0].requires, "sessions");
    }

    #[test]
    fn discover_modules_missing_dir() {
        let tmp = std::env::temp_dir().join("rotiv_no_modules_dir_test");
        let result = discover_modules(&tmp).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_manifest_valid() {
        let json = r#"{
            "name": "test",
            "version": "1.0.0",
            "provides": ["test"],
            "requires": [],
            "configures": ["middleware"],
            "tier": "slot"
        }"#;
        let m: ModuleManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.name, "test");
        assert_eq!(m.provides, vec!["test"]);
        assert!(matches!(m.tier, ModuleTier::Slot));
    }
}
