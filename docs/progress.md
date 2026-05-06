# Andamento do Projeto

Data: 2026-05-05

## Estado atual

O LinkedIn Dev Companion esta implementado com a fundacao local-first, rastreamento real, analise tecnica opcional via Copilot CLI, sinais Git de diff/status e inicio da fase de aprovacao/publicacao. A entrega chamada anteriormente de fase 4 (`git_snapshot`) foi reclassificada como complemento da fase de tracking/inteligencia. A fase 4 arquitetural agora foi iniciada com publisher LinkedIn desacoplado, desabilitado por padrao, e com aprovacao humana obrigatoria antes de qualquer chamada externa.

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

## Proximos passos recomendados implementados apos a fase 2

- Teste HTTP de integracao do daemon usando o router Axum real.
- Aprovacao de rascunho com persistencia do texto editado pelo usuario.
- Rejeicao de rascunho com motivo em `POST /posts/{id}/reject`.
- Interface `LlmProvider` em `ldc-llm`, com provider `TemplateProvider` e `OpenAiProvider` via Responses API.
- Configuracao de provider por ambiente: `LDC_LLM_PROVIDER`, `LDC_DRAFT_MODEL`, `LDC_REASONING_EFFORT` e `OPENAI_API_KEY`.
- Crate `ldc-copilot` com adaptador de subprocess para `copilot -p ... -s --no-ask-user`.
- Score local simples de estilo gravado em `generated_drafts.style_score`.
- Endpoint `POST /personality/examples/ranked` para recuperar exemplos de voz por similaridade local.

## Avanco de testabilidade antes da fase 3

- Adicionadas configuracoes `.vscode/launch.json` e `.vscode/tasks.json` na raiz e em `vscode-extension/`.
- `F5` agora deve abrir a configuracao `Run LinkedIn Dev Companion Extension` sem cair no seletor generico de debugger.
- Adicionado comando `LinkedIn Dev Companion: Verificar daemon local`.
- Se o daemon estiver offline, a extensao avisa e oferece copiar `cargo run -p ldc-daemon`.
- README atualizado com o fluxo correto: iniciar daemon, abrir Extension Development Host e procurar comandos via `Ctrl+Shift+P`.
- Configuracao de F5 agora abre uma pasta alvo explicita, evitando cair no ultimo workspace usado pelo VS Code.
- Adicionado comando `LinkedIn Dev Companion: Salvar clipboard como exemplo de voz` para capturar prompts/chats copiados conscientemente pelo usuario.
- Decisao de produto registrada: a extensao nao ingere historico do Copilot Chat automaticamente; prompts entram por acao explicita do usuario.

## Fase 3 implementada

- `ldc-copilot` foi integrado ao daemon como analise tecnica diaria opcional.
- Nova configuracao por ambiente: `LDC_COPILOT_ENABLED`, `LDC_COPILOT_CLI_PATH`, `LDC_COPILOT_MODEL` e `LDC_COPILOT_GITHUB_TOKEN_ENV`.
- Copilot fica desabilitado por padrao para evitar consumo inesperado de premium requests.
- `GET /copilot/status` informa se Copilot esta habilitado e disponivel.
- `GET /analysis/today` retorna a analise tecnica do dia.
- `POST /posts/generate` injeta `technical_analysis` no provider e na auditoria do rascunho.
- Quando Copilot esta ausente/desabilitado/falha, o daemon usa fallback local baseado em resumo diario.
- Extensao recebeu comandos `LinkedIn Dev Companion: Verificar Copilot CLI` e `LinkedIn Dev Companion: Ver analise tecnica de hoje`.

## Complemento de tracking/inteligencia: sinais Git reais

- A extensao agora observa tambem o estado do worktree Git, nao apenas o ultimo commit.
- Novo evento normalizado `git_snapshot` para registrar `git diff --shortstat`, arquivos alterados e resumo de `git status --short`.
- O ingestor passou a aceitar `git_snapshot` mantendo a redacao de caminhos sensiveis em `files_modified`.
- `DailySummary` agora inclui `git_changes`, com commits e snapshots recentes do dia.
- A analise tecnica usa `git_changes` como evidencia principal quando existe diff/status capturado.
- O prompt enviado ao Copilot CLI passa a orientar o uso de `git_changes` como fonte de evidencia tecnica.
- O fallback local cita o ultimo sinal Git, arquivos alterados e linhas adicionadas/removidas.
- O dashboard Markdown da extensao ganhou a secao `Sinais Git`.
- A auditoria do rascunho inclui `summary.git_changes` dentro de `context_audit`.

