# Decisões de Arquitetura (ADRs)

Registro das decisões tomadas no design do Formatador de Pendrive, com contexto e justificativa. Formato leve inspirado em ADR (Architecture Decision Record). Inspirado na convenção do projeto de referência `solana` (`docs/arquitetura/decisoes/`).

| # | Decisão | Status |
|---|---------|--------|
| [0001](0001-stack-rust-egui.md) | Linguagem Rust + UI egui/eframe 0.35 (modernizado) | Aceita |
| [0002](0002-arquitetura-hexagonal.md) | Arquitetura hexagonal em workspace de crates | Aceita |
| [0003](0003-cross-platform-faseado.md) | Cross-platform faseado (Linux primeiro) | Aceita |
| [0004](0004-backend-disco-udisks-polkit.md) | Backend de disco Linux via udisks2 + polkit | Aceita |
| [0005](0005-gravador-inteligente-vs-ventoy.md) | Gravador inteligente com auto-detecção (Ventoy descartado) | Aceita |
| [0006](0006-ordem-raw-write-primeiro-com-spike.md) | Raw write primeiro + spike Windows cedo | Aceita |
| [0007](0007-tema-claro-escuro.md) | Tema claro/escuro (desvio do solana dark-only) | Aceita |
| [0008](0008-padroes-qualidade-lints.md) | Padrões de qualidade e lints herdados | Aceita |
| [0009](0009-nome-do-projeto.md) | Nome do projeto: **Nur (نور)** — "luz" | Aceita |
| [0010](0010-nucleo-rust-nativo.md) | Núcleo Rust-nativo (sem ferramentas externas) + formatador FAT32 nativo | Aceita |

> Fontes que embasam estas decisões: `../pesquisa/01`, `../pesquisa/02`, `../pesquisa/03`.
