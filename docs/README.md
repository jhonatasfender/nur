# Nur (نور) — Formatador de Pendrive & Criador de Boot

> **Nur** (árabe نور, "luz" — *An-Nûr*, um dos nomes de Deus). Dar boot é acender a máquina. Binário: `nur`. Ver [ADR 0009](decisoes/0009-nome-do-projeto.md).

App desktop em **Rust** para **formatar pendrives** e **criar pendrives bootáveis a partir de uma ISO**. UI em **egui/eframe**, arquitetura **hexagonal**, **cross-platform** (Linux primeiro). Idioma do produto: **pt-BR**.

> **Projeto de referência:** `/home/jonatas/projects/github/solana` — herdamos dele arquitetura, lints e abordagem de UI. Ver `pesquisa/01-projeto-referencia-solana.md`.

## Índice

### Especificação
- [`superpowers/specs/2026-06-26-interface-formatador-pendrive-design.md`](superpowers/specs/2026-06-26-interface-formatador-pendrive-design.md) — **spec principal** (interface + arquitetura técnica + critérios de aceite).

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

## Estado atual (2026-06-26)

**Fase:** design/planejamento concluído; **ainda não há código**.

Decisões fechadas:
- ✅ UI: painel único compacto, claro/escuro, proteção "digite APAGAR", animações aprovadas (protótipos HTML são a fonte de verdade visual).
- ✅ Stack: Rust edição 2024, egui/eframe 0.35 (Wayland+X11, renderer `glow`, persistência+serde).
- ✅ Arquitetura: workspace hexagonal (`domain`→`application`→`infrastructure`→`app`→`ui`).
- ✅ Cross-platform faseado: **Linux (udisks2) primeiro**, depois Windows e macOS.
- ✅ Backend Linux: udisks2 via `zbus`, `Block.OpenDevice` com polkit (sem rodar como root).
- ✅ Gravação: **gravador inteligente com auto-detecção** (raw write p/ Linux, extração p/ Windows). Ventoy descartado.
- ✅ Ordem: **raw write primeiro + spike Windows cedo** para retirar risco (não inverter a ordem de construção).
- ✅ Qualidade: lints sem-pânico, `unsafe_code=forbid`, limite de ~199 linhas/arquivo, testes em arquivo irmão — herdados do `solana`.

## Próximos passos

1. (Opcional) `git init` + commit desta documentação.
2. Invocar a skill **writing-plans** para o plano de implementação da **Fase 1** (esqueleto do workspace + lints + UI rodando com adapters stub + raw write no Linux).
3. Rodar o **spike** do caminho Windows em paralelo (validação manual).
