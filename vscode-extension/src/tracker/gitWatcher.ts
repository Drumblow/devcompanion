import { execFile } from 'child_process';
import { promisify } from 'util';
import * as vscode from 'vscode';
import { RustBackend } from '../api/rustBackend';
import { SessionManager } from './sessionManager';

const execFileAsync = promisify(execFile);

interface CommitSnapshot {
  hash: string;
  subject: string;
  committedAt: string;
  files: string[];
}

export class GitWatcher implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private lastCommitHash?: string;

  constructor(private readonly backend: RustBackend, private readonly session: SessionManager) {}

  start(): void {
    void this.poll();
    const interval = setInterval(() => void this.poll(), 60000);
    this.disposables.push({ dispose: () => clearInterval(interval) });
  }

  dispose(): void {
    this.disposables.forEach(disposable => disposable.dispose());
  }

  private async poll(): Promise<void> {
    const cwd = this.session.workspacePath();
    if (!cwd) {
      return;
    }

    const snapshot = await latestCommit(cwd);
    if (!snapshot || snapshot.hash === this.lastCommitHash) {
      return;
    }

    this.lastCommitHash = snapshot.hash;
    const git = await this.session.gitContext();

    try {
      await this.backend.sendEvent({
        timestamp: new Date().toISOString(),
        session_id: this.session.sessionId,
        event_type: 'git_commit',
        project: {
          name: this.session.workspaceName(),
          path: cwd,
          git_branch: git.branch,
          git_remote: git.remote
        },
        activity: {
          files_modified: snapshot.files,
          languages: {},
          lines_added: 0,
          lines_removed: 0,
          time_spent_minutes: 0
        },
        metadata: {
          commit_hash: snapshot.hash,
          subject: snapshot.subject,
          committed_at: snapshot.committedAt
        }
      });
    } catch {
      return;
    }
  }
}

async function latestCommit(cwd: string): Promise<CommitSnapshot | undefined> {
  try {
    const { stdout } = await execFileAsync(
      'git',
      ['-C', cwd, 'log', '-1', '--name-only', '--format=%H%n%s%n%cI'],
      { timeout: 5000 }
    );
    const lines = stdout.split(/\r?\n/).map(line => line.trim()).filter(Boolean);
    const [hash, subject, committedAt, ...files] = lines;
    if (!hash || !subject || !committedAt) {
      return undefined;
    }
    return { hash, subject, committedAt, files };
  } catch {
    return undefined;
  }
}