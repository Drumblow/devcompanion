use chrono::{DateTime, NaiveDate, Utc};
use ldc_core::{
    CodingEvent, DailySummary, GeneratedDraft, RankedVoiceExample, RecentEvent, StoredEvent,
    VoiceExampleRequest,
};
use serde_json::{json, Value};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::{collections::BTreeMap, path::Path, str::FromStr};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(path: &Path) -> anyhow::Result<Self> {
        let url = format!("sqlite://{}", path.display());
        let options = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS coding_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                session_id TEXT NOT NULL,
                event_date TEXT NOT NULL,
                event_type TEXT NOT NULL,
                project_name TEXT,
                project_path TEXT,
                git_branch TEXT,
                git_remote TEXT,
                files_modified TEXT NOT NULL,
                languages TEXT NOT NULL,
                lines_added INTEGER NOT NULL DEFAULT 0,
                lines_removed INTEGER NOT NULL DEFAULT 0,
                time_spent_minutes INTEGER NOT NULL DEFAULT 0,
                metadata TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS daily_sessions (
                date TEXT PRIMARY KEY,
                event_count INTEGER NOT NULL DEFAULT 0,
                total_time_minutes INTEGER NOT NULL DEFAULT 0,
                projects TEXT NOT NULL,
                languages TEXT NOT NULL,
                files_modified TEXT NOT NULL,
                git_commits INTEGER NOT NULL DEFAULT 0,
                lines_added INTEGER NOT NULL DEFAULT 0,
                lines_removed INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS voice_examples (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                context TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS generated_drafts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_date TEXT NOT NULL,
                content TEXT NOT NULL,
                status TEXT NOT NULL,
                model TEXT NOT NULL,
                context_audit TEXT NOT NULL,
                style_score REAL,
                created_at TEXT NOT NULL,
                approved_at TEXT,
                rejected_at TEXT,
                rejection_reason TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        self.ensure_column("generated_drafts", "style_score", "REAL")
            .await?;
        self.ensure_column("generated_drafts", "rejected_at", "TEXT")
            .await?;
        self.ensure_column("generated_drafts", "rejection_reason", "TEXT")
            .await?;

        Ok(())
    }

    async fn ensure_column(&self, table: &str, column: &str, definition: &str) -> sqlx::Result<()> {
        let pragma = format!("PRAGMA table_info({table})");
        let rows = sqlx::query(&pragma).fetch_all(&self.pool).await?;
        let exists = rows.iter().any(|row| {
            row.try_get::<String, _>("name")
                .map(|name| name == column)
                .unwrap_or(false)
        });
        if !exists {
            let alter = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
            sqlx::query(&alter).execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn insert_event(&self, event: CodingEvent) -> anyhow::Result<StoredEvent> {
        let timestamp = event.timestamp.unwrap_or_else(Utc::now);
        let event_date = timestamp.date_naive().to_string();
        let files_modified = serde_json::to_string(&event.activity.files_modified)?;
        let languages = serde_json::to_string(&event.activity.languages)?;
        let metadata = serde_json::to_string(&event.metadata)?;

        let result = sqlx::query(
            r#"
            INSERT INTO coding_events (
                timestamp, session_id, event_date, event_type, project_name, project_path,
                git_branch, git_remote, files_modified, languages, lines_added,
                lines_removed, time_spent_minutes, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(timestamp.to_rfc3339())
        .bind(&event.session_id)
        .bind(event_date)
        .bind(&event.event_type)
        .bind(&event.project.name)
        .bind(&event.project.path)
        .bind(&event.project.git_branch)
        .bind(&event.project.git_remote)
        .bind(files_modified)
        .bind(languages)
        .bind(event.activity.lines_added)
        .bind(event.activity.lines_removed)
        .bind(event.activity.time_spent_minutes)
        .bind(metadata)
        .execute(&self.pool)
        .await?;

        Ok(StoredEvent {
            id: result.last_insert_rowid(),
            timestamp,
            session_id: event.session_id,
            event_type: event.event_type,
        })
    }

    pub async fn refresh_daily_session(&self, date: NaiveDate) -> anyhow::Result<DailySummary> {
        let summary = self.daily_summary(date).await?;
        sqlx::query(
            r#"
            INSERT INTO daily_sessions (
                date, event_count, total_time_minutes, projects, languages, files_modified,
                git_commits, lines_added, lines_removed, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(date) DO UPDATE SET
                event_count = excluded.event_count,
                total_time_minutes = excluded.total_time_minutes,
                projects = excluded.projects,
                languages = excluded.languages,
                files_modified = excluded.files_modified,
                git_commits = excluded.git_commits,
                lines_added = excluded.lines_added,
                lines_removed = excluded.lines_removed,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(date.to_string())
        .bind(summary.event_count)
        .bind(summary.total_time_minutes)
        .bind(serde_json::to_string(&summary.projects)?)
        .bind(serde_json::to_string(&summary.languages)?)
        .bind(serde_json::to_string(&summary.files_modified)?)
        .bind(summary.git_commits)
        .bind(summary.lines_added)
        .bind(summary.lines_removed)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(summary)
    }

    pub async fn daily_summary(&self, date: NaiveDate) -> anyhow::Result<DailySummary> {
        let date_text = date.to_string();
        let totals = sqlx::query(
            r#"
            SELECT
                COUNT(*) AS event_count,
                COALESCE(SUM(time_spent_minutes), 0) AS total_time_minutes,
                COALESCE(SUM(lines_added), 0) AS lines_added,
                COALESCE(SUM(lines_removed), 0) AS lines_removed,
                COALESCE(SUM(CASE WHEN event_type = 'git_commit' THEN 1 ELSE 0 END), 0) AS git_commits
            FROM coding_events
            WHERE event_date = ?
            "#,
        )
        .bind(&date_text)
        .fetch_one(&self.pool)
        .await?;

        let event_count: i64 = totals.try_get("event_count")?;
        let total_time_minutes: i64 = totals.try_get("total_time_minutes")?;
        let lines_added: i64 = totals.try_get("lines_added")?;
        let lines_removed: i64 = totals.try_get("lines_removed")?;
        let git_commits: i64 = totals.try_get("git_commits")?;

        let rows = sqlx::query("SELECT project_name, files_modified, languages FROM coding_events WHERE event_date = ?")
            .bind(&date_text)
            .fetch_all(&self.pool)
            .await?;

        let mut projects = Vec::new();
        let mut files_modified = Vec::new();
        let mut languages = BTreeMap::new();

        for row in rows {
            let project_name: Option<String> = row.try_get("project_name")?;
            if let Some(project_name) = project_name.filter(|value| !value.is_empty()) {
                if !projects.contains(&project_name) {
                    projects.push(project_name);
                }
            }

            let files_json: String = row.try_get("files_modified")?;
            let event_files: Vec<String> = serde_json::from_str(&files_json).unwrap_or_default();
            for file in event_files {
                if !files_modified.contains(&file) {
                    files_modified.push(file);
                }
            }

            let languages_json: String = row.try_get("languages")?;
            let event_languages: BTreeMap<String, i64> =
                serde_json::from_str(&languages_json).unwrap_or_default();
            for (language, minutes) in event_languages {
                *languages.entry(language).or_insert(0) += minutes;
            }
        }

        projects.sort();
        files_modified.sort();

        let voice_examples: i64 = sqlx::query("SELECT COUNT(*) AS total FROM voice_examples")
            .fetch_one(&self.pool)
            .await?
            .try_get("total")?;

        Ok(DailySummary {
            date,
            event_count,
            total_time_minutes,
            lines_added,
            lines_removed,
            git_commits,
            projects,
            languages,
            files_modified,
            voice_examples,
        })
    }

    pub async fn insert_voice_example(&self, example: VoiceExampleRequest) -> anyhow::Result<i64> {
        let result =
            sqlx::query("INSERT INTO voice_examples (text, context, created_at) VALUES (?, ?, ?)")
                .bind(example.text)
                .bind(example.context)
                .bind(Utc::now().to_rfc3339())
                .execute(&self.pool)
                .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn recent_voice_examples(&self, limit: i64) -> anyhow::Result<Vec<String>> {
        let rows = sqlx::query("SELECT text FROM voice_examples ORDER BY id DESC LIMIT ?")
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|row| row.try_get("text").map_err(Into::into))
            .collect()
    }

    pub async fn ranked_voice_examples(
        &self,
        query: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<RankedVoiceExample>> {
        let rows = sqlx::query(
            "SELECT id, text, context, created_at FROM voice_examples ORDER BY id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut examples = Vec::new();
        for row in rows {
            let text: String = row.try_get("text")?;
            let created_at_text: String = row.try_get("created_at")?;
            examples.push(RankedVoiceExample {
                id: row.try_get("id")?,
                score: text_similarity(query, &text),
                text,
                context: row.try_get("context")?,
                created_at: DateTime::parse_from_rfc3339(&created_at_text)?.with_timezone(&Utc),
            });
        }
        examples.sort_by(|left, right| right.score.total_cmp(&left.score));
        examples.truncate(limit.clamp(1, 20) as usize);
        Ok(examples)
    }

    pub async fn insert_draft(
        &self,
        date: NaiveDate,
        content: String,
        model: &str,
        context_audit: Value,
        style_score: Option<f64>,
    ) -> anyhow::Result<GeneratedDraft> {
        let created_at = Utc::now();
        let audit = serde_json::to_string(&context_audit)?;
        let result = sqlx::query(
            "INSERT INTO generated_drafts (session_date, content, status, model, context_audit, style_score, created_at) VALUES (?, ?, 'pending_approval', ?, ?, ?, ?)"
        )
        .bind(date.to_string())
        .bind(content)
        .bind(model)
        .bind(audit)
        .bind(style_score)
        .bind(created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        self.get_draft(result.last_insert_rowid()).await
    }

    pub async fn pending_drafts(&self) -> anyhow::Result<Vec<GeneratedDraft>> {
        let rows = sqlx::query(
            "SELECT * FROM generated_drafts WHERE status = 'pending_approval' ORDER BY id DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_draft).collect()
    }

    pub async fn recent_events(&self, limit: i64) -> anyhow::Result<Vec<RecentEvent>> {
        let rows = sqlx::query(
            "SELECT id, timestamp, event_type, project_name, git_branch, files_modified, languages FROM coding_events ORDER BY id DESC LIMIT ?",
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_recent_event).collect()
    }

    pub async fn get_draft(&self, id: i64) -> anyhow::Result<GeneratedDraft> {
        let row = sqlx::query("SELECT * FROM generated_drafts WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        row_to_draft(row)
    }

    pub async fn approve_draft(
        &self,
        id: i64,
        approved_content: Option<String>,
    ) -> anyhow::Result<Option<GeneratedDraft>> {
        let approved_at = Utc::now().to_rfc3339();
        let affected = if let Some(content) = approved_content {
            sqlx::query("UPDATE generated_drafts SET content = ?, status = 'approved', approved_at = ? WHERE id = ?")
                .bind(content)
                .bind(approved_at)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected()
        } else {
            sqlx::query(
                "UPDATE generated_drafts SET status = 'approved', approved_at = ? WHERE id = ?",
            )
            .bind(approved_at)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected()
        };

        if affected == 0 {
            return Ok(None);
        }

        Ok(Some(self.get_draft(id).await?))
    }

    pub async fn reject_draft(
        &self,
        id: i64,
        reason: String,
    ) -> anyhow::Result<Option<GeneratedDraft>> {
        let rejected_at = Utc::now().to_rfc3339();
        let affected = sqlx::query(
            "UPDATE generated_drafts SET status = 'rejected', rejected_at = ?, rejection_reason = ? WHERE id = ?",
        )
        .bind(rejected_at)
        .bind(reason)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Ok(None);
        }

        Ok(Some(self.get_draft(id).await?))
    }
}

fn row_to_draft(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<GeneratedDraft> {
    let date_text: String = row.try_get("session_date")?;
    let created_at_text: String = row.try_get("created_at")?;
    let approved_at_text: Option<String> = row.try_get("approved_at")?;
    let rejected_at_text: Option<String> = row.try_get("rejected_at")?;
    let audit_text: String = row.try_get("context_audit")?;

    Ok(GeneratedDraft {
        id: row.try_get("id")?,
        session_date: NaiveDate::parse_from_str(&date_text, "%Y-%m-%d")?,
        content: row.try_get("content")?,
        status: row.try_get("status")?,
        model: row.try_get("model")?,
        context_audit: serde_json::from_str(&audit_text).unwrap_or_else(|_| json!({})),
        style_score: row.try_get("style_score")?,
        created_at: DateTime::parse_from_rfc3339(&created_at_text)?.with_timezone(&Utc),
        approved_at: approved_at_text
            .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
            .map(|value| value.with_timezone(&Utc)),
        rejected_at: rejected_at_text
            .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
            .map(|value| value.with_timezone(&Utc)),
        rejection_reason: row.try_get("rejection_reason")?,
    })
}

fn row_to_recent_event(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<RecentEvent> {
    let timestamp_text: String = row.try_get("timestamp")?;
    let files_text: String = row.try_get("files_modified")?;
    let languages_text: String = row.try_get("languages")?;

    Ok(RecentEvent {
        id: row.try_get("id")?,
        timestamp: DateTime::parse_from_rfc3339(&timestamp_text)?.with_timezone(&Utc),
        event_type: row.try_get("event_type")?,
        project_name: row.try_get("project_name")?,
        git_branch: row.try_get("git_branch")?,
        files_modified: serde_json::from_str(&files_text).unwrap_or_default(),
        languages: serde_json::from_str(&languages_text).unwrap_or_default(),
    })
}

fn text_similarity(query: &str, text: &str) -> f64 {
    let query_tokens = tokens(query);
    let text_tokens = tokens(text);
    if query_tokens.is_empty() || text_tokens.is_empty() {
        return 0.0;
    }
    let overlap = query_tokens
        .iter()
        .filter(|token| text_tokens.contains(token))
        .count() as f64;
    let denominator = query_tokens.len().max(text_tokens.len()) as f64;
    (overlap / denominator * 100.0).round() / 100.0
}

fn tokens(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| token.len() > 3)
        .collect()
}
