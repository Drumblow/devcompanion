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

## Fase 3 implementada

- Analise tecnica diaria via `ldc-copilot`, opcional e desabilitada por padrao.
- Fallback local quando Copilot CLI esta desabilitado, ausente ou falha.
- Endpoint `GET /copilot/status` para diagnostico.
- Endpoint `GET /analysis/today` para ver a analise tecnica usada pelo rascunho.
- Rascunhos passam a receber `technical_analysis` na auditoria e no contexto do provider.
- Comandos da extensao para verificar Copilot CLI e abrir a analise tecnica de hoje.

## Complemento de tracking/inteligencia

- A extensao captura snapshots de Git quando `git diff` ou `git status` mudam no workspace.
- Novo evento `git_snapshot` registra arquivos alterados, linhas adicionadas/removidas, resumo de diff e resumo de status.
- `DailySummary` inclui `git_changes`, usado pela analise tecnica e salvo na auditoria dos rascunhos.
- O fallback local e o prompt do Copilot passam a priorizar sinais reais de Git quando disponiveis.
- Dashboard Markdown mostra a secao `Sinais Git` para inspecionar commits e snapshots recentes.

## Fase 4 arquitetural iniciada

- Novo crate `ldc-linkedin` isola a integracao com LinkedIn Posts API.
- Endpoint `GET /publisher/status` diagnostica a configuracao de publicacao.
- Endpoint `POST /posts/{id}/publish` publica apenas rascunhos previamente aprovados.
- Publicacao fica desabilitada por padrao e pode ser testada com `LDC_LINKEDIN_DRY_RUN=true`.
- Rascunhos persistem `published_at`, `linkedin_post_id` e `publication_error`.
- A extensao permite verificar o publisher e publicar um rascunho aprovado.

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

Se voce apertar `F5` dentro de `vscode-extension`, a Extension Development Host abre carregando a pasta pai `Automatatizador`. Se apertar `F5` na raiz do projeto, ela abre a propria raiz. A janela nova que abrir e a Extension Development Host; nela use `Ctrl+Shift+P` e procure por `LinkedIn Dev Companion`.

Comandos uteis na Extension Development Host:

- `LinkedIn Dev Companion: Verificar daemon local`
- `LinkedIn Dev Companion: Gerar rascunho de hoje`
- `LinkedIn Dev Companion: Ver rascunhos pendentes`
- `LinkedIn Dev Companion: Abrir dashboard local`
- `LinkedIn Dev Companion: Salvar selecao como exemplo de voz`
- `LinkedIn Dev Companion: Salvar clipboard como exemplo de voz`
- `LinkedIn Dev Companion: Verificar Copilot CLI`
- `LinkedIn Dev Companion: Ver analise tecnica de hoje`
- `LinkedIn Dev Companion: Verificar publisher LinkedIn`
- `LinkedIn Dev Companion: Publicar rascunho aprovado`

## Como a extensao deve funcionar

Durante desenvolvimento da propria extensao, `F5` abre uma segunda janela do VS Code com a extensao carregada temporariamente. Essa janela e apenas um ambiente de teste.

No uso real, a extensao deve ser empacotada e instalada uma vez. Depois disso, voce abre qualquer projeto normalmente no VS Code e ela acompanha aquele workspace, enviando eventos para o daemon local.

O daemon Rust e quem guarda a memoria local. A extensao captura sinais do projeto aberto, como arquivos editados, linguagens, branch/remoto Git e commits observados. Os exemplos de voz sao salvos explicitamente pelo usuario, por selecao, input manual ou clipboard.

Na fase 4, o `GitWatcher` tambem observa o worktree e envia `git_snapshot` quando o resumo de `git diff` ou `git status` muda. O evento contem estatisticas e nomes de arquivos normalizados, mas nao envia patch completo nem conteudo de arquivo.

Por privacidade e limitacao da API publica do VS Code, a extensao nao le automaticamente o historico do Copilot Chat. Para usar seus prompts como personalidade, copie o texto do chat e rode `LinkedIn Dev Companion: Salvar clipboard como exemplo de voz`, ou selecione um texto em um arquivo e rode `LinkedIn Dev Companion: Salvar selecao como exemplo de voz`.

## Endpoints principais

- `GET /health`
- `POST /events`
- `GET /events/recent`
- `GET /dashboard/today`
- `GET /copilot/status`
- `GET /publisher/status`
- `GET /analysis/today`
- `GET /sessions/{date}/summary`
- `POST /posts/generate`
- `GET /posts/pending`
- `POST /posts/{id}/approve`
- `POST /posts/{id}/reject`
- `POST /posts/{id}/publish`
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

O adaptador Copilot CLI existe como crate isolado (`ldc-copilot`) e e usado pela analise tecnica quando habilitado explicitamente.

Para testar a fase 3 com Copilot CLI, deixe o CLI instalado e habilite explicitamente:

```powershell
$env:LDC_COPILOT_ENABLED = "true"
$env:LDC_COPILOT_CLI_PATH = "copilot"
$env:LDC_COPILOT_MODEL = "copilot-latest"
cargo run -p ldc-daemon
```

Sem essas variaveis, a analise usa fallback local e nao consome requests do Copilot.

Para testar o fluxo de publicacao sem chamar LinkedIn de verdade:

```powershell
$env:LDC_LINKEDIN_ENABLED = "true"
$env:LDC_LINKEDIN_DRY_RUN = "true"
$env:LDC_LINKEDIN_AUTHOR_URN = "urn:li:person:test"
cargo run -p ldc-daemon
```

Para usar a LinkedIn Posts API real, configure tambem `LDC_LINKEDIN_ACCESS_TOKEN` com scope `w_member_social` e mantenha `LDC_LINKEDIN_DRY_RUN=false`.

Consulte [docs/progress.md](docs/progress.md) para o handoff detalhado.
