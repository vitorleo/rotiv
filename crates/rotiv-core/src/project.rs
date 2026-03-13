use std::path::PathBuf;

use crate::error::RotivError;

/// Walk up from `cwd` until `.rotiv/spec.json` is found.
///
/// Returns the project root directory, or `E_NOT_A_PROJECT` if not found.
pub fn find_project_root() -> Result<PathBuf, RotivError> {
    let mut dir = std::env::current_dir().map_err(|e| {
        RotivError::new("E_IO", format!("cannot read current directory: {e}"))
    })?;

    loop {
        if dir.join(".rotiv").join("spec.json").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(
                RotivError::new("E_NOT_A_PROJECT", "not inside a Rotiv project")
                    .with_suggestion(
                        "Run `rotiv new <name>` to create a project, then `cd <name> && rotiv dev`",
                    ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_project_root_fails_outside_project() {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(std::env::temp_dir()).unwrap();
        let result = find_project_root();
        std::env::set_current_dir(original).unwrap();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "E_NOT_A_PROJECT");
    }
}
