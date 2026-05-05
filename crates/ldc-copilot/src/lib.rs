use anyhow::Context;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct CopilotAdapter {
    cli_path: String,
    model: String,
    github_token_env: Option<String>,
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
        }
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

        let output = command
            .output()
            .await
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
