# Pesquisa 01 — Projeto de referência `solana`

> Exploração de `/home/jonatas/projects/github/solana` (jun/2026). Apesar do nome, **não é blockchain** — é um **app desktop nativo** (coletor/medidor de oportunidades) com GUI em egui/eframe. É a referência de arquitetura, lints e UI do nosso formatador.

## 1. Stack e UI

**GUI: egui + eframe (immediate-mode, nativo) v0.29.**
- `crates/ui/Cargo.toml:29-36`: `eframe = "0.29"` (features `default_fonts`, `glow`, `x11`), `egui`, `egui_extras`, `egui_plot`. Replicado no composition root `crates/app/Cargo.toml`.
- ADR da escolha: `docs/arquitetura/decisoes/0009-ui-desktop-egui.md`.
- `default-features = false` para cortar image loaders/http/acessibilidade; **só backend X11** (sem Wayland) porque smithay/wayland exige edition2024/Rust ≥ 1.85. Travado em 0.29 por ser "a última linha que compila em rustc 1.84.1" (`crates/ui/Cargo.toml:22-33`). MSRV efetivo 1.84.1 (informal, sem `rust-version`).
- **Não é web-based** (sem Tauri/Dioxus/Leptos/Slint). Node/npm só para spellcheck (cspell).

**Frontend "fonte de verdade" = mockup HTML/CSS estático**: `design/desktop-mockup/` (`index.html`, etc. + `assets/theme.css`). O código Rust traduz o CSS fielmente — `crates/ui/src/theme/palette.rs:1-66` cita cada token CSS (ex.: `--bg #0B0E14`, `--accent #9945FF`). **→ é exatamente o papel que nossos protótipos HTML cumprem.**

**Tema: só ESCURO (dark-only), sem toggle.**
- `crates/ui/src/theme/theme_kit.rs:35` parte de `Visuals::dark()` e sobrescreve com a `Palette`.
- Instalação única: `ThemeKit::install(&ctx)` no construtor de `RoiApp` (`crates/ui/src/app.rs:75`).
- Fontes embarcadas (`crates/ui/assets/*.ttf`): Inter (UI), JetBrains Mono (números), Phosphor (ícones) + `RoiSymbols.ttf`. Cores semânticas centralizadas em `theme_kit.rs:68-82`.

## 2. Arquitetura

**Workspace Cargo multi-crate = Hexagonal (Ports & Adapters).** Regra de dependência **imposta pelo grafo de crates** (`Cargo.toml:6-19`).

Membros (`Cargo.toml:8-15`):
- `crates/domain` — núcleo puro. Só `thiserror`/`rand`. Zero conhecimento de outros crates.
- `crates/application` — casos de uso + **portas (traits)**. Depende só de `domain`. Subdirs `ports/`, `use_cases/`, `bots/`, `strategies/`.
- `crates/infrastructure` — adapters concretos (Solana WS, Jupiter HTTP/reqwest, SeaORM/SQLite). Vê `domain`+`application`, nunca o contrário.
- `crates/app` — **composition root**; único que conhece todos. Binários: `medidor` (CLI), `roi-live` (UI, `default-run`), `roi-send`, `roi-swap`.
- `crates/ui` — **driving adapter / presenter** (egui). Depende só de `domain`+`application`, **nunca** de `infrastructure`.
- `crates/migration` — migrations SeaORM.
- `tools/xtask` — ferramenta de build **fora** do workspace de produção (workspace próprio), para os `members` serem exatamente as camadas.

**Separação UI vs lógica vs IO:**
- UI fala com o domínio só por **portas injetadas como `Arc<dyn Trait>`**. Campos de `RoiApp` em `crates/ui/src/app.rs:30-70`: `ui_state: Arc<dyn UiState>`, `arming: Option<Arc<dyn ArmingControl>>`, etc.
- **Builder de injeção**: `RoiApp::new` cria com adapters in-memory (stub); `with_data_source`/`with_arming`/... trocam por implementações reais (`app.rs:103-140`). Permite rodar UI com dados falsos (preview/screenshots) ou reais.
- Organização da UI: `screens/`, `layout/`, `widgets/`, `theme/`. `update()` desenha painéis fixos e delega o `CentralPanel` à tela ativa.

**Erros — política em camadas:**
- Libs/núcleo: **`thiserror`** (enums tipados por camada).
- Composition root (`app`): **`anyhow`**.
- **Sem-pânico em produção** (lint): `unwrap`/`expect`/`panic!` proibidos. Boot trata erro com `match`+`eprintln!`+`exit(1)` (`roi_live/main.rs:40-60`); criador do eframe devolve `Err` tipado (`roi_live/window.rs:74-78`).

**Async & estado:**
- **Tokio** (não async-std). Cada camada liga só as features que precisa.
- **Ponte async→UI (molde p/ nosso progresso de gravação):** runtime tokio criado e "entered" no `main`; feeds rodam como tasks em background; UI síncrona lê o estado via portas. Ordem em `roi-live/main.rs:53-115`: cria runtime → DB com `block_on` → `runtime.enter()` → compõe `LiveSource` (spawna tasks) → `Window::run` bloqueia a main thread. UI repinta com `ctx.request_repaint_after(500ms)` (`app.rs:171-173`). Adapters que chamam async no frame recebem `runtime.handle().clone()`.
- **Shutdown cooperativo**: `tokio-util` `CancellationToken`; drain das filas antes do drop do runtime.
- `async-trait` para portas assíncronas.

