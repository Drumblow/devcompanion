# Arquitetura Proposta

## Resumo executivo

O sistema deve ser composto por dois processos principais:

1. Uma extensao do VS Code, carregada em qualquer workspace aberto.
2. Um servico local em Rust, executado em nivel de usuario da maquina.

Essa separacao resolve dois requisitos ao mesmo tempo:

- o agente acompanha qualquer projeto que voce abrir no editor;
- o agendamento diario e a memoria do sistema continuam existindo fora de um repositorio especifico.

## Por que nao rodar apenas dentro do VS Code

Se tudo rodar apenas dentro da extensao, o produto depende do editor estar aberto para agendar, consolidar memoria e eventualmente publicar. Isso enfraquece a confiabilidade operacional.

Por isso, a recomendacao e:

- usar a extensao para captura de contexto e interacao com o usuario;
- usar o daemon em Rust para estado persistente, processamento e agendamento.

## Topologia

```text
VS Code Extension
    |
    | local IPC / HTTP local / named pipe
    v
Rust Daemon
    |
    +--> SQLite / vetor / arquivos locais
    +--> Adaptador de modelo
    +--> Scheduler diario
    +--> Publicador LinkedIn
```

## Componentes detalhados

### 1. VS Code Extension

Responsabilidades:

- identificar workspace atual, branch, stack e sinais do projeto;
- resumir alteracoes recentes do Git;
- capturar eventos relevantes do editor;
- oferecer comandos como "salvar como exemplo da minha voz";
- mostrar rascunhos e pedir aprovacao.

Sinais que valem coletar:

- nomes de arquivos e tecnologias em uso;
- mensagens de commit;
- diffs resumidos e nunca o codigo inteiro por padrao;
- nome do repositorio e categoria do projeto;
- notas manuais do usuario;
- prompts ou textos marcados explicitamente para o perfil.

Sinais que nao devem ser coletados por padrao:

- segredos;
- conteudo integral de arquivos sensiveis;
- historico completo de conversa privada sem consentimento explicito.

### 2. Rust Daemon

Responsabilidades:

- manter identidade unica do usuario;
- armazenar memoria de contexto entre projetos;
- consolidar o diario de trabalho;
- gerar briefs e rascunhos;
- agendar jobs;
- entregar saida para aprovacao ou publicacao.

Stack sugerida:

- runtime: `tokio`;
- API local: `axum`;
- serializacao: `serde`;
- persistencia: `sqlx` com SQLite;
- jobs: scheduler simples no proprio processo;
- observabilidade: `tracing`.

### 3. Voice Profile Engine

Objetivo: aproximar o sistema da sua voz real sem depender de promessas artificiais.

Entradas:

- posts antigos escritos por voce;
- trechos aprovados por voce;
- frases preferidas e frases proibidas;
- nivel de tecnicidade desejado;
- tom por contexto: reflexivo, pratico, tutorial, bastidor.

Saidas:

- guia de estilo estruturado;
- exemplos ranqueados por similaridade;
- blacklist de cliches;
- score de aderencia ao perfil.

Estrutura minima do perfil:

```json
{
  "tone_tags": ["pratico", "honesto", "tecnico"],
  "preferred_patterns": [
    "abrir com contexto real do trabalho",
    "fechar com aprendizado ou trade-off"
  ],
  "avoid_patterns": [
    "frases genericas de produtividade",
    "promessas grandiosas"
  ],
  "approved_examples": []
}
```

### 4. Content Planner

Nem todo dia precisa gerar post. O planner decide se existe material suficiente.

Exemplos de gatilhos validos:

- uma decisao tecnica relevante;
- um bug interessante resolvido;
- um aprendizado concreto de arquitetura;
- um experimento com resultado mensuravel;
- um padrao recorrente observado em varios projetos.

Se nao houver sinal forte, o sistema deve:

- sugerir rascunho curto;
- gerar apenas um insight interno;
- ou pular o dia.

### 5. Adaptador de Modelo

O modelo deve entrar por interface, nao por dependencia acoplada.

Interface minima:

```rust
pub trait LlmProvider {
    async fn generate_draft(&self, input: DraftInput) -> anyhow::Result<DraftOutput>;
    async fn score_style(&self, input: StyleScoreInput) -> anyhow::Result<StyleScoreOutput>;
    async fn summarize_day(&self, input: DaySummaryInput) -> anyhow::Result<DaySummaryOutput>;
}
```

Decisao importante:

- o design pode prever GPT-5.4 com reasoning medio como configuracao desejada;
- a implementacao deve usar um provedor configuravel via API oficial.

### 6. LinkedIn Publisher

Responsabilidades:

- montar payload final;
- validar limites de tamanho;
- anexar links quando existirem;
- publicar ou agendar;
- registrar sucesso, falha e tentativa.

Recomendacao de rollout:

1. Comecar por exportacao de rascunho.
2. Depois aprovar e publicar manualmente.
3. So depois avaliar postagem automatica.

## Fluxo de dados diario

```text
Editor activity
    -> Extension event capture
    -> Daily work log
    -> Signal ranking
    -> Brief generation
    -> Draft generation
    -> Style scoring
    -> User approval
    -> Publish / Schedule
```

## Modelo de dados inicial

Tabelas sugeridas:

- `workspaces`
- `work_events`
- `daily_briefs`
- `voice_examples`
- `style_rules`
- `drafts`
- `publish_jobs`
- `provider_settings`

Campos importantes:

- origem do contexto;
- nivel de confianca;
- hash do artefato de origem;
- status do draft;
- motivo de rejeicao ou aprovacao.

## Privacidade e seguranca

Requisitos obrigatorios:

- armazenamento local por padrao;
- opt-in para qualquer sincronizacao externa;
- mascaramento de segredos;
- trilha de auditoria para o que foi enviado ao modelo;
- configuracao para excluir repositorios ou pastas.

## Riscos principais

### Risco 1. Dependencia do Copilot como se fosse API

Mitigacao: isolar a camada de modelo e tratar Copilot apenas como ferramenta de desenvolvimento, nao como backend do produto.

### Risco 2. Conteudo generico

Mitigacao: exigir sinais reais do trabalho, exemplos aprovados e score de aderencia ao perfil.

### Risco 3. Automacao precoce demais

Mitigacao: operar com aprovacao humana ate haver historico suficiente.

### Risco 4. Publicacao fragil no LinkedIn

Mitigacao: manter o publicador isolado e intercambiavel.

## Decisao recomendada

Construa primeiro um sistema local-first com extensao + daemon em Rust + aprovacao humana. Se isso funcionar bem, a automacao de postagem vira um detalhe operacional, nao o coracao do produto.