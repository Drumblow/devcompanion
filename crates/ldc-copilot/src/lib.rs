use anyhow::Context;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
pub struct CopilotAdapter {
    cli_path: String,
    model: String,
    github_token_env: Option<String>,
    timeout_seconds: u64,
}

impl CopilotAdapter {
    pub fn new(
        cli_path: impl Into<String>,
        model: impl Into<String>,
        github_token_env: Option<String>,
    ) -> Self {
        Self {
            cli_path: cli_path.into(),
            model: model.into(),
            github_token_env,
            timeout_seconds: 45,
        }
    }

    pub fn cli_path(&self) -> &str {
        &self.cli_path
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub async fn is_available(&self) -> bool {
        Command::new(&self.cli_path)
            .arg("--help")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub async fn analyze_daily_context(&self, prompt: &str) -> anyhow::Result<String> {
        let mut command = Command::new(&self.cli_path);
        command
            .args(["-p", prompt])
            .args(["-s", "--no-ask-user"])
            .args(["--model", &self.model])
            .args(["--allow-tool", "shell(git:*)"]);

        if let Some(env_name) = &self.github_token_env {
            if let Ok(token) = std::env::var(env_name) {
                command.env("COPILOT_GITHUB_TOKEN", token);
            }
        }

        let output = timeout(Duration::from_secs(self.timeout_seconds), command.output())
            .await
            .with_context(|| format!("timeout ao executar Copilot CLI em {}", self.cli_path))?
            .with_context(|| format!("falha ao executar Copilot CLI em {}", self.cli_path))?;

        if !output.status.success() {
            anyhow::bail!(
                "Copilot CLI retornou erro: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        String::from_utf8(output.stdout).context("saida do Copilot CLI nao e UTF-8 valida")
    }
}
