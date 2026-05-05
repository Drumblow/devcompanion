# Andamento do Projeto

Data: 2026-05-05

## Estado atual

A fase 1 do LinkedIn Dev Companion foi implementada como MVP local-first e publicada no GitHub. A fase 2 tambem foi implementada, adicionando rastreamento Git real, ingestor dedicado, agregacao diaria persistida e dashboard local.

## O que foi criado

- Workspace Rust na raiz com `ldc-core` e `ldc-daemon`.
- `ldc-core` concentra os contratos JSON compartilhados entre extensao e daemon.
- `ldc-daemon` expoe uma API HTTP local com Axum em `127.0.0.1:8787`.
- Persistencia SQLite local em `.ldc/linkedin-dev-companion.db` por padrao.
- Migracoes idempotentes criadas em runtime para eventos, exemplos de voz e rascunhos.
- Gerador local deterministico de rascunhos (`local-template-v1`) para validar o fluxo sem credenciais externas.
- Auditoria minima no registro de cada rascunho, incluindo resumo usado, quantidade de exemplos de voz e obrigatoriedade de aprovacao humana.
- Extensao VS Code com captura de abertura de documento, edicoes com debounce, editor ativo e heartbeat de sessao.
- Comandos da extensao para gerar rascunho, listar pendentes e salvar selecao como exemplo de voz.
- README, `.gitignore` e configuracao base em `config/default.toml`.

## Fase 2 adicionada

- Novo crate `ldc-ingestor` para validar e normalizar eventos antes da persistencia.
- Redacao de caminhos sensiveis como `.env`, arquivos com `secret`, `password`, `token`, `api_key`, `.pem` e `.key`.
- Normalizacao de `event_type`, `session_id`, linguagens, listas de arquivos, bibliotecas e funcoes tocadas.
- Teste unitario cobrindo normalizacao e remocao de metadados sensiveis.
- Tabela `daily_sessions` para materializar a agregacao diaria apos cada evento ingerido.
- Endpoint `GET /events/recent` para inspecionar os ultimos eventos recebidos.
- Endpoint `GET /dashboard/today` com resumo do dia, eventos recentes e rascunhos pendentes.
- `GitWatcher` na extensao para observar o ultimo commit do workspace e enviar eventos `git_commit`.
- `SessionManager.gitContext()` para enriquecer eventos com branch e remoto Git.
- Comando `LinkedIn Dev Companion: Abrir dashboard local` renderizando o snapshot em Markdown.

## Fluxos disponiveis

1. A extensao envia eventos para `POST /events`.
2. O daemon agrega o dia por `GET /sessions/{date}/summary`.
3. O usuario aciona a geracao via extensao ou `POST /posts/generate`.
4. O rascunho fica com status `pending_approval`.
5. O usuario aprova manualmente por `POST /posts/{id}/approve`.

## Endpoints implementados

- `GET /health`: verifica servico e banco.
- `POST /events`: ingere eventos do VS Code.
- `GET /events/recent`: lista eventos recentes ja normalizados e persistidos.
- `GET /dashboard/today`: retorna resumo operacional local para validacao diaria.
- `GET /sessions/{date}/summary`: retorna agregacao diaria.
- `POST /posts/generate`: cria um rascunho local para a data informada ou para hoje.
- `GET /posts/pending`: lista rascunhos aguardando aprovacao.
- `POST /posts/{id}/approve`: aprova um rascunho localmente.
- `POST /personality/examples`: salva exemplos de voz aprovados explicitamente.

## Decisoes importantes

- A fase 1 nao chama OpenAI, Copilot CLI ou LinkedIn. Isso foi proposital para validar tudo sem tokens, custos ou dependencias externas.
- O gerador atual e um template local. Ele deve ser substituido por um provider configuravel na fase de inteligencia.
- A publicacao no LinkedIn permanece desabilitada. O status aprovado e apenas local.
- A extensao ignora falhas silenciosamente durante tracking para nao atrapalhar o usuario se o daemon estiver offline.

## Como validar manualmente

```powershell
cargo check
cargo test
cargo run -p ldc-daemon
```

Em outro terminal:

```powershell
Invoke-RestMethod http://127.0.0.1:8787/health
Invoke-RestMethod http://127.0.0.1:8787/events -Method Post -ContentType 'application/json' -Body '{"session_id":"manual","event_type":"document_edit","project":{"name":"demo","path":"demo"},"activity":{"files_modified":["src/main.rs"],"languages":{"rust":10},"lines_added":12,"lines_removed":3,"time_spent_minutes":10}}'
Invoke-RestMethod http://127.0.0.1:8787/posts/generate -Method Post -ContentType 'application/json' -Body '{}'
Invoke-RestMethod http://127.0.0.1:8787/posts/pending
Invoke-RestMethod http://127.0.0.1:8787/dashboard/today
```

## Validacao executada na fase 2

- `cargo fmt --all`: ok.
- `cargo check`: ok.
- `cargo test`: ok, incluindo teste do `ldc-ingestor`.
- `npm run compile` em `vscode-extension`: ok.
- `GET /health`: ok com o daemon rodando.
- `POST /events` com `event_type` em maiusculas foi normalizado para `document_edit`.
- Arquivos sensiveis `.env` e `config/secret-token.txt` foram removidos antes de persistir.
- Linguagens `Rust` e `TypeScript` foram normalizadas para `rust` e `typescript`.
- `lines_added` negativo foi normalizado para `0`.
- Evento `git_commit` foi persistido com branch `main` e remoto do GitHub.
- `GET /events/recent` retornou os eventos normalizados.
- `GET /dashboard/today` retornou resumo com `event_count = 3`, `git_commits = 1`, linguagens agregadas e nenhum rascunho pendente.

Para a extensao:

```powershell
cd vscode-extension
npm install
npm run compile
```

## Proximos passos recomendados

- Adicionar testes de integracao HTTP para o daemon.
- Persistir edicoes feitas pelo usuario no documento temporario antes de aprovar um rascunho.
- Trocar `local-template-v1` por uma interface real de provider (`LlmProvider`).
- Implementar adaptador Copilot CLI como crate isolado.
- Integrar OpenAI Responses API para geracao final com `reasoning.effort = medium`.
- Adicionar tela de aprovacao com edicao persistida, nao apenas documento temporario.
- Implementar rejeicao com motivo para alimentar o feedback loop.
- Evoluir memoria de voz para score de aderencia e exemplos ranqueados.
