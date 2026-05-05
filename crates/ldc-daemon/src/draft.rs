use ldc_core::DailySummary;
use serde_json::{json, Value};

pub fn build_daily_draft(summary: &DailySummary, voice_examples: &[String]) -> (String, Value) {
    let project = summary
        .projects
        .first()
        .cloned()
        .unwrap_or_else(|| "um projeto local".to_string());
    let language_list = if summary.languages.is_empty() {
        "a stack principal ainda nao ficou clara".to_string()
    } else {
        summary
            .languages
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    };
    let files = summary
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
    let voice_line = if voice_examples.is_empty() {
        "Ainda sem exemplos de voz aprovados, entao este rascunho esta mais neutro.".to_string()
    } else {
        "Usei os exemplos de voz aprovados como referencia de tom, mantendo o conteudo tecnico preso ao contexto do dia.".to_string()
    };

    let content = format!(
        "Hoje avancei em {project} e o sinal mais interessante foi o conjunto de pequenas decisoes tecnicas ao redor de {language_list}.\n\nForam {event_count} eventos registrados, {lines_added} linhas adicionadas e {lines_removed} removidas. O recorte passou por: {file_line}.\n\nO aprendizado do dia: transformar trabalho real em conteudo fica mais facil quando o sistema registra contexto suficiente para lembrar o que mudou, mas ainda deixa a decisao final na mao de quem escreveu o codigo.\n\n{voice_line}\n\nRascunho pendente de revisao manual antes de qualquer publicacao.",
        event_count = summary.event_count,
        lines_added = summary.lines_added,
        lines_removed = summary.lines_removed,
    );

    let audit = json!({
        "source": "daily_summary",
        "summary": summary,
        "voice_examples_used": voice_examples.len(),
        "human_approval_required": true,
        "publication": "disabled_in_phase_1"
    });

    (content, audit)
}
