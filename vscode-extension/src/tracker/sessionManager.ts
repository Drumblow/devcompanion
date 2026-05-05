import * as vscode from 'vscode';
import { execFile } from 'child_process';
import { randomUUID } from 'crypto';
import { promisify } from 'util';

const execFileAsync = promisify(execFile);

export interface GitContext {
  branch?: string;
  remote?: string;
}

export class SessionManager {
  readonly sessionId: string;

  constructor(private readonly extensionContext: vscode.ExtensionContext) {
    const stored = extensionContext.globalState.get<string>('sessionId');
    this.sessionId = stored ?? randomUUID();
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

  async gitContext(): Promise<GitContext> {
    const cwd = this.workspacePath();
    if (!cwd) {
      return {};
    }

    const [branch, remote] = await Promise.all([
      runGit(cwd, ['rev-parse', '--abbrev-ref', 'HEAD']),
      runGit(cwd, ['config', '--get', 'remote.origin.url'])
    ]);

    return {
      branch: branch || undefined,
      remote: remote || undefined
    };
  }
}

async function runGit(cwd: string, args: string[]): Promise<string | undefined> {
  try {
    const { stdout } = await execFileAsync('git', ['-C', cwd, ...args], { timeout: 5000 });
    return stdout.trim() || undefined;
  } catch {
    return undefined;
  }
}
