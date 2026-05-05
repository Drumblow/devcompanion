# 🧠 LinkedIn Dev Companion
## Automação de Conteúdo Técnico para LinkedIn com Personalidade Adaptativa

**Versão:** 1.1.0  
**Data:** 2026-05-05  
**Status:** Especificação de Arquitetura (Revisada)

> **Changelog v1.1.0:** Removido Copilot Bridge (Node.js) — substituído por invocação direta do Copilot CLI via subprocess Rust. Atualizada integração LinkedIn para a Posts API (ugcPosts API depreciada). Corrigidos limites de tokens do GPT-5.4 (contexto 1M, output máx 128K). Adicionada nota sobre cobrança de premium requests no Copilot SDK. Seção de humanização reformulada para foco em autenticidade. Mencionada Responses API da OpenAI.

---

## 📋 Sumário

1. [Visão Geral](#1-visão-geral)
2. [Arquitetura do Sistema](#2-arquitetura-do-sistema)
3. [Componentes](#3-componentes)
4. [Fluxo de Dados](#4-fluxo-de-dados)
5. [Estratégia de Personalidade & Voz](#5-estratégia-de-personalidade--voz)
6. [Integração com GitHub Copilot & GPT-5.4](#6-integração-com-github-copilot--gpt-54)
7. [Especificação Técnica](#7-especificação-técnica)
8. [Plano de Implementação](#8-plano-de-implementação)
9. [Segurança & Privacidade](#9-segurança--privacidade)
10. [Configuração & Deploy](#10-configuração--deploy)

---

## 1. Visão Geral

### 1.1 Propósito
O **LinkedIn Dev Companion** é um agente autônomo que acompanha o desenvolvimento diário no VS Code, analisa padrões de código, aprende com a comunicação do usuário e gera conteúdo técnico autêntico para o LinkedIn — tudo automaticamente, com tom natural e indetectável por ferramentas de IA.

### 1.2 Funcionalidades Principais
- ✅ **Tracking contínuo** de atividade de desenvolvimento no VS Code (independente do projeto)
- ✅ **Análise técnica** via GitHub Copilot CLI (invocado diretamente por subprocess — sem side-car Node.js)
- ✅ **Geração de conteúdo** via GPT-5.4 com reasoning effort `medium`
- ✅ **Aprendizado de personalidade** a partir dos prompts do chat diário
- ✅ **Autenticidade** como princípio central — conteúdo genuíno, não fabricado
- ✅ **Aprovação humana** obrigatória no MVP antes de qualquer publicação
- ✅ **Backend 100% Rust** — performático, seguro e com baixo consumo de recursos

### 1.3 Diferenciais
- Não requer alteração nos projetos — funciona como extensão global do VS Code
- Usa o plano existente do GitHub Copilot para análise de código **via CLI direto** (sem side-car Node.js)
- Personalidade evolutiva baseada em interações reais e exemplos aprovados explicitamente
- Aprovação humana como etapa obrigatória — o sistema propõe, o usuário decide

---

## 2. Arquitetura do Sistema

### 2.1 Diagrama de Alto Nível

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              USUÁRIO (VS Code)                              │
│  ┌──────────────────────┐                                                   │
│  │  VS Code Extension   │                                                   │
│  │  (TypeScript)        │                                                   │
│  │                      │                                                   │
│  │  • Event tracking    │                                                   │
│  │  • Captura de diff   │                                                   │
│  │  • Envio de dados    │                                                   │
│  │  • UI de aprovação   │                                                   │
│  └──────────┬───────────┘                                                   │
└─────────────┼───────────────────────────────────────────────────────────────┘
              │ HTTP local / named pipe
              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         BACKEND RUST (Serviço Local)                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐ │
│  │  API Server  │  │  Ingestor    │  │  Orquestrador│  │  Agendador      │ │
│  │  (Axum)      │  │  de Eventos  │  │  de Conteúdo │  │  (Tokio Cron)   │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └─────────────────┘ │
│         │                 │                 │                               │
│  ┌──────▼───────┐  ┌──────▼───────┐  ┌──────▼───────┐                      │
│  │  Banco de    │  │  Motor de    │  │  LinkedIn    │                      │
│  │  Dados       │  │  Personalidade│  │  Publisher   │                      │
│  │  (SQLite)    │  │  + RAG       │  │  (Posts API) │                      │
│  └──────────────┘  └──────────────┘  └──────────────┘                      │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  Copilot CLI Adapter (subprocess)                                   │  │
│  │  • Chama `copilot -p "..." -s --no-ask-user` via std::process       │  │
│  │  • Sem dependência de Node.js ou side-car                           │  │
│  │  • Auth via COPILOT_GITHUB_TOKEN (env var)                          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SERVIÇOS EXTERNOS                              │
│  ┌──────────────────────┐    ┌──────────────────────┐                      │
│  │  OpenAI API          │    │  LinkedIn Posts API  │                      │
│  │  (GPT-5.4)           │    │  (OAuth 2.0)         │                      │
│  │  Responses API       │    │  /rest/posts         │                      │
│  └──────────────────────┘    └──────────────────────┘                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Por que Rust no Backend?
- **Performance:** Processamento de eventos em tempo real com mínima latência
- **Segurança:** Memory safety sem garbage collector — ideal para serviço contínuo
- **Concorrência:** Modelo async/await do Tokio permite milhares de conexões simultâneas
- **Binário único:** Deploy simples, sem dependências de runtime
- **Consumo de recursos:** Ideal para rodar em background no desenvolvimento diário

### 2.3 Por que Copilot CLI via Subprocess (em vez de SDK ou Node.js side-car)?

O Copilot CLI suporta uso **completamente programático** com a flag `-p` (ou piped input):

```bash
copilot -p "Analise este diff e identifique decisões técnicas relevantes" -s --no-ask-user
```

A flag `-s` (silent) suprime metadados de sessão e retorna apenas o texto. Isso significa que do Rust basta um `std::process::Command` — **sem Node.js, sem side-car, sem dependência extra**. Benefícios:

- Mesma autenticação do usuário já configurada no Copilot CLI local
- Controle de permissões via `--allow-tool=[TOOLS...]` por chamada
- Modelo configurável por chamada via `--model`
- Auth via variável de ambiente `COPILOT_GITHUB_TOKEN`

> ⚠️ **Nota importante:** Cada chamada ao Copilot CLI consome **premium requests** da cota do plano do usuário (não é gratuita dentro do plano Copilot). Para análises de código diárias, o consumo deve ser moderado (1-2 chamadas por dia de desenvolvimento).

> ℹ️ Existe um **Copilot SDK Rust não-oficial** em `copilot-community-sdk/copilot-sdk-rust`, porém não é suportado pelo GitHub. Para produção, preferir o CLI direto que é estável e oficialmente documentado. O Copilot SDK oficial está em **Public Preview** e disponível apenas para Node.js, Python, Go, .NET e Java.

---

## 3. Componentes

### 3.1 VS Code Extension (`linkedin-dev-companion-ext`)

**Tecnologia:** TypeScript + VS Code Extension API

**Responsabilidades:**
- Capturar eventos globais do VS Code (independente do workspace)
- Rastrear: arquivos editados, linguagens usadas, tempo por projeto, git activity
- Enviar payloads para o backend Rust via WebSocket
- Exibir status e notificações na status bar

**Eventos Capturados:**

| Evento | Fonte VS Code | Frequência |
|--------|---------------|------------|
| `document_open` | `onDidOpenTextDocument` | Real-time |
| `document_edit` | `onDidChangeTextDocument` | Debounced (5s) |
| `active_editor` | `onDidChangeActiveTextEditor` | Real-time |
| `git_commit` | Git API / file watcher | Real-time |
| `terminal_command` | `onDidExecuteTerminalCommand` | Opcional |
| `diagnostic` | `onDidChangeDiagnostics` | Aggregated |
| `session_heartbeat` | Interval (60s) | Periódico |

**Estrutura do Payload:**
```json
{
  "timestamp": "2026-05-05T14:30:00Z",
  "session_id": "uuid",
  "event_type": "coding_session",
  "project": {
    "name": "meu-app-rust",
    "path": "/home/dev/projects/meu-app-rust",
    "git_branch": "feature/auth",
    "git_remote": "github.com/user/repo"
  },
  "activity": {
    "files_modified": ["src/auth.rs", "Cargo.toml"],
    "languages": {"rust": 45, "toml": 5},
    "lines_added": 120,
    "lines_removed": 30,
    "functions_touched": ["authenticate", "validate_token"],
    "time_spent_minutes": 45,
    "errors_encountered": 2,
    "libraries_used": ["axum", "jsonwebtoken", "serde"]
  }
}
```

### 3.2 Backend Rust (`linkedin-dev-companion`)

**Tecnologia:** Rust + Tokio + Axum + SQLx + Reqwest

**Módulos:**

#### `api_server` (Axum)
- REST API para receber eventos da extensão
- WebSocket para comunicação bidirecional
- Endpoints de configuração e status

#### `ingestor`
- Normaliza e valida eventos recebidos
- Armazena no SQLite com SQLx (migrations incluídas)
- Agrega dados por sessão/dia

#### `orchestrator`
- Executa pipeline diária de geração de conteúdo
- Coordena chamadas ao Copilot (análise) e GPT-5.4 (criação)
- Aplica regras de personalidade

#### `personality_engine`
- Sistema RAG (Retrieval Augmented Generation) para memória
- Vetorização de prompts do usuário
- Construção dinâmica de system prompts

#### `linkedin_publisher`
- Integração com **LinkedIn Posts API** (substitui ugcPosts API, depreciada em abril/2025)
- Endpoint: `POST https://api.linkedin.com/rest/posts`
- Requer header: `Linkedin-Version: YYYYMM` e `X-Restli-Protocol-Version: 2.0.0`
- OAuth 2.0 flow com scope `w_member_social`
- Agendamento e publicação somente após aprovação humana

#### `scheduler`
- Tokio cron jobs para execução diária
- Configuração de horário de publicação
- Retry logic com backoff exponencial

### 3.3 GitHub Copilot CLI Adapter (`ldc-copilot`)

**Tecnologia:** Rust puro — `std::process::Command` + `tokio::process::Command`

**Por que não usar o SDK oficial?** O GitHub Copilot SDK oficial (em Public Preview) está disponível para Node.js, Python, Go, .NET e Java — mas **não para Rust**. O CLI, no entanto, pode ser invocado diretamente via subprocess com a flag `-p`, o que elimina qualquer dependência de side-car externo.

**Funcionamento:**

```
Rust Backend ──subprocess──► copilot -p "prompt" -s --no-ask-user
                              └─► resposta em stdout como texto puro
```

**Implementação em Rust:**

```rust
use tokio::process::Command;

pub struct CopilotAdapter {
    github_token: String,
}

impl CopilotAdapter {
    pub async fn analyze_session(&self, diff: &str, project_context: &str) -> Result<String> {
        let prompt = format!(
            "Contexto do projeto: {project_context}\n\nDiff do dia:\n{diff}\n\n\
             Analise em JSON com os campos: \
             insights (array), tech_stack (array), complexity (0-10), learnings (array)"
        );

        let output = Command::new("copilot")
            .args(["-p", &prompt, "-s", "--no-ask-user"])
            .args(["--allow-tool", "shell(git:*)"])  // permissão mínima necessária
            .env("COPILOT_GITHUB_TOKEN", &self.github_token)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Copilot CLI error: {}", stderr));
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}
```

**Observações:**
- Requer `copilot` no PATH da máquina do usuário (instalado via `gh extension install github/gh-copilot` ou standalone)
- Cada chamada consome premium requests da cota do plano
- Para análise de diff diária, recomendar `--model copilot-latest` para consistência

### 3.4 OpenAI GPT-5.4 Integration

**Modelo recomendado:** `gpt-5.4` via OpenAI Responses API  
**Parâmetro:** `reasoning.effort: "medium"` (equilíbrio custo/qualidade para posts)  
**Janela de contexto:** 1M tokens  
**Output máximo:** 128K tokens  
**Preço:** $2,50/M tokens input · $15/M tokens output  
**Data de corte do conhecimento:** 31 ago 2025

> ℹ️ **Alternativa mais capaz:** `gpt-5.5` (lançado em 05/05/2026) — melhor para raciocínio complexo. Preço: $5/M input, $30/M output. Recomendado para geração de posts que exijam mais contexto histórico ou narrativa longa.

**API:** A OpenAI lançou a **Responses API** como substituta moderna da Chat Completions API. Ambas funcionam, mas a Responses API suporta ferramentas nativas (web search, file search, computer use) e melhor suporte a agentes.

**Uso:** Geração final do conteúdo do LinkedIn, com:
- Contexto do dia de desenvolvimento (vindo do Copilot CLI)
- Perfil de personalidade do usuário
- Exemplos aprovados (few-shot via RAG)
- Formato específico do LinkedIn

**Exemplo de chamada via Responses API (Rust):**

```rust
use reqwest::Client;
use serde_json::json;

async fn generate_post(content_context: &str, personality_prompt: &str) -> Result<String> {
    let client = Client::new();

    let response = client
        .post("https://api.openai.com/v1/responses")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "gpt-5.4",
            "reasoning": {
                "effort": "medium"
            },
            "instructions": personality_prompt,
            "input": format!(
                "Crie um post de LinkedIn baseado nesse contexto de desenvolvimento: {}",
                content_context
            ),
            "max_output_tokens": 800
        }))
        .send()
        .await?;

    // parse response.output[0].content[0].text
}
```

> ℹ️ Se preferir manter a Chat Completions API por familiaridade, ela continua funcional. Use `POST /v1/chat/completions` com `"model": "gpt-5.4"` e o campo `"reasoning": {"effort": "medium"}`. Evite `temperature` alto em modelos com reasoning — pode conflitar com o processo interno do modelo.

---

## 4. Fluxo de Dados

### 4.1 Fluxo Diário Completo

```
08:00 - Usuário abre VS Code
        └─► Extensão ativa e inicia sessão
        └─► Conecta WebSocket com backend Rust

08:00-17:00 - Desenvolvimento contínuo
        └─► Extensão captura eventos e envia a cada 5 minutos
        └─► Backend ingere e armazena no SQLite
        └─► Git commits são detectados e analisados

17:30 - Trigger do Agendador (horário configurável)
        └─► Orquestrador inicia pipeline

        PASSO 1: Compilação do Dia
        └─► Query no DB: resumo das 8h de desenvolvimento
        └─► Gera "Dev Diary" estruturado

        PASSO 2: Análise Técnica (GitHub Copilot)
        └─► Envia diff + contexto para Copilot Bridge
        └─► Recebe insights técnicos detalhados

        PASSO 3: Recuperação de Personalidade (RAG)
        └─► Consulta vetor DB com prompts históricos do usuário
        └─► Monta "voice profile" dinâmico do dia

        PASSO 4: Geração de Conteúdo (GPT-5.4)
        └─► Prompt enriquecido: insights + personalidade + regras
        └─► Gera 3 variações de post

        PASSO 5: Humanização
        └─► Aplica técnicas de variação linguística
        └─► Verifica contra padrões detectáveis de IA

        PASSO 6: Publicação (LinkedIn API)
        └─► Se modo automático: publica o mais engajador
        └─► Se modo review: envia notificação para aprovação
```

### 4.2 Estrutura do Banco de Dados (SQLite)

```sql
-- Eventos brutos do VS Code
CREATE TABLE coding_events (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    project_name TEXT,
    project_path TEXT,
    git_branch TEXT,
    event_type TEXT, -- 'edit', 'open', 'commit', 'heartbeat'
    file_path TEXT,
    language TEXT,
    lines_added INTEGER DEFAULT 0,
    lines_removed INTEGER DEFAULT 0,
    duration_seconds INTEGER DEFAULT 0,
    metadata JSON
);

-- Sessões diárias agregadas
CREATE TABLE daily_sessions (
    id INTEGER PRIMARY KEY,
    date DATE UNIQUE,
    total_time_minutes INTEGER,
    projects JSON,
    languages JSON,
    files_modified JSON,
    git_commits INTEGER,
    summary_generated TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Memória de personalidade (prompts do usuário)
CREATE TABLE user_prompts (
    id INTEGER PRIMARY KEY,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    prompt_text TEXT NOT NULL,
    context TEXT, -- 'chat', 'code_review', 'debug'
    embedding BLOB, -- vetor para RAG
    extracted_style JSON -- análise de tom, estrutura, gírias
);

-- Posts gerados
CREATE TABLE generated_posts (
    id INTEGER PRIMARY KEY,
    session_date DATE,
    content TEXT NOT NULL,
    variation INTEGER, -- 1, 2, 3
    engagement_score REAL, -- predição
    posted BOOLEAN DEFAULT FALSE,
    post_id TEXT, -- LinkedIn URN
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Configurações
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 5. Estratégia de Personalidade & Voz

### 5.1 Sistema de Aprendizado (Chat → Personalidade)

**Objetivo:** O agente aprende com os prompts que o usuário envia no dia a dia para replicar:
- Tom de voz (formal/informal, técnico/acessível)
- Estrutura de pensamento (direto, analítico, narrativo)
- Expressões e vocabulário característicos
- Formato preferido de explicação técnica
- Trade-offs e opiniões reais do usuário

**Implementação:**

1. **Coleta explícita:** O usuário marca ativamente textos de sua autoria como "exemplos de estilo" — nenhuma coleta implícita de conversas privadas
2. **Vetorização:** Usa embeddings (OpenAI `text-embedding-3-large`) para indexar semanticamente
3. **Análise de Estilo:** Extrai features linguísticas:
   - Comprimento médio de frases
   - Nível de jargão técnico
   - Tom (assertivo, questionador, colaborativo)
   - Conectores e marcadores discursivos preferidos
4. **RAG na Geração:** Antes de criar o post, consulta os 5 exemplos mais similares ao tema do dia e injeta como few-shot examples

**Exemplo de System Prompt Dinâmico:**
```
Você vai gerar um rascunho de post para LinkedIn a partir de contexto real de desenvolvimento.

PERFIL DE VOZ DO USUÁRIO (construído a partir de exemplos aprovados por ele):
- Tom: Direto, com toque de humor seco. Evita fluff.
- Estrutura: Começa com contexto real do trabalho.
  Desenvolve em 2-3 parágrafos curtos. Fecha com aprendizado ou trade-off.
- Linguagem: Mistura português técnico com termos em inglês quando natural.
- Evita: Listas numeradas excessivas, frases genéricas de produtividade,
  conclusões do tipo "espero que isso ajude alguém".

EXEMPLOS DE POSTS DO USUÁRIO (recuperados do RAG):
[5 exemplos mais relevantes do vetor DB]

INSTRUÇÕES PARA O RASCUNHO:
- Base o conteúdo 100% no contexto técnico real fornecido
- Não invente insights que não estejam no contexto
- Gere um rascunho honesto — o usuário irá revisar e aprovar antes de publicar
- Se o contexto não for rico o suficiente para um post, diga isso em vez de gerar conteúdo vago
```

### 5.2 Princípios de Qualidade de Conteúdo

**Princípio central: autenticidade, não performance.**

O objetivo do sistema não é enganar detectores de IA ou parecer mais humano do que é. O objetivo é gerar **rascunhos com contexto técnico real** que o usuário **revisa, edita e publica como seus** — porque de fato são, já que partem do seu trabalho real.

**O que torna um post bom:**

| Elemento | Por que importa | Como o sistema contribui |
|----------|----------------|--------------------------|
| **Contexto real** | Leitores identificam experiência genuína | O tracking captura o que você realmente fez no dia |
| **Voz consistente** | Constrói identidade reconhecível | RAG a partir de exemplos aprovados pelo usuário |
| **Profundidade técnica** | Diferencia de posts genéricos | Análise via Copilot CLI extrai insights reais do código |
| **Trade-offs honestos** | Mais crível que só mostrar sucessos | O prompt instrui a incluir dificuldades encontradas |
| **Revisão humana** | Você garante que está confortável com o que publica | Aprovação obrigatória no MVP |

**O que o sistema NÃO faz:**
- Não gera conteúdo "para parecer humano" artificialmente
- Não tenta contornar políticas de plataformas
- Não publica sem aprovação do usuário (no MVP)
- Não coleta dados sem consentimento explícito

**Pipeline de geração:**
```
Contexto do dia (tracking)
        │
        ▼
Análise Copilot CLI (insights técnicos reais)
        │
        ▼
RAG: recupera exemplos de voz do usuário
        │
        ▼
GPT-5.4 gera rascunho contextualizado
        │
        ▼
Score de aderência ao perfil (RF-07)
        │
        ▼
Enviado para aprovação do usuário ← etapa obrigatória no MVP
        │
        ▼
Usuário revisa, edita se quiser, e aprova
        │
        ▼
Publicação no LinkedIn
```

---

## 6. Integração com GitHub Copilot & GPT-5.4

### 6.1 GitHub Copilot CLI — Uso Direto por Subprocess

O Copilot CLI suporta **uso completamente programático** via flag `-p`:

```bash
# Uso direto no terminal
copilot -p "Explique as mudanças no último commit" -s --allow-tool='shell(git:*)'

# Captura output em variável (shell scripting)
result=$(copilot -p "Analise src/auth.rs em 3 pontos" -s)
```

Do Rust, basta `tokio::process::Command`. Não há necessidade de Node.js, side-car, ou SDK separado.

**Opções de integração comparadas:**

| Método | Pros | Cons |
|--------|------|------|
| **Copilot CLI via subprocess** ✅ Recomendado | Sem dependências extras, auth já configurada, controle de permissões por chamada | Requer CLI instalado no PATH |
| **Copilot SDK oficial (Node.js)** | API estruturada, multi-turn | Requer Node.js + side-car, adiciona complexidade, ainda em Public Preview |
| **Copilot SDK Rust (não-oficial)** | Integração nativa Rust | Não suportado pelo GitHub, risco de manutenção |
| **OpenAI API direta** | Mais controle, sem dependência do Copilot | Custo adicional, não usa o plano existente |

**Autenticação:**

```bash
# Opção 1: variável de ambiente (recomendado para daemon)
export COPILOT_GITHUB_TOKEN="ghp_..."

# Opção 2: login interativo (usuário já fez, reutilizado pelo CLI)
gh auth login
```

**Configuração do Adapter (Rust):**

```rust
// crates/ldc-copilot/src/lib.rs
use tokio::process::Command;
use anyhow::{Result, Context};

pub struct CopilotAdapter {
    token: String,
    model: String,  // "copilot-latest" para consistência
}

impl CopilotAdapter {
    pub async fn analyze_daily_diff(
        &self,
        diff_summary: &str,
        project_context: &str,
    ) -> Result<String> {
        let prompt = format!(
            "Você é um tech lead revisando o trabalho do dia. \
             Projeto: {project_context}\n\nDiff resumido:\n{diff_summary}\n\n\
             Retorne SOMENTE JSON válido com: \
             {{\"insights\": [], \"tech_stack\": [], \"complexity\": 0-10, \"learnings\": []}}"
        );

        let output = Command::new("copilot")
            .args(["-p", &prompt])
            .args(["-s", "--no-ask-user"])
            .args(["--model", &self.model])
            .args(["--allow-tool", "shell(git:*)"])
            .env("COPILOT_GITHUB_TOKEN", &self.token)
            .output()
            .await
            .context("Falha ao executar Copilot CLI — verifique se está instalado e no PATH")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Copilot CLI retornou erro: {}", stderr);
        }

        String::from_utf8(output.stdout)
            .context("Output do Copilot CLI não é UTF-8 válido")
    }
}
```

> **Limitar permissões:** Use `--allow-tool='shell(git:*)'` para acesso somente a comandos git. Evite `--allow-all` fora de ambiente sandbox.

### 6.2 GPT-5.4 via OpenAI Responses API

O GPT-5.4 suporta o parâmetro `reasoning.effort` com níveis: `none`, `low`, `medium` (padrão), `high`, `xhigh`.

**Para este projeto:**
- **Análise de código (Copilot CLI):** Não aplica — é o Copilot quem raciocina
- **Geração de rascunho:** `reasoning.effort: "medium"` — permite coerência narrativa sem custo excessivo
- **Se quiser mais qualidade:** `reasoning.effort: "high"` ou migrar para `gpt-5.5`

**Chamada via Responses API (Rust):**

```rust
use reqwest::Client;
use serde_json::json;

async fn generate_post_draft(
    content_context: &str,
    personality_prompt: &str,
    api_key: &str,
) -> Result<String> {
    let client = Client::new();

    let body = json!({
        "model": "gpt-5.4",
        "reasoning": {
            "effort": "medium"
        },
        "instructions": personality_prompt,
        "input": format!(
            "Gere um rascunho de post para LinkedIn a partir deste contexto de desenvolvimento:\n\n{}",
            content_context
        ),
        "max_output_tokens": 800
    });

    let resp = client
        .post("https://api.openai.com/v1/responses")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // Responses API retorna output[0].content[0].text
    resp["output"][0]["content"][0]["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Estrutura de resposta inesperada da OpenAI API"))
}
```

---

## 7. Especificação Técnica

### 7.1 Stack Tecnológica Completa

| Camada | Tecnologia | Versão | Justificativa |
|--------|-----------|--------|---------------|
| **Extensão VS Code** | TypeScript | ^5.0 | Tipagem nativa, excelente suporte à API do VS Code |
| **Backend** | Rust | 1.85+ | Performance, segurança, async nativo |
| **Web Framework** | Axum | 0.8 | Baseado em Tokio, ergonomia excelente |
| **Async Runtime** | Tokio | 1.4+ | Padrão de facto para async em Rust |
| **Database** | SQLite | 3.4+ | Zero-config, suficiente para dados locais |
| **ORM/Query** | SQLx | 0.8 | Queries verificadas em compile-time |
| **HTTP Client** | Reqwest | 0.12 | API ergonômica, baseada em hyper |
| **Scheduler** | tokio-cron-scheduler | 0.11 | Cron jobs async |
| **Serialização** | serde | 1.0 | Padrão Rust para JSON |
| **Config** | config + dotenv | - | Múltiplas fontes de config |
| **Logging** | tracing | 0.1 | Observabilidade estruturada |
| **Copilot** | Copilot CLI (subprocess) | latest | Via `std::process::Command` — sem side-car Node.js |
| **Embeddings** | OpenAI API | - | `text-embedding-3-large` |
| **LLM** | OpenAI Responses API | - | `gpt-5.4` com reasoning medium |

> ⚠️ **Removido da stack:** Node.js / `@github/copilot-sdk` — side-car desnecessário; o CLI cobre o caso de uso com menos complexidade operacional.

### 7.2 Estrutura de Diretórios

```
linkedin-dev-companion/
├── README.md
├── Cargo.toml                    # Workspace Rust
├── .env.example
├── config/
│   ├── default.toml
│   └── production.toml
├── docs/
│   ├── ARCHITECTURE.md
│   ├── API.md
│   └── DEPLOY.md
├── crates/
│   ├── ldc-core/                 # Tipos compartilhados, models, errors
│   ├── ldc-api/                  # Servidor HTTP (Axum)
│   ├── ldc-ingestor/             # Processamento de eventos
│   ├── ldc-orchestrator/         # Pipeline de conteúdo
│   ├── ldc-personality/          # Motor de personalidade + RAG
│   ├── ldc-copilot/              # Adapter para Copilot CLI (subprocess)
│   ├── ldc-linkedin/             # Integração LinkedIn Posts API
│   └── ldc-scheduler/            # Agendamento de tarefas
├── vscode-extension/
│   ├── package.json
│   ├── tsconfig.json
│   ├── src/
│   │   ├── extension.ts          # Entry point
│   │   ├── tracker/
│   │   │   ├── eventCollector.ts
│   │   │   ├── gitWatcher.ts
│   │   │   └── sessionManager.ts
│   │   ├── api/
│   │   │   └── rustBackend.ts
│   │   └── utils/
│   │       └── config.ts
│   └── out/
└── scripts/
    ├── install.sh
    └── dev-setup.sh
```

> **Removido:** `copilot-bridge/` — Node.js side-car não é mais necessário. A integração com o Copilot é feita pelo crate `ldc-copilot` via subprocess.

### 7.3 API do Backend (Rust)

```yaml
openapi: 3.0.0
info:
  title: LinkedIn Dev Companion API
  version: 1.0.0
paths:
  /events:
    post:
      summary: Recebe eventos de coding do VS Code
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CodingEvent'
      responses:
        201:
          description: Evento recebido

  /ws:
    get:
      summary: WebSocket para streaming de eventos em tempo real

  /sessions/{date}/summary:
    get:
      summary: Retorna resumo do dia
      responses:
        200:
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DailySummary'

  /posts/generate:
    post:
      summary: Gera posts para uma data específica
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                date:
                  type: string
                  format: date
                count:
                  type: integer
                  default: 3
      responses:
        200:
          description: Posts gerados

  /posts/{id}/publish:
    post:
      summary: Publica post no LinkedIn via Posts API (requer aprovação prévia)
      description: |
        Usa POST https://api.linkedin.com/rest/posts com header Linkedin-Version.
        Requer que o post já tenha sido aprovado pelo usuário.

  /personality/examples:
    post:
      summary: Adiciona exemplo explicitamente aprovado pelo usuário ao perfil de voz
      description: |
        O usuário marca um texto de sua autoria como referência de estilo.
        Não há coleta implícita de conversas.
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                text:
                  type: string
                  description: Texto de autoria do usuário (post anterior, etc.)
                context:
                  type: string
                  enum: ['linkedin_post', 'manual_note', 'approved_draft']

components:
  schemas:
    CodingEvent:
      type: object
      properties:
        timestamp:
          type: string
          format: date-time
        session_id:
          type: string
        project:
          type: object
        activity:
          type: object
```

---

## 8. Plano de Implementação

### Fase 1: Fundação (Semanas 1-2)
- [ ] Setup do workspace Rust com Cargo workspace
- [ ] Implementar `ldc-core` (tipos, models, errors)
- [ ] Implementar `ldc-api` com Axum (REST + WebSocket)
- [ ] Setup do SQLite com SQLx e migrations
- [ ] Criar estrutura base da extensão VS Code
- [ ] Comunicação básica Extensão ↔ Backend

### Fase 2: Tracking & Ingestão (Semanas 3-4)
- [ ] Implementar coleta de eventos no VS Code
- [ ] Capturar: edições, arquivos abertos, git commits
- [ ] Implementar `ldc-ingestor` com normalização
- [ ] Agregação de sessões diárias
- [ ] Dashboard local básico (opcional)

### Fase 3: Inteligência (Semanas 5-7)
- [ ] Implementar `ldc-copilot` — adapter subprocess para Copilot CLI
- [ ] Validar instalação e autenticação do Copilot CLI na máquina do usuário
- [ ] Integrar análise de diff via Copilot CLI (`copilot -p ... -s`)
- [ ] Implementar `ldc-personality` com embeddings (text-embedding-3-large)
- [ ] Sistema RAG para recuperação de exemplos de voz aprovados
- [ ] Integrar GPT-5.4 via OpenAI Responses API
- [ ] Pipeline de geração de rascunho com score de aderência ao perfil

### Fase 4: Aprovação & Publicação (Semanas 8-9)
- [ ] UI de aprovação na extensão VS Code (exibir rascunho, aceitar/rejeitar/editar)
- [ ] Integrar LinkedIn Posts API (OAuth + `POST /rest/posts` com headers de versão)
- [ ] Sistema de agendamento (`ldc-scheduler`)
- [ ] Modo somente-aprovação como padrão no MVP
- [ ] Auditoria: registrar contexto usado, modelo chamado, versão aprovada

### Fase 5: Polish & Deploy (Semanas 10-11)
- [ ] Testes E2E
- [ ] Documentação de usuário
- [ ] Script de instalação one-liner
- [ ] Otimização de performance
- [ ] Release v1.0

---

## 9. Segurança & Privacidade

### 9.1 Dados Sensíveis
- **Código-fonte:** O diff enviado para análise pode conter código proprietário
- **Exemplos de voz:** Armazenados localmente, coletados somente com ação explícita do usuário
- **Credenciais:** LinkedIn OAuth tokens e OpenAI API key armazenados no keyring do SO
- **Copilot CLI:** Auth gerenciada pelo próprio CLI (`gh auth`) — o daemon não armazena tokens do GitHub

### 9.2 Medidas de Segurança
- Todo processamento de código acontece via GitHub Copilot CLI (termos de privacidade do GitHub se aplicam)
- Dados armazenados localmente em SQLite (não há cloud próprio)
- Comunicação Extensão-Backend via localhost (HTTP)
- API keys em variáveis de ambiente ou keyring (via `keyring` crate)
- Permissões do Copilot CLI limitadas por chamada (`--allow-tool` específico, nunca `--allow-all` em produção)
- Opção de anonimização de nomes de projetos/empresas antes de enviar para LLMs

### 9.3 Configuração de Privacidade
```toml
[privacy]
anonymize_project_names = true  # Substitui "EmpresaX" por "cliente-A"
exclude_file_patterns = ["*.env", "*secret*", "*password*"]
local_only_mode = false         # Se true, não envia para nenhuma API externa
```

---

## 10. Configuração & Deploy

### 10.1 Instalação

```bash
# 1. Instalar a extensão VS Code
# Via marketplace ou .vsix

# 2. Instalar o backend Rust
curl --proto '=https' --tlsv1.2 -sSf https://install.linkedin-dev-companion.dev | sh

# 3. Garantir que o Copilot CLI está instalado e autenticado
# Opção A: via GitHub CLI extension
gh extension install github/gh-copilot
gh auth login

# Opção B: binário standalone
# https://github.com/features/copilot/cli

# Verificar que funciona:
copilot -p "teste" -s

# 4. Configurar credenciais
ldc config set openai.api_key "sk-..."
ldc config set linkedin.client_id "..."
ldc config set linkedin.client_secret "..."

# 5. Iniciar serviço
ldc start
```

### 10.2 Configuração (`~/.ldc/config.toml`)

```toml
[server]
host = "127.0.0.1"
port = 8787

[content]
posting_time = "17:30"           # Horário diário para gerar rascunho
timezone = "America/Sao_Paulo"
variations_to_generate = 3       # Número de rascunhos para revisar
auto_publish = false             # SEMPRE false no MVP — aprovação humana obrigatória
character_limit = 3000           # Limite do LinkedIn

[llm]
openai_api_key = "${OPENAI_API_KEY}"
gpt_model = "gpt-5.4"           # ou "gpt-5.5" para maior qualidade
reasoning_effort = "medium"      # none, low, medium, high, xhigh
embedding_model = "text-embedding-3-large"

[copilot]
enabled = true
cli_path = "copilot"             # comando no PATH, ou caminho absoluto
model = "copilot-latest"         # modelo usado nas chamadas de análise
github_token_env = "COPILOT_GITHUB_TOKEN"  # variável de ambiente com o token

[linkedin]
client_id = "${LINKEDIN_CLIENT_ID}"
client_secret = "${LINKEDIN_CLIENT_SECRET}"
access_token = "${LINKEDIN_ACCESS_TOKEN}"
author_urn = "urn:li:person:..."
api_version = "202506"           # Atualizar conforme versionamento da LinkedIn API (YYYYMM)

[personality]
learning_enabled = true
vector_db_path = "~/.ldc/vectors"
min_examples_required = 3        # Mínimo de exemplos antes de ativar RAG

[privacy]
anonymize = true
excluded_patterns = ["*.env", "*.key", "*secret*"]
```

### 10.3 Comandos CLI

```bash
ldc start              # Inicia o serviço
ldc stop               # Para o serviço
ldc status             # Status do serviço e últimos posts
ldc generate --today   # Força geração de posts para hoje
ldc posts --pending    # Lista posts pendentes de aprovação
ldc approve <id>       # Aprova e publica post pendente
ldc learn "prompt"     # Alimenta manualmente o motor de personalidade
ldc config get <key>   # Lê configuração
ldc config set <key> <value>  # Escreve configuração
```

---

## 📎 Referências

- GitHub Copilot SDK (oficial, Public Preview): https://github.com/github/copilot-sdk — SDKs para Node.js, Python, Go, .NET, Java
- GitHub Copilot CLI — Uso Programático: https://docs.github.com/en/copilot/how-tos/copilot-cli/automate-copilot-cli/run-cli-programmatically
- GitHub Copilot CLI — Referência de flags: https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-programmatic-reference
- Copilot SDK Rust (não-oficial, comunidade): https://github.com/copilot-community-sdk/copilot-sdk-rust
- OpenAI GPT-5.4 — Anúncio: https://openai.com/index/introducing-gpt-5-4/
- OpenAI Models — Catálogo atualizado: https://developers.openai.com/api/docs/models
- OpenAI Responses API (substitui Chat Completions): https://developers.openai.com/api/docs/guides/migrate-to-responses
- LinkedIn Posts API (substitui ugcPosts): https://learn.microsoft.com/en-us/linkedin/marketing/community-management/shares/posts-api
- LinkedIn API Versioning: https://learn.microsoft.com/en-us/linkedin/marketing/versioning
- VS Code Extension API: https://code.visualstudio.com/api

---

*Documento revisado em 2026-05-05 (v1.1.0). Principais mudanças: remoção do Copilot Bridge (Node.js), atualização para Posts API do LinkedIn, correção de limites de tokens GPT-5.4, alinhamento com princípios de autenticidade.*
