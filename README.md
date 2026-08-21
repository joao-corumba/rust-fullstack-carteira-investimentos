# Carteira de Investimentos Inteligente — Rust + Axum

Projeto final do bootcamp **Santander 2026 - Rust AI Developer** (DIO). Este repositório é a
minha evolução do [projeto base](https://github.com/digitalinnovationone/rust-fullstack-carteira-investimentos)
apresentado nas aulas do desafio "Desenvolvendo sua Carteira de Investimentos Inteligente com Rust".

## O que o projeto faz

É uma aplicação Fullstack em Rust para cadastrar e acompanhar ativos de uma carteira de
investimentos. Ela combina:

- uma **API REST** para criar, listar e atualizar ativos (`/api/assets`);
- **persistência em PostgreSQL** via SQLx, com migrations versionadas;
- **autenticação de usuários** (cadastro automático no primeiro login) usando cookies + JWT;
- um **dashboard web** renderizado no servidor com Askama, mostrando os ativos cadastrados e o
  valor total da carteira.

## Tecnologias usadas

- [Rust](https://www.rust-lang.org/) (edition 2024)
- [Axum](https://github.com/tokio-rs/axum) — framework web
- [SQLx](https://github.com/launchbadge/sqlx) + PostgreSQL — persistência
- [Askama](https://github.com/askama-rs/askama) — templates HTML server-side
- [jwt-simple](https://crates.io/crates/jwt-simple) + `axum-extra` cookies — autenticação
- [password-auth](https://crates.io/crates/password-auth) — hashing de senhas
- [insta](https://insta.rs/) — snapshot testing dos endpoints da API
- Docker / `docker compose` — banco de dados local para desenvolvimento

## Melhoria implementada

O projeto base tinha ativos com apenas `name` e `unit_value`, sem nenhuma forma de calcular o
valor total investido. Implementei o seguinte:

1. **Novo campo `quantity` em `assets`** (migration
   `20260801000000_add_asset_quantity`), permitindo registrar quantas unidades de cada ativo a
   pessoa possui.
2. **`Asset::total_value()`** — método que calcula `unit_value * quantity` para um ativo.
3. **Novo endpoint `GET /api/assets/total`** — retorna a soma do valor de todos os ativos da
   carteira (`{"total_value": <f64>}`), calculada a partir de `total_portfolio_value()` no
   `Repository`.
4. **Dashboard reformulado** (`templates/dashboard.html`) — em vez do antigo "Hello, {username}"
   estático, a página inicial agora lista todos os ativos (nome, preço unitário, quantidade e
   valor total de cada um) e destaca o **valor total da carteira** em um card.
5. **`CreateAssetRequest`/`UpdateAssetRequest`** atualizados para aceitar `quantity` ao criar ou
   editar um ativo pela API.
6. **Testes** atualizados/criados: os testes existentes de criar/listar/atualizar ativo agora
   cobrem `quantity`, e foi adicionado `test_total_portfolio_value` cobrindo o novo endpoint.

Todas as consultas SQL (`sqlx::query_as!`) foram verificadas em tempo de compilação contra um
banco PostgreSQL real, e a suíte de testes (`cargo test`) foi executada localmente antes da
entrega.

## Como executar a aplicação

### Pré-requisitos

- Rust (edition 2024 — use a toolchain estável mais recente via [rustup](https://rustup.rs/))
- Docker e Docker Compose (para o PostgreSQL)
- [`sqlx-cli`](https://crates.io/crates/sqlx-cli): `cargo install sqlx-cli --no-default-features --features postgres,rustls`

### Passo a passo

```bash
# 1. Suba o banco de dados
docker compose up -d

# 2. Configure a variável de ambiente (já existe um .env de exemplo no repositório)
# DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres

# 3. Rode as migrations (cria as tabelas assets e users, e adiciona a coluna quantity)
sqlx migrate run

# 4. Rode a aplicação
cargo run
```

A aplicação sobe em `http://localhost:3000`. Acesse `/login` para criar seu usuário (o primeiro
login com um username novo já cadastra a conta automaticamente) e será redirecionado para o
dashboard em `/`.

### Endpoints da API

| Método | Rota                | Autenticação | Descrição                                   |
|--------|---------------------|--------------|------------------------------------------------|
| GET    | `/api/assets`        | —            | Lista todos os ativos                        |
| POST   | `/api/assets`        | Admin        | Cria um novo ativo (`name`, `unit_value`, `quantity`) |
| PATCH  | `/api/assets`        | Admin        | Atualiza um ativo existente                  |
| GET    | `/api/assets/total`  | —            | Retorna o valor total da carteira            |

As rotas de escrita exigem o header `Authorization: im-the-admin` (chave fixa definida no
projeto base, em `src/auth/admin.rs`, apenas para fins didáticos).

## Como testar sua versão

```bash
# com o banco de dados (docker compose up -d) já rodando:
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
cargo test
```

Os testes usam `#[sqlx::test]`, que cria automaticamente um banco de testes isolado por execução
a partir da `DATABASE_URL` configurada, aplica as migrations e roda cada teste em uma transação
própria. Os testes de API usam snapshots (`insta`); se você alterar o formato de resposta, rode
`cargo insta review` para revisar e aceitar os novos snapshots.

Para testar manualmente o endpoint novo:

```bash
curl -X POST http://localhost:3000/api/assets \
  -H "Authorization: im-the-admin" \
  -H "Content-Type: application/json" \
  -d '{"name": "Bitcoin", "unit_value": 350000.0, "quantity": 0.05}'

curl http://localhost:3000/api/assets/total
# {"total_value":17500.0}
```

## O que eu aprendi durante o desafio

- Como o Axum usa extractors (`FromRequestParts`) para compor autenticação/autorização
  (`User`, `Option<User>`, `Admin`) direto na assinatura dos handlers, sem middleware manual
  espalhado pelo código.
- Como o SQLx verifica as queries em tempo de compilação (`query_as!`) contra um banco real, o
  que pega erros de schema (nome de coluna errado, tipo incompatível) antes mesmo de rodar o
  código — inclusive me obrigou a manter a migration, o `struct Asset` e as queries sempre em
  sincronia ao adicionar a coluna `quantity`.
- Como estruturar migrations reversíveis (`.up.sql` / `.down.sql`) com `sqlx-cli` para evoluir um
  schema já em produção sem quebrar dados existentes (usei `DEFAULT 0` na nova coluna
  justamente para isso).
- Como o Askama gera código Rust a partir dos templates HTML em tempo de compilação, então optei
  por formatar os valores monetários (`format!("{:.2}", ...)`) no Rust antes de passar pro
  template, mantendo a lógica de formatação fora do HTML.
- Como testes de snapshot (`insta`) ajudam a pegar mudanças não intencionais no formato de
  resposta da API — tive que atualizar os snapshots existentes ao adicionar o campo `quantity`.

---

Projeto desenvolvido como parte do bootcamp **Santander 2026 - Rust AI Developer** da
[DIO](https://web.dio.me/).
