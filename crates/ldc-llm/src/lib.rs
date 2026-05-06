use async_trait::async_trait;
use ldc_core::{DailySummary, TechnicalAnalysis};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct DraftInput {
    pub summary: DailySummary,
    pub voice_examples: Vec<String>,
    pub technical_analysis: Option<TechnicalAnalysis>,
}

#[derive(Debug, Clone)]
pub struct DraftOutput {
    pub content: String,
    pub model: String,
    pub style_score: f64,
    pub audit: Value,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate_draft(&self, input: DraftInput) -> anyhow::Result<DraftOutput>;
}

#[derive(Debug, Clone)]
pub struct TemplateProvider;

#[async_trait]
impl LlmProvider for TemplateProvider {
    async fn generate_draft(&self, input: DraftInput) -> anyhow::Result<DraftOutput> {
        let project = input
            .summary
            .projects
            .first()
            .cloned()
            .unwrap_or_else(|| "um projeto local".to_string());
        let language_list = if input.summary.languages.is_empty() {
            "a stack principal ainda nao ficou clara".to_string()
        } else {
            input
                .summary
                .languages
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        };
        let files = input
            .summary
            .files_modified
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>();
        let file_line = if files.is_empty() {
            "sem arquivos relevantes registrados ainda".to_string()
        } else {
            files.join(", ")
        };
        let voice_line = if input.voice_examples.is_empty() {
            "Ainda sem exemplos de voz aprovados, entao este rascunho esta mais neutro.".to_string()
        } else {
            "Usei exemplos de voz aprovados como referencia de tom, mantendo o conteudo tecnico preso ao contexto do dia.".to_string()
        };
        let analysis_line = input
            .technical_analysis
            .as_ref()
            .and_then(|analysis| analysis.insights.first())
            .map(|insight| format!("\n\nAnalise tecnica do dia: {insight}"))
            .unwrap_or_default();

        let content = format!(
            "Hoje avancei em {project} e o sinal mais interessante foi o conjunto de decisoes tecnicas ao redor de {language_list}.\n\nForam {event_count} eventos registrados, {lines_added} linhas adicionadas e {lines_removed} removidas. O recorte passou por: {file_line}.{analysis_line}\n\nO aprendizado do dia: transformar trabalho real em conteudo fica mais facil quando o sistema registra contexto suficiente para lembrar o que mudou, mas ainda deixa a decisao final na mao de quem escreveu o codigo.\n\n{voice_line}\n\nRascunho pendente de revisao manual antes de qualquer publicacao.",
            event_count = input.summary.event_count,
            lines_added = input.summary.lines_added,
            lines_removed = input.summary.lines_removed,
        );

        let style_score = score_style(&content, &input.voice_examples);
        Ok(DraftOutput {
            content,
            model: "local-template-v2".to_string(),
            style_score,
            audit: json!({
                "provider": "template",
                "summary": input.summary,
                "technical_analysis": input.technical_analysis,
                "voice_examples_used": input.voice_examples.len(),
                "human_approval_required": true,
                "publication": "disabled"
            }),
        })
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    reasoning_effort: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String, reasoning_effort: String) -> Self {
        Self {
            api_key,
            model,
            reasoning_effort,
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate_draft(&self, input: DraftInput) -> anyhow::Result<DraftOutput> {
        let prompt = format!(
            "Gere um rascunho de LinkedIn em portugues com base neste resumo real do dia: {}",
            serde_json::to_string(&input.summary)?
        );
        let instructions = format!(
            "Use somente o contexto fornecido. Aprovacao humana e obrigatoria. Analise tecnica opcional: {}\n\nExemplos de voz aprovados: {}",
            serde_json::to_string(&input.technical_analysis)?,
            input.voice_examples.join("\n---\n")
        );
        let body = json!({
            "model": self.model,
            "reasoning": { "effort": self.reasoning_effort },
            "instructions": instructions,
            "input": prompt,
            "max_output_tokens": 800
        });

        let response = reqwest::Client::new()
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("OpenAI Responses API retornou {}", response.status());
        }

        let payload: Value = response.json().await?;
        let content = payload["output"][0]["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("estrutura inesperada da Responses API"))?
            .to_string();
        let style_score = score_style(&content, &input.voice_examples);

        Ok(DraftOutput {
            content,
            model: self.model.clone(),
            style_score,
            audit: json!({
                "provider": "openai_responses",
                "model": self.model,
                "reasoning_effort": self.reasoning_effort,
                "summary": input.summary,
                "technical_analysis": input.technical_analysis,
                "voice_examples_used": input.voice_examples.len(),
                "human_approval_required": true,
                "publication": "disabled"
            }),
        })
    }
}

pub fn score_style(content: &str, examples: &[String]) -> f64 {
    if examples.is_empty() {
        return 0.0;
    }
    let content_tokens = tokens(content);
    if content_tokens.is_empty() {
        return 0.0;
    }
    let mut best = 0.0_f64;
    for example in examples {
        let example_tokens = tokens(example);
        if example_tokens.is_empty() {
            continue;
        }
        let overlap = content_tokens
            .iter()
            .filter(|token| example_tokens.contains(token))
            .count() as f64;
        let denominator = content_tokens.len().max(example_tokens.len()) as f64;
        best = best.max(overlap / denominator);
    }
    (best * 100.0).round() / 100.0
}

fn tokens(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| token.len() > 3)
        .collect()
}
