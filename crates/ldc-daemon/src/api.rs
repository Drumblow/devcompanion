use crate::{draft::build_daily_draft, error::ApiError, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use ldc_core::{
    ApproveDraftRequest, CodingEvent, GenerateDraftRequest, HealthResponse, VoiceExampleRequest,
};
use serde_json::json;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/events", post(receive_event))
        .route("/sessions/{date}/summary", get(summary))
        .route("/posts/generate", post(generate_post))
        .route("/posts/pending", get(pending_posts))
        .route("/posts/{id}/approve", post(approve_post))
        .route("/personality/examples", post(add_voice_example))
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    state
        .db
        .daily_summary(Utc::now().date_naive())
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        service: "linkedin-dev-companion".to_string(),
        database: "ok".to_string(),
    }))
}

async fn receive_event(
    State(state): State<AppState>,
    Json(event): Json<CodingEvent>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if event.session_id.trim().is_empty() {
        return Err(ApiError::bad_request("session_id e obrigatorio"));
    }
    if event.event_type.trim().is_empty() {
        return Err(ApiError::bad_request("event_type e obrigatorio"));
    }

    let stored = state
        .db
        .insert_event(event)
        .await
        .map_err(ApiError::internal)?;
    Ok((StatusCode::CREATED, Json(json!(stored))))
}

async fn summary(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| ApiError::bad_request("data deve estar no formato YYYY-MM-DD"))?;
    let summary = state
        .db
        .daily_summary(date)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(json!(summary)))
}

async fn generate_post(
    State(state): State<AppState>,
    Json(request): Json<GenerateDraftRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let date = request.date.unwrap_or_else(|| Utc::now().date_naive());
    let summary = state
        .db
        .daily_summary(date)
        .await
        .map_err(ApiError::internal)?;
    let examples = state
        .db
        .recent_voice_examples(3)
        .await
        .map_err(ApiError::internal)?;
    let (content, audit) = build_daily_draft(&summary, &examples);
    let draft = state
        .db
        .insert_draft(date, content, &state.draft_model, audit)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(json!(draft)))
}

async fn pending_posts(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let drafts = state
        .db
        .pending_drafts()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(json!(drafts)))
}

async fn approve_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<ApproveDraftRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let draft = state
        .db
        .approve_draft(id, request.approved_content)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(|| ApiError::not_found("rascunho nao encontrado"))?;
    Ok(Json(json!(draft)))
}

async fn add_voice_example(
    State(state): State<AppState>,
    Json(request): Json<VoiceExampleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if request.text.trim().len() < 20 {
        return Err(ApiError::bad_request(
            "exemplo de voz deve ter pelo menos 20 caracteres",
        ));
    }
    let id = state
        .db
        .insert_voice_example(request)
        .await
        .map_err(ApiError::internal)?;
    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}
