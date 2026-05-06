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

interface WorktreeSnapshot {
  signature: string;
  files: string[];
  linesAdded: number;
  linesRemoved: number;
  diffSummary?: string;
  statusSummary: string;
}

export class GitWatcher implements vscode.Disposable {
  private readonly disposables: vscode.Disposable[] = [];
  private lastCommitHash?: string;
  private lastWorktreeSignature?: string;

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

    const git = await this.session.gitContext();
    const snapshot = await latestCommit(cwd);
    if (snapshot && snapshot.hash !== this.lastCommitHash) {
      this.lastCommitHash = snapshot.hash;
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

    const worktree = await worktreeSnapshot(cwd);
    if (!worktree || worktree.signature === this.lastWorktreeSignature) {
      return;
    }

    this.lastWorktreeSignature = worktree.signature;
    try {
      await this.backend.sendEvent({
        timestamp: new Date().toISOString(),
        session_id: this.session.sessionId,
        event_type: 'git_snapshot',
        project: {
          name: this.session.workspaceName(),
          path: cwd,
          git_branch: git.branch,
          git_remote: git.remote
        },
        activity: {
          files_modified: worktree.files,
          languages: {},
          lines_added: worktree.linesAdded,
          lines_removed: worktree.linesRemoved,
          time_spent_minutes: 0
        },
        metadata: {
          diff_summary: worktree.diffSummary,
          status_summary: worktree.statusSummary,
          changed_files_count: worktree.files.length
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

async function worktreeSnapshot(cwd: string): Promise<WorktreeSnapshot | undefined> {
  const [status, diffSummary, diffFiles] = await Promise.all([
    runGit(cwd, ['status', '--short']),
    runGit(cwd, ['diff', '--shortstat']),
    runGit(cwd, ['diff', '--name-only'])
  ]);
  const statusLines = (status ?? '').split(/\r?\n/).map(line => line.trimEnd()).filter(Boolean);
  if (statusLines.length === 0 && !diffSummary) {
    return undefined;
  }

  const statusFiles = statusLines.map(statusFileName).filter((file): file is string => Boolean(file));
  const files = Array.from(new Set([
    ...(diffFiles ?? '').split(/\r?\n/).map(line => line.trim()).filter(Boolean),
    ...statusFiles
  ])).sort();
  const parsed = parseShortStat(diffSummary ?? '');
  const statusSummary = summarizeStatus(statusLines);
  const signature = [diffSummary, statusSummary, files.join('|')].join('::');

  return {
    signature,
    files,
    linesAdded: parsed.linesAdded,
    linesRemoved: parsed.linesRemoved,
    diffSummary: diffSummary?.trim() || undefined,
    statusSummary
  };
}

async function runGit(cwd: string, args: string[]): Promise<string | undefined> {
  try {
    const { stdout } = await execFileAsync('git', ['-C', cwd, ...args], { timeout: 5000 });
    return stdout.trim() || undefined;
  } catch {
    return undefined;
  }
}

function parseShortStat(summary: string): { linesAdded: number; linesRemoved: number } {
  const insertions = summary.match(/(\d+) insertion/);
  const deletions = summary.match(/(\d+) deletion/);
  return {
    linesAdded: insertions ? Number(insertions[1]) : 0,
    linesRemoved: deletions ? Number(deletions[1]) : 0
  };
}

function summarizeStatus(lines: string[]): string {
  let staged = 0;
  let unstaged = 0;
  let untracked = 0;
  for (const line of lines) {
    if (line.startsWith('??')) {
      untracked += 1;
      continue;
    }
    if (line[0] && line[0] !== ' ') {
      staged += 1;
    }
    if (line[1] && line[1] !== ' ') {
      unstaged += 1;
    }
  }
  return `staged: ${staged}, unstaged: ${unstaged}, untracked: ${untracked}`;
}

function statusFileName(line: string): string | undefined {
  const file = line.slice(3).trim();
  if (!file) {
    return undefined;
  }
  const renamed = file.split(' -> ').pop();
  return renamed?.replace(/\\/g, '/');
}