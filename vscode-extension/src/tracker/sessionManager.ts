import * as vscode from 'vscode';

export class SessionManager {
  readonly sessionId: string;

  constructor(private readonly extensionContext: vscode.ExtensionContext) {
    const stored = extensionContext.globalState.get<string>('sessionId');
    this.sessionId = stored ?? crypto.randomUUID();
    if (!stored) {
      void extensionContext.globalState.update('sessionId', this.sessionId);
    }
  }

  workspaceName(): string | undefined {
    return vscode.workspace.name;
  }

  workspacePath(): string | undefined {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  }
}
