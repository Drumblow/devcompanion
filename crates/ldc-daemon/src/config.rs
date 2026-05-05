use anyhow::Context;
use std::{env, fs, path::PathBuf};

pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_path: PathBuf,
    pub draft_model: String,
    pub provider: String,
    pub openai_api_key: Option<String>,
    pub reasoning_effort: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let host = env::var("LDC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("LDC_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8787);
        let draft_model =
            env::var("LDC_DRAFT_MODEL").unwrap_or_else(|_| "local-template-v1".to_string());
        let database_path = env::var("LDC_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".ldc/linkedin-dev-companion.db"));
        let provider = env::var("LDC_LLM_PROVIDER").unwrap_or_else(|_| "template".to_string());
        let openai_api_key = env::var("OPENAI_API_KEY")
            .ok()
            .filter(|value| !value.is_empty());
        let reasoning_effort =
            env::var("LDC_REASONING_EFFORT").unwrap_or_else(|_| "medium".to_string());

        if let Some(parent) = database_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("falha ao criar diretorio {}", parent.display()))?;
        }

        Ok(Self {
            host,
            port,
            database_path,
            draft_model,
            provider,
            openai_api_key,
            reasoning_effort,
        })
    }
}