## Fase 4 arquitetural iniciada: aprovacao e publicacao desacoplada

- Novo crate `ldc-linkedin` para isolar a integracao com LinkedIn Posts API.
- O publisher usa `POST https://api.linkedin.com/rest/posts`, header `Linkedin-Version` e `X-Restli-Protocol-Version: 2.0.0`.
- Configuracoes por ambiente: `LDC_LINKEDIN_ENABLED`, `LDC_LINKEDIN_DRY_RUN`, `LDC_LINKEDIN_ACCESS_TOKEN`, `LDC_LINKEDIN_AUTHOR_URN` e `LDC_LINKEDIN_API_VERSION`.
- O publisher fica desabilitado por padrao, evitando qualquer publicacao externa acidental.
- `LDC_LINKEDIN_DRY_RUN=true` permite validar o fluxo sem chamar LinkedIn de verdade.
- Novo endpoint `GET /publisher/status` para diagnostico do publisher.
- Novo endpoint `POST /posts/{id}/publish` para publicar somente rascunhos com `status = approved`.
- Rascunhos ganharam auditoria de publicacao: `published_at`, `linkedin_post_id` e `publication_error`.
- A extensao recebeu comandos `LinkedIn Dev Companion: Verificar publisher LinkedIn` e `LinkedIn Dev Companion: Publicar rascunho aprovado`.
- Apos aprovar um rascunho pela extensao, o usuario pode escolher publicar imediatamente.

## Validacao executada no complemento de sinais Git

- `cargo fmt --all`: ok.
- `cargo check`: ok.
- `cargo test`: ok, incluindo cobertura HTTP para `git_snapshot` em resumo, analise e auditoria.
- `npm run compile` em `vscode-extension`: ok.
- Daemon subiu em `127.0.0.1:8787`.
- `POST /events` com `event_type = git_snapshot`: ok.
- `GET /sessions/2026-05-06/summary`: retornou `git_changes` com `diff_summary`, `status_summary`, arquivos e linhas adicionadas/removidas.
- `GET /analysis/today`: retornou insight citando `Ultimo sinal Git`.
- `POST /posts/generate`: salvou `summary.git_changes` dentro de `context_audit`.

## Validacao executada na fase 4 arquitetural

- `cargo fmt --all`: ok.
- `cargo check`: ok.
- `cargo test`: ok.
- `npm run compile` em `vscode-extension`: ok.
- Teste HTTP atualizado para aprovar rascunho e publicar em modo `linkedin_dry_run`.
- Teste HTTP confirma que rascunho pendente nao pode ser publicado.
- Daemon subiu com `LDC_LINKEDIN_ENABLED=true`, `LDC_LINKEDIN_DRY_RUN=true` e `LDC_LINKEDIN_AUTHOR_URN=urn:li:person:test`.
- `GET /publisher/status`: retornou `enabled = true`, `dry_run = true` e provider `linkedin_dry_run`.
- `POST /posts/{id}/publish` em rascunho pendente: bloqueado com HTTP 400.
- `POST /posts/{id}/approve` seguido de `POST /posts/{id}/publish`: retornou `status = published` e `linkedin_post_id = dryrun-*`.

## Validacao executada na fase 3

- `cargo fmt --all`: ok.
- `cargo check`: ok.
- `cargo test`: ok.
- `npm run compile` em `vscode-extension`: ok.
- `get_errors` do VS Code: sem erros.
- O daemon subiu apos encerrar um processo antigo que bloqueava `target/debug/ldc-daemon.exe` no Windows.
- `GET /copilot/status`: retornou Copilot desabilitado por padrao, sem consumir requests.
- `GET /analysis/today`: retornou analise local com `source = local_fallback` e `status = ok`.
- `POST /posts/generate`: criou rascunho `local-template-v2` com `technical_analysis` salvo em `context_audit`.
- Observacao: a validacao mostrou eventos do workspace `igreja`, confirmando que a extensao acompanha o workspace aberto na Extension Development Host.

