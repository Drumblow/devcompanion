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
    pub copilot_enabled: bool,
    pub copilot_cli_path: String,
    pub copilot_model: String,
    pub copilot_github_token_env: Option<String>,
    pub linkedin_enabled: bool,
    pub linkedin_dry_run: bool,
    pub linkedin_access_token: Option<String>,
    pub linkedin_author_urn: Option<String>,
    pub linkedin_api_version: String,
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
        let copilot_enabled = env::var("LDC_COPILOT_ENABLED")
            .ok()
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false);
        let copilot_cli_path =
            env::var("LDC_COPILOT_CLI_PATH").unwrap_or_else(|_| "copilot".to_string());
        let copilot_model =
            env::var("LDC_COPILOT_MODEL").unwrap_or_else(|_| "copilot-latest".to_string());
        let copilot_github_token_env = env::var("LDC_COPILOT_GITHUB_TOKEN_ENV")
            .ok()
            .filter(|value| !value.is_empty())
            .or_else(|| Some("COPILOT_GITHUB_TOKEN".to_string()));
        let linkedin_enabled = bool_env("LDC_LINKEDIN_ENABLED", false);
        let linkedin_dry_run = bool_env("LDC_LINKEDIN_DRY_RUN", false);
        let linkedin_access_token = env::var("LDC_LINKEDIN_ACCESS_TOKEN")
            .ok()
            .filter(|value| !value.is_empty());
        let linkedin_author_urn = env::var("LDC_LINKEDIN_AUTHOR_URN")
            .ok()
            .filter(|value| !value.is_empty());
        let linkedin_api_version =
            env::var("LDC_LINKEDIN_API_VERSION").unwrap_or_else(|_| "202506".to_string());

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
            copilot_enabled,
            copilot_cli_path,
            copilot_model,
            copilot_github_token_env,
            linkedin_enabled,
            linkedin_dry_run,
            linkedin_access_token,
            linkedin_author_urn,
            linkedin_api_version,
        })
    }
}

fn bool_env(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}
