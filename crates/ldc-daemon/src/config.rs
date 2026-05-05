use anyhow::Context;
use std::{env, fs, path::PathBuf};

pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_path: PathBuf,
    pub draft_model: String,
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

        if let Some(parent) = database_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("falha ao criar diretorio {}", parent.display()))?;
        }

        Ok(Self {
            host,
            port,
            database_path,
            draft_model,
        })
    }
}
