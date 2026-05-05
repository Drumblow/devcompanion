import { backendUrl } from '../config';

export interface Draft {
  id: number;
  session_date: string;
  content: string;
  status: string;
}

export interface DailySummary {
  date: string;
  event_count: number;
  total_time_minutes: number;
  lines_added: number;
  lines_removed: number;
  git_commits: number;
  projects: string[];
  languages: Record<string, number>;
  files_modified: string[];
  voice_examples: number;
}

export interface RecentEvent {
  id: number;
  timestamp: string;
  event_type: string;
  project_name?: string;
  git_branch?: string;
  files_modified: string[];
  languages: Record<string, number>;
}

export interface DashboardSnapshot {
  summary: DailySummary;
  recent_events: RecentEvent[];
  pending_drafts: Draft[];
}

export class RustBackend {
  async health(): Promise<boolean> {
    try {
      const response = await fetch(`${backendUrl()}/health`);
      return response.ok;
    } catch {
      return false;
    }
  }

  async sendEvent(payload: unknown): Promise<void> {
    const response = await fetch(`${backendUrl()}/events`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (!response.ok) {
      throw new Error(`Falha ao enviar evento: ${response.status}`);
    }
  }

  async generateDraft(): Promise<Draft> {
    const response = await fetch(`${backendUrl()}/posts/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({})
    });
    if (!response.ok) {
      throw new Error(`Falha ao gerar rascunho: ${response.status}`);
    }
    return response.json() as Promise<Draft>;
  }

  async pendingDrafts(): Promise<Draft[]> {
    const response = await fetch(`${backendUrl()}/posts/pending`);
    if (!response.ok) {
      throw new Error(`Falha ao buscar rascunhos: ${response.status}`);
    }
    return response.json() as Promise<Draft[]>;
  }

  async dashboard(): Promise<DashboardSnapshot> {
    const response = await fetch(`${backendUrl()}/dashboard/today`);
    if (!response.ok) {
      throw new Error(`Falha ao buscar dashboard: ${response.status}`);
    }
    return response.json() as Promise<DashboardSnapshot>;
  }

  async approveDraft(id: number, approvedContent?: string): Promise<Draft> {
    const response = await fetch(`${backendUrl()}/posts/${id}/approve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ approved_content: approvedContent })
    });
    if (!response.ok) {
      throw new Error(`Falha ao aprovar rascunho: ${response.status}`);
    }
    return response.json() as Promise<Draft>;
  }

  async saveVoiceExample(text: string, context: string): Promise<void> {
    const response = await fetch(`${backendUrl()}/personality/examples`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text, context })
    });
    if (!response.ok) {
      throw new Error(`Falha ao salvar exemplo de voz: ${response.status}`);
    }
  }
}
