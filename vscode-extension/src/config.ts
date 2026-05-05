import * as vscode from 'vscode';

export function backendUrl(): string {
  return vscode.workspace.getConfiguration('linkedinDevCompanion').get<string>('backendUrl', 'http://127.0.0.1:8787');
}

export function eventDebounceMs(): number {
  return vscode.workspace.getConfiguration('linkedinDevCompanion').get<number>('eventDebounceMs', 5000);
}
