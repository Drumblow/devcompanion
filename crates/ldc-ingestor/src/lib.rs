use anyhow::{bail, Context};
use ldc_core::CodingEvent;
use serde_json::Value;
use std::collections::BTreeMap;

const ALLOWED_EVENT_TYPES: &[&str] = &[
    "document_open",
    "document_edit",
    "active_editor",
    "git_commit",
    "session_heartbeat",
    "manual_note",
];

const SENSITIVE_MARKERS: &[&str] = &[
    ".env", "secret", "password", "token", "apikey", "api_key", ".pem", ".key",
];

pub fn normalize_event(mut event: CodingEvent) -> anyhow::Result<CodingEvent> {
    event.event_type = event.event_type.trim().to_ascii_lowercase();
    if !ALLOWED_EVENT_TYPES.contains(&event.event_type.as_str()) {
        bail!("event_type nao suportado: {}", event.event_type);
    }

    event.session_id = event.session_id.trim().to_string();
    if event.session_id.is_empty() {
        bail!("session_id e obrigatorio");
    }

    event.project.name = clean_optional(event.project.name);
    event.project.path = clean_optional(event.project.path);
    event.project.git_branch = clean_optional(event.project.git_branch);
    event.project.git_remote = clean_optional(event.project.git_remote);

    event.activity.lines_added = event.activity.lines_added.max(0);
    event.activity.lines_removed = event.activity.lines_removed.max(0);
    event.activity.time_spent_minutes = event.activity.time_spent_minutes.clamp(0, 24 * 60);
    event.activity.errors_encountered = event.activity.errors_encountered.max(0);
    event.activity.files_modified = normalize_files(event.activity.files_modified);
    event.activity.functions_touched = normalize_strings(event.activity.functions_touched);
    event.activity.libraries_used = normalize_strings(event.activity.libraries_used);
    event.activity.languages = normalize_languages(event.activity.languages);
    event.metadata = normalize_metadata(event.metadata).context("metadata invalido")?;

    Ok(event)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_files(files: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for file in files {
        let file = file.trim().replace('\\', "/");
        if file.is_empty() || is_sensitive_path(&file) || normalized.contains(&file) {
            continue;
        }
        normalized.push(file);
    }
    normalized.sort();
    normalized
}

fn normalize_strings(values: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let value = value.trim().to_string();
        if !value.is_empty() && !normalized.contains(&value) {
            normalized.push(value);
        }
    }
    normalized.sort();
    normalized
}

fn normalize_languages(languages: BTreeMap<String, u64>) -> BTreeMap<String, u64> {
    let mut normalized = BTreeMap::new();
    for (language, minutes) in languages {
        let language = language.trim().to_ascii_lowercase();
        if !language.is_empty() && minutes > 0 {
            *normalized.entry(language).or_insert(0) += minutes.min(24 * 60);
        }
    }
    normalized
}

fn normalize_metadata(metadata: Value) -> anyhow::Result<Value> {
    match metadata {
        Value::Object(mut map) => {
            for key in ["secret", "token", "password", "apiKey", "api_key"] {
                map.remove(key);
            }
            Ok(Value::Object(map))
        }
        Value::Null => Ok(Value::Object(Default::default())),
        value => Ok(value),
    }
}

fn is_sensitive_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    SENSITIVE_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldc_core::{ActivityPayload, ProjectContext};
    use serde_json::json;

    #[test]
    fn normalizes_and_redacts_sensitive_fields() {
        let event = CodingEvent {
            timestamp: None,
            session_id: " session ".to_string(),
            event_type: "DOCUMENT_EDIT".to_string(),
            project: ProjectContext {
                name: Some(" demo ".to_string()),
                path: None,
                git_branch: None,
                git_remote: None,
            },
            activity: ActivityPayload {
                files_modified: vec!["src\\main.rs".to_string(), ".env".to_string()],
                languages: BTreeMap::from([("Rust".to_string(), 5)]),
                lines_added: -1,
                lines_removed: 2,
                functions_touched: vec![],
                time_spent_minutes: 10,
                errors_encountered: 0,
                libraries_used: vec![],
            },
            metadata: json!({ "token": "hidden", "source": "test" }),
        };

        let normalized = normalize_event(event).unwrap();
        assert_eq!(normalized.event_type, "document_edit");
        assert_eq!(normalized.session_id, "session");
        assert_eq!(normalized.activity.files_modified, vec!["src/main.rs"]);
        assert_eq!(normalized.activity.lines_added, 0);
        assert!(normalized.metadata.get("token").is_none());
    }
}
