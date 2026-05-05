# Requisitos e Escopo do MVP

## Problema

Desenvolvedores produzem sinais valiosos todos os dias, mas esses sinais se perdem porque transformar trabalho em conteudo exige energia extra, consistencia e contexto historico.

## Proposta

Criar um agente pessoal que acompanhe o trabalho no VS Code, monte memoria de longo prazo sobre o estilo do usuario e produza rascunhos diarios de posts para LinkedIn a partir de contexto real.

## Persona principal

- desenvolvedor que trabalha diariamente no VS Code;
- quer construir presenca publica sem virar produtor de conteudo em tempo integral;
- valoriza autenticidade, contexto tecnico e baixo atrito.

## Requisitos funcionais

### RF-01. Acompanhamento multi-projeto

O sistema deve acompanhar qualquer workspace aberto no VS Code sem depender da estrutura de um repositorio especifico.

### RF-02. Captura de contexto

O sistema deve capturar sinais relevantes do dia de desenvolvimento, como tecnologias em uso, resumos de commits, diffs sintetizados e notas manuais.

### RF-03. Memoria de voz

O sistema deve armazenar exemplos aprovados pelo usuario para aproximar o estilo de escrita real.

### RF-04. Curadoria explicita

O usuario deve conseguir marcar um texto, prompt ou rascunho como exemplo de estilo de forma consciente.

### RF-05. Planejamento diario

O sistema deve decidir se ha material suficiente para gerar conteudo naquele dia.

### RF-06. Geracao de rascunho

O sistema deve gerar pelo menos um rascunho de post com base no contexto do dia e no perfil de voz.

### RF-07. Score de aderencia

O sistema deve pontuar o quanto o rascunho se aproxima do perfil definido pelo usuario.

### RF-08. Aprovacao humana

No MVP, o sistema deve submeter o rascunho para aprovacao antes de qualquer publicacao.

### RF-09. Publicacao desacoplada

O sistema deve tratar a publicacao no LinkedIn como modulo independente do gerador.

### RF-10. Auditoria minima

O sistema deve registrar qual contexto foi usado, qual modelo foi chamado e qual versao do rascunho foi aprovada.

## Requisitos nao funcionais

### RNF-01. Backend em Rust

O nucleo do sistema deve ser implementado em Rust.

### RNF-02. Local-first

O armazenamento deve ser local por padrao.

### RNF-03. Baixo atrito

O uso diario nao deve exigir mais que poucos cliques ou um comando por fluxo principal.

### RNF-04. Portabilidade

O sistema deve funcionar no nivel do usuario da maquina, independente do repositorio aberto.

### RNF-05. Seguranca

Credenciais e tokens devem ser armazenados em cofre seguro do sistema operacional.

## Nao objetivos do MVP

- garantir postagem totalmente automatica desde o primeiro dia;
- depender do plano do GitHub Copilot como API de execucao do produto;
- prometer evasao de detectores de IA;
- ingerir historico privado inteiro sem controle do usuario;
- fazer analise profunda de codigo por upload integral do repositorio.

## Premissas de produto

### Premissa 1. O melhor sinal vem do trabalho real

O sistema gera conteudo melhor quando usa fatos do seu dia, nao templates genericos.

### Premissa 2. O perfil de voz precisa ser construivel

Em vez de tentar inferir tudo automaticamente, o sistema deve permitir curadoria e feedback.

### Premissa 3. Automacao total e etapa posterior

Antes de publicar automaticamente, o produto precisa provar qualidade, consistencia e seguranca operacional.

## Roadmap sugerido

### Fase 0. Descoberta

- definir sinais minimos de captura;
- escolher modelo e interface de provider;
- validar caminho de integracao com LinkedIn.

### Fase 1. MVP local

- extensao do VS Code;
- daemon em Rust;
- memoria local;
- resumo diario;
- um rascunho por dia;
- aprovacao manual.

### Fase 2. Qualidade

- score de estilo;
- variantes de post;
- ranking por qualidade;
- feedback loop de aprovacao e rejeicao.

### Fase 3. Operacao

- fila de agendamento;
- politicas de horario;
- telemetria local;
- publicacao automatica com trilhos.

## Perguntas em aberto

1. Quais sinais do seu dia realmente valem mais para conteudo: commit, diff, issue, prompt, nota manual ou todos?
2. Qual sera a fonte oficial do modelo para o produto em producao?
3. O LinkedIn sera integrado por API oficial, fluxo manual assistido ou automacao de navegador?
4. Qual nivel de aprovacao humana voce quer manter no primeiro mes?
5. Onde o perfil de voz sera alimentado primeiro: posts antigos, notas, prompts ou textos livres?

## Criterios de sucesso do MVP

- gerar pelo menos um rascunho util em dias com atividade relevante;
- reduzir o tempo de transformar trabalho em post;
- manter sensacao de autoria do usuario;
- permitir que o usuario rejeite facilmente um rascunho ruim e melhore o perfil com esse feedback.