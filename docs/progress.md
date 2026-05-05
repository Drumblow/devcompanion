# Andamento do Projeto

Data: 2026-05-05

## Estado atual

A fase 1 do LinkedIn Dev Companion foi implementada como MVP local-first. O projeto saiu de uma especificacao docs-first para um scaffold executavel com daemon Rust, banco SQLite local e extensao VS Code em TypeScript.

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

## Fluxos disponiveis

1. A extensao envia eventos para `POST /events`.
2. O daemon agrega o dia por `GET /sessions/{date}/summary`.
3. O usuario aciona a geracao via extensao ou `POST /posts/generate`.
4. O rascunho fica com status `pending_approval`.
5. O usuario aprova manualmente por `POST /posts/{id}/approve`.

## Endpoints implementados

- `GET /health`: verifica servico e banco.
- `POST /events`: ingere eventos do VS Code.
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
```

Para a extensao:

```powershell
cd vscode-extension
npm install
npm run compile
```

## Proximos passos recomendados

- Adicionar testes de integracao HTTP para o daemon.
- Implementar deteccao de branch/remoto Git na extensao.
- Trocar `local-template-v1` por uma interface real de provider (`LlmProvider`).
- Implementar adaptador Copilot CLI como crate isolado.
- Integrar OpenAI Responses API para geracao final com `reasoning.effort = medium`.
- Adicionar tela de aprovacao com edicao persistida, nao apenas documento temporario.
- Implementar rejeicao com motivo para alimentar o feedback loop.
- Evoluir memoria de voz para score de aderencia e exemplos ranqueados.
