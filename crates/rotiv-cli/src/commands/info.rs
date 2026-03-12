use crate::error::CliError;
use crate::output::{human, json, OutputMode};
use serde::Serialize;
use serde_json::Value;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
struct InfoOutput {
    framework_version: String,
    spec: Option<Value>,
}

pub fn run(mode: OutputMode) -> Result<(), CliError> {
    let spec = load_spec_if_present();

    match mode {
        OutputMode::Json => {
            json::print_success(&InfoOutput {
                framework_version: VERSION.to_string(),
                spec,
            });
        }
        OutputMode::Human => {
            human::print_header("Rotiv Framework");
            human::print_info("version", VERSION);

            if let Some(spec) = &spec {
                human::print_header("Project Spec");
                if let Some(project) = spec.get("project") {
                    if let Some(name) = project.get("name").and_then(|v| v.as_str()) {
                        human::print_info("name", name);
                    }
                    if let Some(created) = project.get("created_at").and_then(|v| v.as_str()) {
                        human::print_info("created", created);
                    }
                }
                if let Some(version) = spec.get("version").and_then(|v| v.as_str()) {
                    human::print_info("spec version", version);
                }
                if let Some(routes) = spec.get("routes").and_then(|v| v.as_array()) {
                    human::print_info("routes", &routes.len().to_string());
                }
                if let Some(models) = spec.get("models").and_then(|v| v.as_array()) {
                    human::print_info("models", &models.len().to_string());
                }
            } else {
                println!();
                println!("  Not inside a Rotiv project (no .rotiv/spec.json found).");
                println!("  Run `rotiv new <name>` to create one.");
            }
        }
    }

    Ok(())
}

fn load_spec_if_present() -> Option<Value> {
    // Walk up from cwd looking for .rotiv/spec.json
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let spec_path = dir.join(".rotiv").join("spec.json");
        if spec_path.exists() {
            let content = std::fs::read_to_string(&spec_path).ok()?;
            return serde_json::from_str(&content).ok();
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "version should be semver: {}", VERSION);
    }
}
