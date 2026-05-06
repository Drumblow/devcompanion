import * as vscode from 'vscode';
import { RustBackend, Draft } from './api/rustBackend';
import { EventCollector } from './tracker/eventCollector';
import { GitWatcher } from './tracker/gitWatcher';
import { SessionManager } from './tracker/sessionManager';

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const backend = new RustBackend();
  const session = new SessionManager(context);
  const collector = new EventCollector(backend, session);
  const gitWatcher = new GitWatcher(backend, session);
  const status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);

  status.text = 'LDC: conectando';
  status.command = 'linkedinDevCompanion.generateDraft';
  status.show();

  collector.start();
  gitWatcher.start();
  context.subscriptions.push(collector, gitWatcher, status);

  const isHealthy = await backend.health();
  status.text = isHealthy ? 'LDC: ativo' : 'LDC: daemon offline';
  if (!isHealthy) {
    void showDaemonOfflineMessage();
  }

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.generateDraft', async () => {
    try {
      const draft = await backend.generateDraft();
      await showDraftReview(backend, draft);
      status.text = 'LDC: rascunho gerado';
    } catch (error) {
      vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao gerar rascunho');
    }
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.showPendingDrafts', async () => {
    try {
      const drafts = await backend.pendingDrafts();
      if (drafts.length === 0) {
        vscode.window.showInformationMessage('Nenhum rascunho pendente.');
        return;
      }
      const selected = await vscode.window.showQuickPick(
        drafts.map(draft => ({ label: `#${draft.id} - ${draft.session_date}`, detail: draft.content.slice(0, 120), draft })),
        { placeHolder: 'Escolha um rascunho para revisar' }
      );
      if (selected) {
        await showDraftReview(backend, selected.draft);
      }
    } catch (error) {
      vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao buscar rascunhos');
    }
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.saveVoiceExample', async () => {
    const editor = vscode.window.activeTextEditor;
    const selectedText = editor?.document.getText(editor.selection).trim();
    const text = selectedText || await vscode.window.showInputBox({ prompt: 'Texto de autoria propria para usar como exemplo de voz' });
    if (!text) {
      return;
    }

    try {
      await backend.saveVoiceExample(text, 'manual_selection');
      vscode.window.showInformationMessage('Exemplo de voz salvo localmente.');
    } catch (error) {
      vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao salvar exemplo de voz');
    }
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.saveClipboardAsVoiceExample', async () => {
    const text = (await vscode.env.clipboard.readText()).trim();
    if (!text) {
      vscode.window.showWarningMessage('Clipboard vazio. Copie o texto do prompt/chat antes de salvar como exemplo de voz.');
      return;
    }

    const action = await vscode.window.showInformationMessage(
      `Salvar ${text.length} caracteres do clipboard como exemplo de voz?`,
      'Salvar'
    );
    if (action !== 'Salvar') {
      return;
    }

    try {
      await backend.saveVoiceExample(text, 'clipboard_prompt');
      vscode.window.showInformationMessage('Prompt do clipboard salvo como exemplo de voz.');
    } catch (error) {
      vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao salvar clipboard como exemplo de voz');
    }
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.showDashboard', async () => {
    try {
      const dashboard = await backend.dashboard();
      const document = await vscode.workspace.openTextDocument({
        language: 'markdown',
        content: renderDashboard(dashboard)
      });
      await vscode.window.showTextDocument(document, { preview: false });
    } catch (error) {
      vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao abrir dashboard');
    }
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.checkDaemon', async () => {
    const healthy = await backend.health();
    status.text = healthy ? 'LDC: ativo' : 'LDC: daemon offline';
    if (healthy) {
      vscode.window.showInformationMessage('LinkedIn Dev Companion: daemon local ativo em http://127.0.0.1:8787.');
      return;
    }
    await showDaemonOfflineMessage();
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.checkCopilot', async () => {
    try {
      const status = await backend.copilotStatus();
      vscode.window.showInformationMessage(`Copilot CLI: ${status.message}`);
    } catch (error) {
      vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao verificar Copilot CLI');
    }
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.showTodayAnalysis', async () => {
    try {
      const analysis = await backend.todayAnalysis();
      const document = await vscode.workspace.openTextDocument({
        language: 'markdown',
        content: renderTechnicalAnalysis(analysis)
      });
      await vscode.window.showTextDocument(document, { preview: false });
    } catch (error) {
      vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao abrir analise tecnica');
    }
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.checkPublisher', async () => {
    try {
      const publisher = await backend.publisherStatus();
      vscode.window.showInformationMessage(`LinkedIn Publisher: ${publisher.message}`);
    } catch (error) {
      vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao verificar publisher LinkedIn');
    }
  }));

  context.subscriptions.push(vscode.commands.registerCommand('linkedinDevCompanion.publishApprovedDraft', async () => {
    const idText = await vscode.window.showInputBox({ prompt: 'ID do rascunho aprovado para publicar' });
    const draftId = Number(idText);
    if (!Number.isInteger(draftId) || draftId <= 0) {
      return;
    }

    await publishDraft(backend, draftId);
  }));
}

export function deactivate(): void {}

