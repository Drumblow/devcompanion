# LinkedIn Dev Companion

MVP local-first para acompanhar atividade no VS Code, registrar memoria local e gerar rascunhos revisaveis de posts tecnicos para LinkedIn.

## Fase 1 implementada

- Daemon local em Rust com API HTTP em `127.0.0.1:8787`.
- SQLite local para eventos, exemplos de voz, rascunhos e auditoria minima.
- Extensao VS Code em TypeScript para capturar sinais basicos do editor.
- Fluxo manual de geracao, visualizacao e aprovacao de rascunhos.

## Fase 2 implementada

- Ingestor Rust dedicado para normalizar eventos e remover sinais sensiveis de caminhos/metadados.
- Agregacao diaria persistida em SQLite na tabela `daily_sessions`.
- Captura de contexto Git na extensao: branch, remoto e ultimo commit observado.
- Endpoint de dashboard local com resumo do dia, eventos recentes e rascunhos pendentes.
- Comando da extensao para abrir o dashboard local em Markdown.

## Proximos passos recomendados implementados

- Teste HTTP de integracao cobrindo ingestao, geracao, aprovacao editada e rejeicao.
- Aprovacao na extensao persiste o texto editado no documento temporario.
- Rejeicao de rascunho com motivo para alimentar o feedback loop.
- Interface `LlmProvider` com provider local `template` e provider OpenAI Responses API configuravel.
- Adaptador Copilot CLI isolado em crate proprio.
- Score simples de aderencia ao perfil de voz e exemplos ranqueados por similaridade local.

## Como rodar

```powershell
cargo run -p ldc-daemon
```

Em outro terminal:

```powershell
cd vscode-extension
npm install
npm run compile
```

Depois abra a pasta `vscode-extension` no VS Code e use `F5` para iniciar uma Extension Development Host.

## Endpoints principais

- `GET /health`
- `POST /events`
- `GET /events/recent`
- `GET /dashboard/today`
- `GET /sessions/{date}/summary`
- `POST /posts/generate`
- `GET /posts/pending`
- `POST /posts/{id}/approve`
- `POST /posts/{id}/reject`
- `POST /personality/examples`
- `POST /personality/examples/ranked`

## Configuracao de providers

Por padrao o projeto usa `LDC_LLM_PROVIDER=template`, sem tokens externos.

Para testar geracao via OpenAI Responses API:

```powershell
$env:LDC_LLM_PROVIDER = "openai"
$env:LDC_DRAFT_MODEL = "gpt-5.4"
$env:LDC_REASONING_EFFORT = "medium"
$env:OPENAI_API_KEY = "sk-..."
cargo run -p ldc-daemon
```

O adaptador Copilot CLI ja existe como crate isolado (`ldc-copilot`) para a proxima etapa de analise tecnica via subprocess.

Consulte [docs/progress.md](docs/progress.md) para o handoff detalhado.
