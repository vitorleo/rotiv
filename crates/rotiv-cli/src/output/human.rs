use colored::Colorize;
use rotiv_core::RotivError;

/// Print a success message with a green checkmark.
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print an info line.
pub fn print_info(label: &str, value: &str) {
    println!("  {} {}", format!("{label}:").dimmed(), value);
}

/// Print a section header.
pub fn print_header(title: &str) {
    println!("\n{}", title.bold().underline());
}

/// Print a structured error to stderr with colors.
pub fn print_error(error: &RotivError) {
    eprintln!("{} [{}] {}", "error".red().bold(), error.code.yellow(), error.message);

    if let (Some(file), line) = (&error.file, error.line) {
        if let Some(l) = line {
            eprintln!("  {} {}:{}", "-->".dimmed(), file, l);
        } else {
            eprintln!("  {} {}", "-->".dimmed(), file);
        }
    }

    if let (Some(expected), Some(got)) = (&error.expected, &error.got) {
        eprintln!("  {} {}", "expected:".dimmed(), expected);
        eprintln!("  {}     {}", "got:".dimmed(), got);
    }

    if let Some(suggestion) = &error.suggestion {
        eprintln!("  {} {}", "hint:".cyan().bold(), suggestion);
    }
}
