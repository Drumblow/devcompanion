import * as vscode from 'vscode';
import { RustBackend } from '../api/rustBackend';
import { eventDebounceMs } from '../config';
import { SessionManager } from './sessionManager';

export class EventCollector implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private readonly lastSentByDocument = new Map<string, number>();

  constructor(private readonly backend: RustBackend, private readonly session: SessionManager) {}

  start(): void {
    this.disposables.push(vscode.workspace.onDidOpenTextDocument(document => this.sendDocumentEvent('document_open', document)));
    this.disposables.push(vscode.workspace.onDidChangeTextDocument(event => this.sendEditEvent(event)));
    this.disposables.push(vscode.window.onDidChangeActiveTextEditor(editor => {
      if (editor) {
        void this.sendDocumentEvent('active_editor', editor.document);
      }
    }));

    const heartbeat = setInterval(() => {
      void this.sendHeartbeat();
    }, 60000);
    this.disposables.push({ dispose: () => clearInterval(heartbeat) });
  }

  dispose(): void {
    this.disposables.forEach(disposable => disposable.dispose());
  }

  private async sendEditEvent(event: vscode.TextDocumentChangeEvent): Promise<void> {
    const now = Date.now();
    const key = event.document.uri.toString();
    const lastSent = this.lastSentByDocument.get(key) ?? 0;
    if (now - lastSent < eventDebounceMs()) {
      return;
    }
    this.lastSentByDocument.set(key, now);

    const linesAdded = event.contentChanges.reduce((total, change) => total + change.text.split('\n').length - 1, 0);
    await this.sendDocumentEvent('document_edit', event.document, Math.max(linesAdded, 0));
  }

  private async sendDocumentEvent(eventType: string, document: vscode.TextDocument, linesAdded = 0): Promise<void> {
    if (document.uri.scheme !== 'file') {
      return;
    }

    const payload = {
      timestamp: new Date().toISOString(),
      session_id: this.session.sessionId,
      event_type: eventType,
      project: {
        name: this.session.workspaceName(),
        path: this.session.workspacePath()
      },
      activity: {
        files_modified: [vscode.workspace.asRelativePath(document.uri, false)],
        languages: { [document.languageId]: 1 },
        lines_added: linesAdded,
        lines_removed: 0,
        time_spent_minutes: eventType === 'document_edit' ? 1 : 0
      },
      metadata: {
        uri_scheme: document.uri.scheme
      }
    };

    await this.trySend(payload);
  }

  private async sendHeartbeat(): Promise<void> {
    const payload = {
      timestamp: new Date().toISOString(),
      session_id: this.session.sessionId,
      event_type: 'session_heartbeat',
      project: {
        name: this.session.workspaceName(),
        path: this.session.workspacePath()
      },
      activity: {
        time_spent_minutes: 1
      },
      metadata: {}
    };

    await this.trySend(payload);
  }

  private async trySend(payload: unknown): Promise<void> {
    try {
      await this.backend.sendEvent(payload);
    } catch {
      return;
    }
  }
}
