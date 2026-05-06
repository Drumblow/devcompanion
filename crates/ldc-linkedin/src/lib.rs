use chrono::Utc;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct LinkedInPublisher {
    enabled: bool,
    dry_run: bool,
    access_token: Option<String>,
    author_urn: Option<String>,
    api_version: String,
}

#[derive(Debug, Clone)]
pub struct LinkedInPublishResult {
    pub provider: String,
    pub external_id: String,
}

impl LinkedInPublisher {
    pub fn new(
        enabled: bool,
        dry_run: bool,
        access_token: Option<String>,
        author_urn: Option<String>,
        api_version: String,
    ) -> Self {
        Self {
            enabled,
            dry_run,
            access_token,
            author_urn,
            api_version,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    pub async fn publish(&self, content: &str) -> anyhow::Result<LinkedInPublishResult> {
        if !self.enabled {
            anyhow::bail!("LinkedIn publisher desabilitado. Use LDC_LINKEDIN_ENABLED=true.");
        }

        let author_urn = self
            .author_urn
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("LDC_LINKEDIN_AUTHOR_URN nao configurado"))?;

        if self.dry_run {
            return Ok(LinkedInPublishResult {
                provider: "linkedin_dry_run".to_string(),
                external_id: format!("dryrun-{}", Utc::now().timestamp_millis()),
            });
        }

        let access_token = self
            .access_token
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("LDC_LINKEDIN_ACCESS_TOKEN nao configurado"))?;

        let body = json!({
            "author": author_urn,
            "commentary": content,
            "visibility": "PUBLIC",
            "distribution": {
                "feedDistribution": "MAIN_FEED",
                "targetEntities": [],
                "thirdPartyDistributionChannels": []
            },
            "lifecycleState": "PUBLISHED",
            "isReshareDisabledByAuthor": false
        });

        let response = reqwest::Client::new()
            .post("https://api.linkedin.com/rest/posts")
            .bearer_auth(access_token)
            .header("Linkedin-Version", &self.api_version)
            .header("X-Restli-Protocol-Version", "2.0.0")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();
        let payload: Value = response.json().await.unwrap_or_else(|_| json!({}));

        if !status.is_success() {
            anyhow::bail!("LinkedIn Posts API retornou {status}: {payload}");
        }

        let external_id = restli_id(&headers)
            .or_else(|| {
                payload
                    .get("id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .unwrap_or_else(|| format!("linkedin-{}", Utc::now().timestamp_millis()));

        Ok(LinkedInPublishResult {
            provider: "linkedin_posts_api".to_string(),
            external_id,
        })
    }
}

fn restli_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-restli-id")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}