## Memoria de voz por acao explicita

- A extensao nao le historico do Copilot Chat automaticamente.
- Para salvar um prompt/chat como voz, o usuario copia o texto e executa `LinkedIn Dev Companion: Salvar clipboard como exemplo de voz`.
- Para salvar texto de um arquivo, o usuario seleciona o trecho e executa `LinkedIn Dev Companion: Salvar selecao como exemplo de voz`.
- Esses textos sao persistidos localmente em `voice_examples` e usados pelo ranking local e pelo provider de rascunhos.

## Fluxos disponiveis

1. A extensao envia eventos para `POST /events`.
2. O `GitWatcher` envia `git_commit` quando encontra novo commit e `git_snapshot` quando o diff/status do workspace muda.
3. O daemon agrega o dia por `GET /sessions/{date}/summary`, incluindo `git_changes`.
4. A analise tecnica vem de `GET /analysis/today` com Copilot opcional ou fallback local.
5. O usuario aciona a geracao via extensao ou `POST /posts/generate`.
6. O rascunho fica com status `pending_approval`.
7. O usuario aprova manualmente por `POST /posts/{id}/approve`.
8. Se o publisher estiver habilitado, um rascunho aprovado pode ser publicado por `POST /posts/{id}/publish`.

## Endpoints implementados

- `GET /health`: verifica servico e banco.
- `POST /events`: ingere eventos do VS Code.
- `GET /events/recent`: lista eventos recentes ja normalizados e persistidos.
- `GET /dashboard/today`: retorna resumo operacional local para validacao diaria.
- `GET /copilot/status`: diagnostica a integracao opcional com Copilot CLI.
- `GET /publisher/status`: diagnostica a integracao opcional com LinkedIn Posts API.
- `GET /analysis/today`: retorna analise tecnica diaria com Copilot ou fallback local.
- `GET /sessions/{date}/summary`: retorna agregacao diaria.
- `POST /posts/generate`: cria um rascunho local para a data informada ou para hoje.
- `GET /posts/pending`: lista rascunhos aguardando aprovacao.
- `POST /posts/{id}/approve`: aprova um rascunho localmente.
- `POST /posts/{id}/reject`: rejeita um rascunho localmente com motivo.
- `POST /posts/{id}/publish`: publica somente rascunhos aprovados, se o publisher estiver habilitado.
- `POST /personality/examples`: salva exemplos de voz aprovados explicitamente.
- `POST /personality/examples/ranked`: lista exemplos de voz ranqueados por similaridade textual local.

## Decisoes importantes

