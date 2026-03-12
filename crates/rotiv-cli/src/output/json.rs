use rotiv_core::RotivError;
use serde::Serialize;
use serde_json::Value;

/// Print a JSON success payload to stdout.
pub fn print_success(value: &impl Serialize) {
    println!("{}", serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()));
}

/// Print a JSON error payload to stderr and return exit code 1.
pub fn print_error(error: &RotivError) {
    #[derive(Serialize)]
    struct ErrorEnvelope<'a> {
        error: &'a RotivError,
    }
    let envelope = ErrorEnvelope { error };
    eprintln!(
        "{}",
        serde_json::to_string(&envelope).unwrap_or_else(|_| r#"{"error":{"code":"E_SERIALIZE","message":"failed to serialize error"}}"#.to_string())
    );
}

/// Print arbitrary JSON to stdout.
#[allow(dead_code)]
pub fn print_value(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string()));
}