async function showDraftReview(backend: RustBackend, draft: Draft): Promise<void> {
  const document = await vscode.workspace.openTextDocument({
    language: 'markdown',
    content: draft.content
  });
  await vscode.window.showTextDocument(document, { preview: false });

  const action = await vscode.window.showInformationMessage(
    `Rascunho #${draft.id} pendente de aprovacao manual`,
    'Aprovar texto aberto',
    'Rejeitar',
    'Copiar para clipboard'
  );

  if (action === 'Aprovar texto aberto') {
    const approved = await backend.approveDraft(draft.id, document.getText());
    vscode.window.showInformationMessage(`Rascunho #${draft.id} aprovado localmente.`);
    const publishAction = await vscode.window.showInformationMessage(
      `Rascunho #${approved.id} aprovado. Publicar agora?`,
      'Publicar'
    );
    if (publishAction === 'Publicar') {
      await publishDraft(backend, approved.id);
    }
  }

  if (action === 'Rejeitar') {
    const reason = await vscode.window.showInputBox({ prompt: 'Motivo da rejeicao para melhorar o proximo rascunho' });
    if (reason) {
      await backend.rejectDraft(draft.id, reason);
      vscode.window.showInformationMessage(`Rascunho #${draft.id} rejeitado localmente.`);
    }
  }

  if (action === 'Copiar para clipboard') {
    await vscode.env.clipboard.writeText(document.getText());
    vscode.window.showInformationMessage('Rascunho copiado.');
  }
}

async function publishDraft(backend: RustBackend, draftId: number): Promise<void> {
  try {
    const published = await backend.publishDraft(draftId);
    vscode.window.showInformationMessage(
      `Rascunho #${published.draft.id} publicado via ${published.provider}: ${published.draft.linkedin_post_id ?? 'sem id externo'}.`
    );
  } catch (error) {
    vscode.window.showErrorMessage(error instanceof Error ? error.message : 'Falha ao publicar rascunho');
  }
}

function renderDashboard(dashboard: Awaited<ReturnType<RustBackend['dashboard']>>): string {
  const summary = dashboard.summary;
  const languages = Object.entries(summary.languages).map(([language, minutes]) => `${language}: ${minutes}`).join(', ') || 'nenhuma';
  const recentEvents = dashboard.recent_events
    .map(event => `- ${event.timestamp} | ${event.event_type} | ${(event.files_modified ?? []).join(', ') || 'sem arquivo'}`)
    .join('\n') || '- nenhum evento recente';
  const gitChanges = summary.git_changes
    .map(change => {
      const headline = change.subject ?? change.diff_summary ?? change.status_summary ?? change.event_type;
      const files = (change.files_modified ?? []).slice(0, 5).join(', ') || 'sem arquivo';
      return `- ${change.event_type} | ${headline} | +${change.lines_added} / -${change.lines_removed} | ${files}`;
    })
    .join('\n') || '- nenhum sinal Git capturado';

  return [
    '# LinkedIn Dev Companion',
    '',
    `Data: ${summary.date}`,
    `Eventos: ${summary.event_count}`,
    `Tempo registrado: ${summary.total_time_minutes} min`,
    `Linhas: +${summary.lines_added} / -${summary.lines_removed}`,
    `Commits: ${summary.git_commits}`,
    `Projetos: ${summary.projects.join(', ') || 'nenhum'}`,
    `Linguagens: ${languages}`,
    `Rascunhos pendentes: ${dashboard.pending_drafts.length}`,
    `Score de estilo dos pendentes: ${dashboard.pending_drafts.map(draft => draft.style_score ?? 0).join(', ') || 'nenhum'}`,
    '',
    '## Sinais Git',
    '',
    gitChanges,
    '',
    '## Eventos recentes',
    '',
    recentEvents
  ].join('\n');
}

function renderTechnicalAnalysis(analysis: Awaited<ReturnType<RustBackend['todayAnalysis']>>): string {
  return [
    '# Analise tecnica de hoje',
    '',
    `Origem: ${analysis.source}`,
    `Status: ${analysis.status}`,
    `Complexidade: ${analysis.complexity ?? 'n/a'}`,
    '',
    '## Insights',
    '',
    ...(analysis.insights.length ? analysis.insights.map(item => `- ${item}`) : ['- nenhum insight registrado']),
    '',
    '## Stack',
    '',
    ...(analysis.tech_stack.length ? analysis.tech_stack.map(item => `- ${item}`) : ['- nenhuma stack registrada']),
    '',
    '## Aprendizados',
    '',
    ...(analysis.learnings.length ? analysis.learnings.map(item => `- ${item}`) : ['- nenhum aprendizado registrado']),
    analysis.error ? `\nErro/fallback: ${analysis.error}` : ''
  ].join('\n');
}

async function showDaemonOfflineMessage(): Promise<void> {
  const action = await vscode.window.showWarningMessage(
    'LinkedIn Dev Companion: daemon local offline. Inicie o backend antes de gerar rascunhos.',
    'Copiar comando'
  );

  if (action === 'Copiar comando') {
    await vscode.env.clipboard.writeText('cargo run -p ldc-daemon');
    vscode.window.showInformationMessage('Comando copiado. Execute na raiz do projeto Automatatizador.');
  }
}
