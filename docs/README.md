# Nur (نور) — Formatador de Pendrive & Criador de Boot

> **Nur** (árabe نور, "luz" — *An-Nûr*, um dos nomes de Deus). Dar boot é acender a máquina. Binário: `nur`. Ver [ADR 0009](decisoes/0009-nome-do-projeto.md).

App desktop em **Rust** para **formatar pendrives** e **criar pendrives bootáveis a partir de uma ISO**. UI em **egui/eframe**, arquitetura **hexagonal**, **cross-platform** (Linux primeiro). Idioma do produto: **pt-BR**.

> **Projeto de referência:** `/home/jonatas/projects/github/solana` — herdamos dele arquitetura, lints e abordagem de UI. Ver `pesquisa/01-projeto-referencia-solana.md`.

## Índice

### Especificação e planos
- [`superpowers/specs/2026-06-26-interface-formatador-pendrive-design.md`](superpowers/specs/2026-06-26-interface-formatador-pendrive-design.md) — **spec principal** (interface + arquitetura técnica + critérios de aceite).
- [`superpowers/plans/2026-06-26-nur-fase1-fundacao-linux.md`](superpowers/plans/2026-06-26-nur-fase1-fundacao-linux.md) — plano da **Fase 1** (fundação: workspace hexagonal + lints + UI rodando).
- [`superpowers/specs/2026-06-27-plano3-enumeracao-real-linux-design.md`](superpowers/specs/2026-06-27-plano3-enumeracao-real-linux-design.md) / [`superpowers/plans/2026-06-27-plano3-enumeracao-real-linux.md`](superpowers/plans/2026-06-27-plano3-enumeracao-real-linux.md) — **Fase 3**: enumeração real de pendrives no Linux via sysfs.
- [`superpowers/specs/2026-06-28-gravacao-raw-linux-design.md`](superpowers/specs/2026-06-28-gravacao-raw-linux-design.md) / [`superpowers/plans/2026-06-28-gravacao-raw-linux.md`](superpowers/plans/2026-06-28-gravacao-raw-linux.md) — **Fase 4**: gravação raw da ISO (Linux) via udisks2/polkit.
- [`superpowers/specs/2026-06-28-abrir-pendrive-gerenciador-design.md`](superpowers/specs/2026-06-28-abrir-pendrive-gerenciador-design.md) / [`superpowers/plans/2026-06-28-abrir-pendrive-gerenciador.md`](superpowers/plans/2026-06-28-abrir-pendrive-gerenciador.md) — **Fase 5**: abrir o pendrive no gerenciador de arquivos do SO.
- [`superpowers/plans/2026-06-27-english-identifiers.md`](superpowers/plans/2026-06-27-english-identifiers.md) — plano de identificadores em inglês + encapsulamento de campos.

### Pesquisa (relatórios completos — não perder)
- [`pesquisa/01-projeto-referencia-solana.md`](pesquisa/01-projeto-referencia-solana.md) — arquitetura/lints/UI do projeto de referência.
- [`pesquisa/02-ferramentas-rust-usb-iso.md`](pesquisa/02-ferramentas-rust-usb-iso.md) — crates Rust, egui 0.35, udisks2, técnica de gravação, privilégios, armadilhas.
- [`pesquisa/03-gravacao-iso-linux-vs-windows.md`](pesquisa/03-gravacao-iso-linux-vs-windows.md) — por que não existe método raw único; detecção de ISO; caminho Windows (WIM split/UEFI:NTFS); Ventoy; Secure Boot.

### Decisões (ADRs)
- [`decisoes/README.md`](decisoes/README.md) — índice das decisões com o porquê de cada uma.

### Protótipos de UI
- `../superdesign/design_iterations/desktop_app_for_form_1_1.html` — **versão final aprovada** (compacta, claro/escuro, com animações).
- `../superdesign/design_iterations/desktop_app_for_form_1.html` — base compacta.
- `../superdesign/design_iterations/desktop_app_for_form_2.html` — variação 2 colunas (descartada como base).
- `../superdesign/gallery.html` — galeria.

## Estado atual (2026-06-29)

**Fase:** as **Fases 1–5 estão implementadas e mergeadas** nesta base; a **Fase 6** (modo Formatar real) está em **PR aberta** (ainda não nesta base).

Entregue (mergeado):
- ✅ **Fase 1 — Fundação:** workspace hexagonal (`domain`→`application`→`infrastructure`→`app`→`ui`) com lints exigentes (sem-pânico, `unsafe_code=forbid`, limite de linhas/arquivo) e pipelines de CI/Release.
- ✅ **Fase 2 — UI completa:** egui/eframe fiel ao protótipo — tema claro/escuro, fonte Inter, janela arredondada, modal de confirmação "digite APAGAR" e componentes reutilizáveis.
- ✅ **Fase 3 — Enumeração real (Linux):** descoberta de pendrives via **sysfs** (`/sys/block`) com atualização ao vivo. Houve **pivot de udisks2 → sysfs** na enumeração por performance.
- ✅ **Fase 4 — Gravação raw da ISO:** gravação via udisks2/polkit (`Block.OpenDevice`, sem rodar como root), com detecção isohybrid, progresso real, cancelamento e verificação.
- ✅ **Fase 5 — Abrir no gerenciador:** abrir o pendrive no gerenciador de arquivos do SO.

Em PR aberta (ainda não nesta base):
- 🔜 **Fase 6 — Modo Formatar real:** formatação do pendrive (em revisão).

Decisões de base (ver [ADRs](decisoes/README.md)):
- Stack: Rust edição 2024, egui/eframe 0.35 (Wayland+X11, renderer `glow`, persistência+serde).
- Arquitetura: workspace hexagonal (`domain`→`application`→`infrastructure`→`app`→`ui`).
- Cross-platform faseado: **Linux primeiro**, depois Windows e macOS.
- Gravação: **gravador inteligente com auto-detecção** (raw write no Linux). Ventoy descartado.

## Próximos passos

1. Concluir a revisão e o merge da **Fase 6** (modo Formatar real).
2. Avançar o caminho **Windows** (spike de gravação/extração) e, depois, macOS.
