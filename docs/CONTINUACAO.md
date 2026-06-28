# Continuação — estado do trabalho (ponto de retomada)

> Snapshot para retomar caso caia a energia. Atualizado em 2026-06-27.
> Este arquivo está **commitado e no GitHub**, então sobrevive a qualquer queda.

## Onde estamos AGORA

- **Repositório:** `https://github.com/jhonatasfender/nur` (local: `/home/jonatas/projects/github/format-bootpendrive`)
- **Branch atual:** `fase3-enumeracao`
- **Último commit:** `4e0ed3b` (working tree limpo, sincronizado com `origin/fase3-enumeracao`)
- **Tudo verde:** `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (21 testes), `cargo fmt`, `cargo doc -D warnings`, `cargo xtask check`, `cargo machete`, `cargo deny`.

## O que já está pronto e MERGEADO em `main`

- **Fase 1 — Fundação** (PR #1, merged): workspace hexagonal (`domain` → `application` → `infrastructure` → `app` → `ui` + `tools/xtask`), lints exigentes (espelham o `solana`), CI + Release, binário `nur`.
- **Fase 2 — UI completa** (PR #2, merged): painel fiel ao protótipo, tema claro/escuro (travado via `set_visuals_of`+`set_theme`), fonte Inter embutida, janela arredondada sem barra nativa + auto-resize, modo em pílula animada, modal "digite APAGAR", captura por F12/botão, componentes reutilizáveis (`crates/ui/src/components/`).

## O que está na branch `fase3-enumeracao` (AINDA SEM PR)

1. **Revisão de arquitetura (OOP + hexagonal)** — commits `3217787`, `0b820bd`, `af2ddfa`:
   - Sem funções livres (exceto `main`): `Window::open`, `struct Xtask`.
   - IO confinado na `infra`: porta `application::ports::ScreenshotWriter` + `infrastructure::screenshot::PngScreenshotWriter`; leitura de env subiu pro composition root (`Window` + builders `NurApp::with_*`); demo em `crates/ui/src/app/demo.rs`. `ui` NÃO depende de `infrastructure` nem de `image`.
2. **Plano 3 — Incremento 1: enumeração real (Linux)** — commits `4692e15`, `713cd3b`, `4e0ed3b`:
   - `DiskService` **assíncrono** (`async-trait`); `DeviceListState` (Loading/Ready/Error); `ListDevices::execute` async.
   - **`SysfsDiskService`** (`crates/infrastructure/src/linux/sysfs_disk_service.rs`): lê `/sys/block`, filtra USB (caminho canônico contém `/usb`), retorna `Device`s. Rápido, sem D-Bus.
   - **Ponte tokio→egui** (`crates/app/src/window.rs` `LiveUiState`): task de polling (1,5s) num `Arc<RwLock<DeviceListState>>` + `egui_ctx.request_repaint()`.
   - UI (`device_selector` em `crates/ui/src/app/draw.rs`) renderiza por estado.
   - Validado por screenshot headless: lista **1 dispositivo USB real** da máquina, rápido.

### Pivot importante (decisão registrada)
Comecei com **udisks2/zbus** (ADR 0004), mas era **~1s por chamada D-Bus** (~7s por listagem — UX ruim). Tentei: cache do client, feature `tokio` do zbus, `async-io` nativo via `spawn_blocking` — nada resolveu. **Pivotei a enumeração para sysfs** (rápido). **udisks2 fica para o incremento de gravação** (precisa do polkit `Block.OpenDevice`, e opera sobre 1 device só). Documentado em `docs/superpowers/specs/2026-06-27-plano3-enumeracao-real-linux-design.md` (§6.1).

## PRÓXIMOS PASSOS (em ordem)

1. **Abrir a PR da Fase 3** (review de arquitetura + enumeração real):
   ```bash
   cd /home/jonatas/projects/github/format-bootpendrive
   gh pr create --base main --head fase3-enumeracao --title "Fase 3: review arquitetura (OOP/hexagonal) + enumeração real (sysfs)" --body "..."
   ```
   Conferir CI verde (`gh pr checks <n>`), depois mergear.
2. **Incremento 2 — gravação (o grande)**: raw write da ISO + `IsoInspector` (isohybrid vs Windows) + udisks2/polkit (`Block.OpenDevice`) + progresso real ligado à barra + seletor de arquivo ISO real (`rfd`). Antes: o **spike** do ADR 0006 (validar FAT32 + split do `install.wim` + boot em VM). Fazer brainstorming → spec → plano → executar.

## Como retomar / comandos úteis

```bash
cd /home/jonatas/projects/github/format-bootpendrive
git status && git log --oneline -5
cargo build --bin nur && cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask check   # line-limit (199) + pub-fields (zero campos pub)
```

**Rodar a UI (janela real):** `cargo run --bin nur` (tem display; F12 ou o botão de câmera salvam print).

**Validar UI headless (eu uso isto):**
```bash
NUR_CAPTURE=/tmp/nur.png NUR_DEMO=ready LIBGL_ALWAYS_SOFTWARE=1 WINIT_UNIX_BACKEND=x11 \
  timeout 60 xvfb-run -a -s "-screen 0 900x1000x24" ./target/debug/nur
```
- Envs de validação (lidas no composition root): `NUR_CAPTURE=<png>` (captura headless), `NUR_THEME=light`, `NUR_DEMO=ready|modal|running|format`.

## Convenções do projeto (IMPORTANTES — para sessão nova)

- **Arquitetura hexagonal**: `domain` puro; `application` = portas+casos de uso; `infrastructure` = todo o IO; `app` = composition root; `ui` = presenter, sem IO e sem depender de `infrastructure`.
- **OOP estrito**: sem função livre exceto `fn main` (helpers viram associated fn de struct). Enforçado em review.
- **Código em inglês**; comentários, logs e textos de UI em **pt-BR**.
- **Zero campos `pub`** (getters) — enforçado por `cargo xtask pub-fields`.
- **Máx 199 linhas por `.rs`** — enforçado por `cargo xtask line-limit`.
- Sem `unsafe` (`unsafe_code = forbid`); sem `unwrap`/`expect`/`panic` fora de testes; `missing_docs`/`unreachable_pub` = erro; testes em arquivo irmão (`foo.rs` → `foo/tests.rs`).
- **Sempre validar a UI por screenshot** e comparar com o protótipo (`superdesign/design_iterations/desktop_app_for_form_1_1.html`) — egui desalinha fácil.

## Mapa de documentos

- `docs/README.md` — índice geral + estado.
- `docs/decisoes/` — ADRs (0001–0009): stack, hexagonal, cross-platform, udisks/polkit, gravador inteligente, ordem, tema, qualidade/lints, nome.
- `docs/pesquisa/` — 01 (projeto referência `solana`), 02 (ferramentas Rust USB/ISO), 03 (gravação Linux vs Windows).
- `docs/superpowers/specs/` e `docs/superpowers/plans/` — specs e planos das fases.
- `docs/fase2-feedback.md` — 14 feedbacks da UI (todos ✅).
- `docs/screenshots/` — prints de validação.
