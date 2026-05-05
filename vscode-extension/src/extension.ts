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
    'Aprovar como esta',
    'Copiar para clipboard'
  );

  if (action === 'Aprovar como esta') {
    await backend.approveDraft(draft.id);
    vscode.window.showInformationMessage(`Rascunho #${draft.id} aprovado localmente.`);
  }

  if (action === 'Copiar para clipboard') {
    await vscode.env.clipboard.writeText(draft.content);
    vscode.window.showInformationMessage('Rascunho copiado.');
  }
}

function renderDashboard(dashboard: Awaited<ReturnType<RustBackend['dashboard']>>): string {
  const summary = dashboard.summary;
  const languages = Object.entries(summary.languages).map(([language, minutes]) => `${language}: ${minutes}`).join(', ') || 'nenhuma';
  const recentEvents = dashboard.recent_events
    .map(event => `- ${event.timestamp} | ${event.event_type} | ${(event.files_modified ?? []).join(', ') || 'sem arquivo'}`)
    .join('\n') || '- nenhum evento recente';

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
    '',
    '## Eventos recentes',
    '',
    recentEvents
  ].join('\n');
}