**Padrões:** Hexagonal; DIP; Strategy (`OpportunityDetector` com 2 impls); Data Mapper (entidades SeaORM confinadas na infra); Value Objects (`Lamports`, `Price`, `BasisPoints`). Docs em `docs/arquitetura/02..05,10`.

## 3. Lints e qualidade — **o coração a replicar**

`[workspace.lints.clippy]` (`Cargo.toml:43-71`):
- `all = { level = "deny", priority = -1 }`
- `pedantic = { level = "warn", priority = -1 }` (nursery OFF)
- **Sem-pânico**: `unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`.
- `allow`s conscientes: `module_name_repetitions`, `must_use_candidate`, `return_self_not_must_use`, `struct_field_names`, `single_match_else`, `too_many_lines`, `doc_markdown`, `missing_errors_doc`; casts (`cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, `cast_possible_wrap`, `cast_lossless`); `float_cmp`.

`[workspace.lints.rust]` (`Cargo.toml:79-104`) — todos como erro:
- `unreachable_pub`, `unnameable_types`, `missing_docs`, `single_use_lifetimes`, `unused_lifetimes`, `trivial_numeric_casts`, `meta_variable_misuse`, `explicit_outlives_requirements`, `unused_qualifications`, `trivial_casts`, `redundant_imports` = `deny`.
- **`unsafe_code = "forbid"`** (mais forte que deny; nem `#[allow]` local reabilita).

Cada crate herda via `[lints] workspace = true`. `tools/xtask` repete manualmente (workspace separado).

**`clippy.toml` (raiz)** — conteúdo exato:
```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
```
A isenção de teste vem do `clippy.toml` (escopo `#[cfg(test)]`), **não** de `#[allow]` no código.

**`rustfmt.toml`: NÃO EXISTE** (rustfmt default; CI roda `cargo fmt --all -- --check`).
**`rust-toolchain.toml`: NÃO EXISTE** (CI usa `dtolnay/rust-toolchain@stable`). **Edição: 2021.**

**`deny.toml` (raiz)** — 4 checks: `[advisories]` (v2, só ignora transitivos irredutíveis com `id`+`reason`), `[licenses]` (allowlist permissiva + exceção das fontes egui `OFL-1.1`), `[bans]` (`multiple-versions = "warn"`), `[sources]` (só crates.io).

**CI — `.github/workflows/ci.yml`**: workspace inteiro em todo PR, `RUSTFLAGS: "-D warnings"`, **8 jobs paralelos**: `fmt`, `clippy` (`--all-targets -- -D warnings`), `test`, `spell` (cspell), `line-limit` (xtask), `machete` (deps não usadas), `deny`, `doc` (`RUSTDOCFLAGS="-D warnings"`). Usa `Swatinem/rust-cache@v2`. **Sem pre-commit / Makefile / justfile** — a ferramenta é `cargo xtask`.

**Lint customizado de tamanho (xtask):** `tools/xtask/src/line_limit/rule.rs:37` define `LIMIT = 200` → **máx. 199 linhas por `.rs`**. (`crates/ui/src/app.rs` tem exatamente 199.)

## 4. Convenções (`docs/arquitetura/01-principios-e-estilo.md`)

- **OOP estrito: SEM função livre.** Toda função é método/associated fn de um `struct`/`enum`; única exceção é `fn main`. Helpers viram associated fn de unit structs (`Clock::now_unix()`, `LineLimitRule::check()`).
- **Verbosidade desejada**: nomes longos descritivos; Value Objects no lugar de primitivos.
- **Encapsulamento**: nenhum campo `pub` em struct própria; campos privados + getters; visibilidade mais restrita (`pub(crate)`/`pub(super)`). Exceção: `Model` do SeaORM.
- **Arquivos pequenos, um conceito por arquivo**; `mod.rs` magro que só reexporta.
- **Testes em arquivo irmão**: `foo.rs` tem só `#[cfg(test)] mod tests;`; os testes vão em `foo/tests.rs` com `use super::*;`. Integração caixa-preta em `crates/<crate>/tests/`.
- **Features Cargo** para opções pesadas/opcionais (ex.: feature `grpc` default OFF).
- **Deps centralizadas** em `[workspace.dependencies]`; cada crate liga só o que pode usar (reforço de fronteira).
- **Docs** em `docs/arquitetura/` numerada (01–11 + `decisoes/` ADRs). Comentários em **pt-BR** explicando o "porquê", não narrando a linha.

## Síntese para o nosso projeto
Reaproveitar: **workspace hexagonal** (domain → application/portas → infrastructure → app → ui egui), **egui nativo espelhando mockup CSS**, **injeção `Arc<dyn Trait>` + builder** (stub/real), **bridge tokio→egui** (`request_repaint_after`, `runtime.handle()`). Copiar quase verbatim o **pacote de qualidade** (lints sem-pânico, `unsafe=forbid`, `clippy.toml`, `deny.toml`, CI 8 jobs, xtask line-limit, testes em arquivo irmão).
**Atenção:** (a) gravar em pendrive exige IO de baixo nível/privilégio e o `solana` é read-only com `unsafe=forbid` — usaremos caminhos seguros (udisks/zbus) e isolaremos FFI quando inevitável; (b) o `solana` é X11-only por MSRV 1.84.1 — vamos modernizar (egui novo + Wayland).
