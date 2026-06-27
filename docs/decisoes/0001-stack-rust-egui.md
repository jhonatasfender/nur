# ADR 0001 — Linguagem Rust + UI egui/eframe 0.35 (modernizado)

**Status:** Aceita · **Data:** 2026-06-26

## Contexto
Precisamos de um app desktop. O usuário definiu **Rust** e pediu para seguir a stack do projeto de referência `solana`, que usa **egui/eframe** nativo (não web). Ver `../pesquisa/01`.

## Decisão
- **Rust**, edição **2024**, stable recente.
- UI em **egui + eframe 0.35** (lançada 2026-06-25), nativo immediate-mode.
- Features `wayland` + `x11`; **renderer `glow`** (evita bug de input do wgpu no Wayland — issue emilk/egui#3924).
- Persistência via feature `persistence` + `serde` (salva tema/estado entre execuções).
- Async com **tokio** em background; ponte para a UI imediata via `request_repaint_after` + `runtime.handle()`.
- Erros: `thiserror` nas libs, `anyhow` no composition root.

## Justificativa
- Paridade com o `solana` (arquitetura/lints/UI que o usuário quer reusar).
- egui nativo é leve, sem runtime web; o mockup HTML serve de fonte de verdade visual (mesmo padrão do `solana`).
- **Modernizar** em vez de espelhar o `solana` (que está travado em egui 0.29/X11/rustc 1.84.1): projeto novo não deve herdar limitações — queremos Wayland e Rust atual. Ver ADR 0007 e `../pesquisa/02` §3.

## Consequências
- egui 0.35 exige edição 2024 e MSRV ~1.88+ — diverge da edição 2021 do `solana`.
- Cuidado com o renderer no Wayland (`glow`, não wgpu).
