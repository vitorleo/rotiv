use rotiv_core::{apply_fixes, find_project_root, run_diagnostics, DiagnosticSeverity};

use crate::error::CliError;
use crate::output::OutputMode;

pub fn run(fix: bool, mode: OutputMode) -> Result<(), CliError> {
    let project_dir = find_project_root().map_err(CliError::Rotiv)?;
    let diagnostics = run_diagnostics(&project_dir).map_err(CliError::Rotiv)?;

    let mut fixed_count = 0;
    if fix && !diagnostics.is_empty() {
        fixed_count = apply_fixes(&diagnostics, &project_dir).map_err(CliError::Rotiv)?;
    }

    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Warning)
        .count();
    let ok = error_count == 0;

    match mode {
        OutputMode::Human => {
            if diagnostics.is_empty() {
                println!("  validate  ✓ no issues found");
            } else {
                for d in &diagnostics {
                    let severity_str = match d.severity {
                        DiagnosticSeverity::Error => "error",
                        DiagnosticSeverity::Warning => "warning",
                    };
                    let loc = match d.line {
                        Some(l) => format!("{}:{}", d.file, l),
                        None => d.file.clone(),
                    };
                    println!("  {}  [{}]  {}", loc, d.code, d.message);
                    println!("         hint: {}", d.suggestion);
                    println!("         severity: {}", severity_str);
                    println!();
                }
                println!(
                    "  validate  {} error(s), {} warning(s)",
                    error_count, warning_count
                );
                if fixed_count > 0 {
                    println!("  validate  auto-fixed {} issue(s)", fixed_count);
                }
            }
            if !ok {
                std::process::exit(1);
            }
        }
        OutputMode::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": ok,
                    "error_count": error_count,
                    "warning_count": warning_count,
                    "fixed": fixed_count,
                    "diagnostics": diagnostics,
                })
            );
            if !ok {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
