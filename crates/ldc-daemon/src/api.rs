use crate::{analysis::analyze_day, error::ApiError, AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{NaiveDate, Utc};
use ldc_core::{
    ApproveDraftRequest, CodingEvent, CopilotStatus, DashboardSnapshot, GenerateDraftRequest,
    HealthResponse, RejectDraftRequest, VoiceExampleRequest,
};
use ldc_ingestor::normalize_event;
use ldc_llm::DraftInput;
use serde_json::json;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/copilot/status", get(copilot_status))
        .route("/analysis/today", get(today_analysis))
        .route("/events", post(receive_event))
        .route("/events/recent", get(recent_events))
        .route("/dashboard/today", get(today_dashboard))
        .route("/sessions/{date}/summary", get(summary))
        .route("/posts/generate", post(generate_post))
        .route("/posts/pending", get(pending_posts))
        .route("/posts/{id}/approve", post(approve_post))
        .route("/posts/{id}/reject", post(reject_post))
        .route("/personality/examples", post(add_voice_example))
        .route("/personality/examples/ranked", post(ranked_voice_examples))
}

async fn copilot_status(State(state): State<AppState>) -> Result<Json<CopilotStatus>, ApiError> {
    let Some(copilot) = &state.copilot else {
        return Ok(Json(CopilotStatus {
            enabled: false,
            available: false,
            cli_path: "copilot".to_string(),
            model: "copilot-latest".to_string(),
            message: "Copilot CLI desabilitado. Use LDC_COPILOT_ENABLED=true para ativar."
                .to_string(),
        }));
    };

    let available = copilot.is_available().await;
    Ok(Json(CopilotStatus {
        enabled: true,
        available,
        cli_path: copilot.cli_path().to_string(),
        model: copilot.model().to_string(),
        message: if available {
            "Copilot CLI disponivel para analise tecnica.".to_string()
        } else {
            "Copilot CLI habilitado, mas nao encontrado ou nao respondeu ao --help.".to_string()
        },
    }))
}

async fn today_analysis(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let summary = state
        .db
        .daily_summary(Utc::now().date_naive())
        .await
        .map_err(ApiError::internal)?;
    let analysis = analyze_day(&summary, state.copilot.as_ref()).await;
    Ok(Json(json!(analysis)))
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

    let event = normalize_event(event).map_err(|error| ApiError::bad_request(error.to_string()))?;
    let event_date = event.timestamp.unwrap_or_else(Utc::now).date_naive();
    let stored = state
        .db
        .insert_event(event)
        .await
        .map_err(ApiError::internal)?;
    state
        .db
        .refresh_daily_session(event_date)
        .await
        .map_err(ApiError::internal)?;
    Ok((StatusCode::CREATED, Json(json!(stored))))
}

async fn recent_events(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let events = state
        .db
        .recent_events(25)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(json!(events)))
}

async fn today_dashboard(
    State(state): State<AppState>,
) -> Result<Json<DashboardSnapshot>, ApiError> {
    let date = Utc::now().date_naive();
    let summary = state
        .db
        .daily_summary(date)
        .await
        .map_err(ApiError::internal)?;
    let recent_events = state
        .db
        .recent_events(10)
        .await
        .map_err(ApiError::internal)?;
    let pending_drafts = state
        .db
        .pending_drafts()
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(DashboardSnapshot {
        summary,
        recent_events,
        pending_drafts,
    }))
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
    let technical_analysis = analyze_day(&summary, state.copilot.as_ref()).await;
    let output = state
        .llm_provider
        .generate_draft(DraftInput {
            summary,
            voice_examples: examples,
            technical_analysis: Some(technical_analysis),
        })
        .await
        .map_err(ApiError::internal)?;
    let draft = state
        .db
        .insert_draft(
            date,
            output.content,
            &output.model,
            output.audit,
            Some(output.style_score),
        )
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

async fn reject_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<RejectDraftRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if request.reason.trim().len() < 3 {
        return Err(ApiError::bad_request(
            "motivo de rejeicao deve ter pelo menos 3 caracteres",
        ));
    }

    let draft = state
        .db
        .reject_draft(id, request.reason)
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

async fn ranked_voice_examples(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let query = request["query"].as_str().unwrap_or_default();
    if query.trim().is_empty() {
        return Err(ApiError::bad_request("query e obrigatoria"));
    }

    let examples = state
        .db
        .ranked_voice_examples(query, 5)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(json!(examples)))
}
