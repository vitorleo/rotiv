use std::path::{Path, PathBuf};

use crate::RotivError;

/// A model file entry discovered in `app/models/`.
#[derive(Debug, Clone)]
pub struct ModelEntry {
    /// PascalCase model name (e.g. `User`).
    pub name: String,
    /// Absolute path to the model file.
    pub file: PathBuf,
}

/// Discover model files in `<project_dir>/app/models/`.
///
/// Returns an empty Vec if the directory does not exist.
pub fn discover_models(project_dir: &Path) -> Result<Vec<ModelEntry>, RotivError> {
    let models_dir = project_dir.join("app").join("models");

    if !models_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    let read_dir = std::fs::read_dir(&models_dir).map_err(|e| {
        RotivError::new("E_MODELS_DIR", &format!("Cannot read models directory: {e}"))
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| {
            RotivError::new("E_MODELS_ENTRY", &format!("Cannot read directory entry: {e}"))
        })?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("ts") {
            continue;
        }

        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let name = snake_to_pascal(stem);
            entries.push(ModelEntry { name, file: path });
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
