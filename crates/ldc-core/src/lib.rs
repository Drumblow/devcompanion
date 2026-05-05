use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: Option<String>,
    pub path: Option<String>,
    pub git_branch: Option<String>,
    pub git_remote: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivityPayload {
    #[serde(default)]
    pub files_modified: Vec<String>,
    #[serde(default)]
    pub languages: BTreeMap<String, u64>,
    #[serde(default)]
    pub lines_added: i64,
    #[serde(default)]
    pub lines_removed: i64,
    #[serde(default)]
    pub functions_touched: Vec<String>,
    #[serde(default)]
    pub time_spent_minutes: i64,
    #[serde(default)]
    pub errors_encountered: i64,
    #[serde(default)]
    pub libraries_used: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingEvent {
    pub timestamp: Option<DateTime<Utc>>,
    pub session_id: String,
    pub event_type: String,
    pub project: ProjectContext,
    #[serde(default)]
    pub activity: ActivityPayload,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: NaiveDate,
    pub event_count: i64,
    pub total_time_minutes: i64,
    pub lines_added: i64,
    pub lines_removed: i64,
    pub git_commits: i64,
    pub projects: Vec<String>,
    pub languages: BTreeMap<String, i64>,
    pub files_modified: Vec<String>,
    pub voice_examples: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDraftRequest {
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedDraft {
    pub id: i64,
    pub session_date: NaiveDate,
    pub content: String,
    pub status: String,
    pub model: String,
    pub context_audit: Value,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveDraftRequest {
    pub approved_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceExampleRequest {
    pub text: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub database: String,
}
