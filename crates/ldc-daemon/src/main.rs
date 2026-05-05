mod api;
mod config;
mod db;
mod error;

use anyhow::Context;
use axum::Router;
use config::AppConfig;
use db::Database;
use ldc_llm::{LlmProvider, OpenAiProvider, TemplateProvider};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub llm_provider: Arc<dyn LlmProvider>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ldc_daemon=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env()?;
    let database = Database::connect(&config.database_path).await?;
    database.migrate().await?;

    let state = AppState {
        db: database,
        llm_provider: build_provider(&config),
    };

    let app = app(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = TcpListener::bind(addr)
        .await
        .context("falha ao iniciar listener HTTP")?;

    info!(%addr, "ldc-daemon iniciado");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .merge(api::routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn build_provider(config: &AppConfig) -> Arc<dyn LlmProvider> {
    if config.provider.eq_ignore_ascii_case("openai") {
        if let Some(api_key) = &config.openai_api_key {
            return Arc::new(OpenAiProvider::new(
                api_key.clone(),
                config.draft_model.clone(),
                config.reasoning_effort.clone(),
            ));
        }
    }

    Arc::new(TemplateProvider)
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use ldc_llm::TemplateProvider;
    use serde_json::{json, Value};
    use tower::ServiceExt;

    async fn test_app() -> Router {
        let db_path = std::env::temp_dir().join(format!(
            "ldc-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let database = Database::connect(&db_path).await.unwrap();
        database.migrate().await.unwrap();
        app(AppState {
            db: database,
            llm_provider: Arc::new(TemplateProvider),
        })
    }

    #[tokio::test]
    async fn http_flow_generates_approves_and_rejects_drafts() {
        let app = test_app().await;
        let event = json!({
            "session_id": "integration",
            "event_type": "DOCUMENT_EDIT",
            "project": { "name": "demo", "path": "demo" },
            "activity": {
                "files_modified": ["src/main.rs", ".env"],
                "languages": { "Rust": 10 },
                "lines_added": 5,
                "lines_removed": 1,
                "time_spent_minutes": 10
            },
            "metadata": { "token": "redact-me", "source": "test" }
        });

        let response = app
            .clone()
            .oneshot(json_request("POST", "/events", event))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(json_request("POST", "/posts/generate", json!({})))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        let draft_id = body["id"].as_i64().unwrap();
        assert_eq!(body["status"], "pending_approval");
        assert!(body["style_score"].is_number());

        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                &format!("/posts/{draft_id}/approve"),
                json!({ "approved_content": "texto editado e aprovado" }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert_eq!(body["status"], "approved");
        assert_eq!(body["content"], "texto editado e aprovado");

        let response = app
            .clone()
            .oneshot(json_request("POST", "/posts/generate", json!({})))
            .await
            .unwrap();
        let draft_id = body_json(response).await["id"].as_i64().unwrap();
        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                &format!("/posts/{draft_id}/reject"),
                json!({ "reason": "muito generico" }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_json(response).await;
        assert_eq!(body["status"], "rejected");
        assert_eq!(body["rejection_reason"], "muito generico");
    }

    fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn body_json(response: axum::response::Response) -> Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }
}
