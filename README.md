# LinkedIn Dev Companion

MVP local-first para acompanhar atividade no VS Code, registrar memoria local e gerar rascunhos revisaveis de posts tecnicos para LinkedIn.

## Fase 1 implementada

- Daemon local em Rust com API HTTP em `127.0.0.1:8787`.
- SQLite local para eventos, exemplos de voz, rascunhos e auditoria minima.
- Extensao VS Code em TypeScript para capturar sinais basicos do editor.
- Fluxo manual de geracao, visualizacao e aprovacao de rascunhos.

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
- `GET /sessions/{date}/summary`
- `POST /posts/generate`
- `GET /posts/pending`
- `POST /posts/{id}/approve`
- `POST /personality/examples`

Consulte [docs/progress.md](docs/progress.md) para o handoff detalhado.
