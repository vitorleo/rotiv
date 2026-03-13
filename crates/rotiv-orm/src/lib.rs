//! rotiv-orm — database migration and model discovery for Phase 4.
pub mod discovery;
pub mod error;
pub mod migration;

pub use discovery::{discover_models, ModelFileEntry};
pub use error::OrmError;
pub use migration::{
    auto_migrate, resolve_migrate_script_path, run_migrations, MigrationOptions, MigrationResult,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orm_error_variants() {
        let err = OrmError::NotImplemented("query builder".to_string());
        assert!(err.to_string().contains("Not implemented"));

        let err = OrmError::ScriptNotFound("no path".to_string());
        assert!(err.to_string().contains("Script not found"));

        let err = OrmError::MigrationFailed("syntax error".to_string());
        assert!(err.to_string().contains("Migration failed"));
    }

    #[test]
    fn discover_models_empty_dir() {
        let tmp = std::env::temp_dir().join("rotiv_orm_lib_test");
        let result = discover_models(&tmp).unwrap();
        assert!(result.is_empty());
    }
}
