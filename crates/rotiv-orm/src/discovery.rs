use std::path::{Path, PathBuf};

use crate::OrmError;

/// A single model file found in the project's `app/models/` directory.
#[derive(Debug, Clone)]
pub struct ModelFileEntry {
    /// PascalCase model name derived from the filename (e.g. `user.ts` → `User`).
    pub name: String,
    /// Absolute path to the model file.
    pub file: PathBuf,
}

/// Scan `<project_dir>/app/models/` for `*.ts` files.
///
/// Converts snake_case filenames to PascalCase for the model name
/// (`user_profile.ts` → `UserProfile`). Returns an empty Vec if the
/// directory does not exist.
pub fn discover_models(project_dir: &Path) -> Result<Vec<ModelFileEntry>, OrmError> {
    let models_dir = project_dir.join("app").join("models");

    if !models_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    for entry in std::fs::read_dir(&models_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("ts") {
            continue;
        }

        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let name = snake_to_pascal(stem);
            entries.push(ModelFileEntry { name, file: path });
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Convert `snake_case` to `PascalCase`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_to_pascal_single() {
        assert_eq!(snake_to_pascal("user"), "User");
    }

    #[test]
    fn snake_to_pascal_multi() {
        assert_eq!(snake_to_pascal("user_profile"), "UserProfile");
    }

    #[test]
    fn snake_to_pascal_already_pascal() {
        assert_eq!(snake_to_pascal("User"), "User");
    }

    #[test]
    fn discover_models_missing_dir() {
        let tmp = std::env::temp_dir().join("rotiv_no_models_test");
        let result = discover_models(&tmp).unwrap();
        assert!(result.is_empty());
    }
}