- O modo padrao nao chama OpenAI, Copilot CLI ou LinkedIn. Isso foi proposital para validar tudo sem tokens, custos ou dependencias externas.
- O Copilot CLI e opcional e so roda com `LDC_COPILOT_ENABLED=true`.
- O LinkedIn Publisher e opcional e so roda com `LDC_LINKEDIN_ENABLED=true`.
- O gerador padrao continua sendo um template local, mas ja existe provider OpenAI configuravel por ambiente.
- A publicacao no LinkedIn permanece desabilitada por padrao. O status aprovado e local ate o usuario habilitar explicitamente o publisher.
- A extensao ignora falhas silenciosamente durante tracking para nao atrapalhar o usuario se o daemon estiver offline.
- A fase 4 captura estatisticas e nomes de arquivos do Git, mas nao envia patch completo nem conteudo de arquivo.

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
Invoke-RestMethod http://127.0.0.1:8787/events -Method Post -ContentType 'application/json' -Body '{"session_id":"manual","event_type":"git_snapshot","project":{"name":"demo","path":"demo","git_branch":"main"},"activity":{"files_modified":["src/main.rs"],"lines_added":12,"lines_removed":3},"metadata":{"diff_summary":"1 file changed, 12 insertions(+), 3 deletions(-)","status_summary":"staged: 0, unstaged: 1, untracked: 0"}}'
Invoke-RestMethod http://127.0.0.1:8787/posts/generate -Method Post -ContentType 'application/json' -Body '{}'
Invoke-RestMethod http://127.0.0.1:8787/posts/pending
Invoke-RestMethod http://127.0.0.1:8787/dashboard/today
Invoke-RestMethod http://127.0.0.1:8787/copilot/status
Invoke-RestMethod http://127.0.0.1:8787/publisher/status
Invoke-RestMethod http://127.0.0.1:8787/analysis/today
Invoke-RestMethod http://127.0.0.1:8787/personality/examples -Method Post -ContentType 'application/json' -Body '{"text":"Hoje eu prefiro explicar o trade-off tecnico sem vender solucao magica.","context":"manual"}'
Invoke-RestMethod http://127.0.0.1:8787/personality/examples/ranked -Method Post -ContentType 'application/json' -Body '{"query":"trade-off tecnico"}'
```

## Validacao executada na fase 2

- `cargo fmt --all`: ok.
- `cargo check`: ok.
- `cargo test`: ok, incluindo teste do `ldc-ingestor`.
- Teste HTTP do daemon: ok, cobrindo gerar, aprovar texto editado e rejeitar com motivo.
- `npm run compile` em `vscode-extension`: ok.
- `GET /health`: ok com o daemon rodando.
- `POST /events` com `event_type` em maiusculas foi normalizado para `document_edit`.
- Arquivos sensiveis `.env` e `config/secret-token.txt` foram removidos antes de persistir.
- Linguagens `Rust` e `TypeScript` foram normalizadas para `rust` e `typescript`.
- `lines_added` negativo foi normalizado para `0`.
- Evento `git_commit` foi persistido com branch `main` e remoto do GitHub.
- `GET /events/recent` retornou os eventos normalizados.
- `GET /dashboard/today` retornou resumo com `event_count = 3`, `git_commits = 1`, linguagens agregadas e nenhum rascunho pendente.

## Validacao executada apos os proximos passos recomendados

- `cargo fmt --all`: ok.
- `cargo check`: ok.
- `cargo test`: ok.
- `npm run compile` em `vscode-extension`: ok.
- `get_errors` do VS Code: sem erros.
- Teste HTTP de integracao `http_flow_generates_approves_and_rejects_drafts`: ok.
- Daemon subiu contra o banco local existente e aplicou migracao incremental das colunas novas.
- `POST /personality/examples`: salvou exemplo de voz local.
- `POST /personality/examples/ranked`: retornou exemplo com score `0.29` para consulta `trade-off tecnico com contexto real`.
- `POST /posts/generate`: criou rascunho com `model = local-template-v2` e `style_score = 0.09`.
- `POST /posts/{id}/approve`: persistiu `approved_content` editado manualmente.
- `POST /posts/{id}/reject`: gravou `status = rejected` e `rejection_reason`.
- `GET /dashboard/today`: retornou resumo, eventos recentes e lista de pendentes vazia apos aprovar/rejeitar.

Para a extensao:

```powershell
cd vscode-extension
npm install
npm run compile
```

Depois pressione `F5` e selecione `Run LinkedIn Dev Companion Extension`. Na janela nova, use `Ctrl+Shift+P` e busque `LinkedIn Dev Companion`.

Para alimentar personalidade com prompts do chat, copie o texto do prompt e use `LinkedIn Dev Companion: Salvar clipboard como exemplo de voz` na Extension Development Host.

## Proximos passos recomendados

- Validar manualmente a extensao via Extension Development Host com um workspace Git real e confirmar a secao `Sinais Git` no dashboard.
- Criar UI dedicada de revisao em Webview, em vez de usar documento Markdown temporario.
- Implementar fila/agendador local para gerar rascunhos em horario configurado.
- Adicionar keyring do sistema operacional para tokens externos.
- Adicionar testes E2E da extensao no Extension Development Host.
- Evoluir similaridade local para embeddings reais quando OpenAI estiver configurado.
- Adicionar captura opcional de resumo de PR/issue GitHub quando houver branch remota associada.
