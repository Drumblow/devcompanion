import { backendUrl } from '../config';

export interface Draft {
  id: number;
  session_date: string;
  content: string;
  status: string;
  style_score?: number;
  rejected_at?: string;
  rejection_reason?: string;
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

export interface CopilotStatus {
  enabled: boolean;
  available: boolean;
  cli_path: string;
  model: string;
  message: string;
}

export interface TechnicalAnalysis {
  source: string;
  status: string;
  insights: string[];
  tech_stack: string[];
  complexity?: number;
  learnings: string[];
  raw?: string;
  error?: string;
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

  async copilotStatus(): Promise<CopilotStatus> {
    const response = await fetch(`${backendUrl()}/copilot/status`);
    if (!response.ok) {
      throw new Error(`Falha ao verificar Copilot CLI: ${response.status}`);
    }
    return response.json() as Promise<CopilotStatus>;
  }

  async todayAnalysis(): Promise<TechnicalAnalysis> {
    const response = await fetch(`${backendUrl()}/analysis/today`);
    if (!response.ok) {
      throw new Error(`Falha ao buscar analise tecnica: ${response.status}`);
    }
    return response.json() as Promise<TechnicalAnalysis>;
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

  async rejectDraft(id: number, reason: string): Promise<Draft> {
    const response = await fetch(`${backendUrl()}/posts/${id}/reject`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ reason })
    });
    if (!response.ok) {
      throw new Error(`Falha ao rejeitar rascunho: ${response.status}`);
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
