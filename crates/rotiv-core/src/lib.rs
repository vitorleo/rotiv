pub mod error;

pub use error::RotivError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotiv_error_creates_correctly() {
        let err = RotivError::new("E001", "Something went wrong");
        assert_eq!(err.code, "E001");
        assert_eq!(err.message, "Something went wrong");
        assert!(err.suggestion.is_none());
    }

    #[test]
    fn rotiv_error_with_suggestion() {
        let err = RotivError::new("E002", "Directory not found")
            .with_suggestion("Run `rotiv new <name>` to create a project");
        assert!(err.suggestion.is_some());
    }

    #[test]
    fn rotiv_error_serializes_to_json() {
        let err = RotivError::new("E001", "test error");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("E001"));
        assert!(json.contains("test error"));
    }
}
