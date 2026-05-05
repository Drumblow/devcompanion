import { backendUrl } from '../config';

export interface Draft {
  id: number;
  session_date: string;
  content: string;
  status: string;
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
