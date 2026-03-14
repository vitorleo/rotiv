use rotiv_core::RotivError;

use crate::error::CliError;
use crate::output::OutputMode;

const TOPICS: &[(&str, &str)] = &[
    ("routes", include_str!("../knowledge/routes.md")),
    ("models", include_str!("../knowledge/models.md")),
    ("loader", include_str!("../knowledge/loader.md")),
    ("action", include_str!("../knowledge/action.md")),
    ("middleware", include_str!("../knowledge/middleware.md")),
    ("signals", include_str!("../knowledge/signals.md")),
    ("migrate", include_str!("../knowledge/migrate.md")),
    ("context", include_str!("../knowledge/context.md")),
    ("modules", include_str!("../knowledge/modules.md")),
];

pub fn run(topic: &str, mode: OutputMode) -> Result<(), CliError> {
    let lower = topic.to_lowercase();

    // Fuzzy match: exact → prefix → contains
    let found = TOPICS
        .iter()
        .find(|(name, _)| *name == lower)
        .or_else(|| TOPICS.iter().find(|(name, _)| name.starts_with(&lower[..])))
        .or_else(|| TOPICS.iter().find(|(name, _)| name.contains(&lower[..])));

    let (name, content) = match found {
        Some(t) => t,
        None => {
            let available: Vec<&str> = TOPICS.iter().map(|(n, _)| *n).collect();
            let err = RotivError::new(
                "E020",
                format!("unknown topic '{}'. Available topics: {}", topic, available.join(", ")),
            )
            .with_suggestion(format!(
                "Try: rotiv explain {}",
                available[0]
            ));
            return Err(CliError::Rotiv(err));
        }
    };

    match mode {
        OutputMode::Human => {
            print!("{}", content);
        }
        OutputMode::Json => {
            let parsed = parse_topic(name, content);
            println!("{}", serde_json::to_string_pretty(&parsed).unwrap_or_default());
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct TopicJson {
    topic: String,
    explanation: String,
    code_example: String,
    related: Vec<String>,
}

/// Parse a Markdown topic file into structured sections.
fn parse_topic(name: &str, content: &str) -> TopicJson {
    let mut explanation = String::new();
    let mut code_example = String::new();
    let mut related: Vec<String> = Vec::new();

    #[derive(PartialEq)]
    enum Section {
        None,
        Explanation,
        CodeExample,
        Related,
    }

    let mut section = Section::None;
    let mut in_code_block = false;

    for line in content.lines() {
        if line.starts_with("## Explanation") {
            section = Section::Explanation;
            continue;
        }
        if line.starts_with("## Code Example") {
            section = Section::CodeExample;
            continue;
        }
        if line.starts_with("## Related") {
            section = Section::Related;
            continue;
        }
        // Skip the H1 title line
        if line.starts_with("# ") {
            continue;
        }

        match section {
            Section::Explanation => {
                explanation.push_str(line);
                explanation.push('\n');
            }
            Section::CodeExample => {
                if line.starts_with("```") {
                    in_code_block = !in_code_block;
                    if !in_code_block {
                        // closing fence — don't include it
                        continue;
                    }
                    if in_code_block {
                        // opening fence — don't include it
                        continue;
                    }
                }
                if in_code_block || !line.starts_with("```") {
                    code_example.push_str(line);
                    code_example.push('\n');
                }
            }
            Section::Related => {
                let trimmed = line.trim_start_matches('-').trim();
                // Handle comma-separated or single items
                for item in trimmed.split(',') {
                    let t = item.trim();
                    if !t.is_empty() {
                        related.push(t.to_string());
                    }
                }
            }
            Section::None => {}
        }
    }

    TopicJson {
        topic: name.to_string(),
        explanation: explanation.trim().to_string(),
        code_example: code_example.trim().to_string(),
        related,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_topics_load() {
        assert_eq!(TOPICS.len(), 9);
        for (name, content) in TOPICS {
            assert!(!name.is_empty());
            assert!(!content.is_empty(), "topic '{}' is empty", name);
        }
    }

    #[test]
    fn parse_produces_explanation() {
        let (name, content) = TOPICS[0]; // routes
        let parsed = parse_topic(name, content);
        assert!(!parsed.explanation.is_empty());
    }
}
