use ldc_copilot::CopilotAdapter;
use ldc_core::{DailySummary, TechnicalAnalysis};
use serde_json::Value;

pub async fn analyze_day(
    summary: &DailySummary,
    copilot: Option<&CopilotAdapter>,
) -> TechnicalAnalysis {
    let Some(copilot) = copilot else {
        return local_analysis(summary, "local_fallback", None);
    };

    let prompt = build_prompt(summary);
    match copilot.analyze_daily_context(&prompt).await {
        Ok(raw) => parse_copilot_analysis(raw).unwrap_or_else(|error| {
            let mut analysis =
                local_analysis(summary, "copilot_parse_fallback", Some(error.to_string()));
            analysis.raw = Some(prompt);
            analysis
        }),
        Err(error) => local_analysis(summary, "copilot_error_fallback", Some(error.to_string())),
    }
}

pub fn local_analysis(
    summary: &DailySummary,
    source: &str,
    error: Option<String>,
) -> TechnicalAnalysis {
    let project = summary
        .projects
        .first()
        .cloned()
        .unwrap_or_else(|| "projeto local".to_string());
    let tech_stack = summary.languages.keys().cloned().collect::<Vec<_>>();
    let first_file = summary
        .files_modified
        .first()
        .cloned()
        .unwrap_or_else(|| "nenhum arquivo especifico".to_string());
    let git_line = summary
        .git_changes
        .first()
        .map(|change| {
            let files = if change.files_modified.is_empty() {
                "sem arquivos listados".to_string()
            } else {
                change
                    .files_modified
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let change_summary = change
                .diff_summary
                .clone()
                .or_else(|| change.status_summary.clone())
                .or_else(|| change.subject.clone())
                .unwrap_or_else(|| "sem resumo textual".to_string());
            format!(
                "Ultimo sinal Git: {} em {} (+{} / -{}).",
                change_summary, files, change.lines_added, change.lines_removed
            )
        })
        .unwrap_or_else(|| "Nenhum diff ou commit recente foi capturado ainda.".to_string());

    TechnicalAnalysis {
        source: source.to_string(),
        status: if error.is_some() { "fallback" } else { "ok" }.to_string(),
        insights: vec![format!(
            "O dia em {project} teve {events} eventos, {commits} commit(s) e foco inicial em {first_file}. {git_line}",
            events = summary.event_count,
            commits = summary.git_commits,
        )],
        tech_stack,
        complexity: Some(estimate_complexity(summary)),
        learnings: vec!["Resumo tecnico gerado localmente a partir dos sinais capturados.".to_string()],
        raw: None,
        error,
    }
}

fn build_prompt(summary: &DailySummary) -> String {
    format!(
        "Voce e um tech lead analisando o dia de desenvolvimento para gerar contexto real de um post LinkedIn. Use os campos git_changes como evidencias principais de diff, status e commits quando existirem. Retorne SOMENTE JSON valido com os campos insights (array de strings), tech_stack (array de strings), complexity (0-10), learnings (array de strings). Resumo do dia: {}",
        serde_json::to_string(summary).unwrap_or_else(|_| "{}".to_string())
    )
}

fn parse_copilot_analysis(raw: String) -> anyhow::Result<TechnicalAnalysis> {
    let value: Value = serde_json::from_str(raw.trim())?;
    Ok(TechnicalAnalysis {
        source: "copilot_cli".to_string(),
        status: "ok".to_string(),
        insights: string_array(&value["insights"]),
        tech_stack: string_array(&value["tech_stack"]),
        complexity: value["complexity"].as_i64(),
        learnings: string_array(&value["learnings"]),
        raw: Some(raw),
        error: None,
    })
}

fn string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn estimate_complexity(summary: &DailySummary) -> i64 {
    let mut score = 1;
    score += (summary.files_modified.len() as i64 / 3).min(3);
    score += (summary.git_commits).min(2);
    score += (summary.git_changes.len() as i64).min(2);
    score += ((summary.lines_added + summary.lines_removed) / 100).min(4);
    score.clamp(1, 10)
}
