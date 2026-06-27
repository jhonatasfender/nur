# ADR 0007 — Tema claro/escuro (desvio do solana dark-only)

**Status:** Aceita · **Data:** 2026-06-26

## Contexto
O usuário declarou claro/escuro como **imprescindível**. O projeto de referência `solana` é **dark-only** (`ThemeKit` parte de `Visuals::dark()`, sem toggle). Ver `../pesquisa/01` §1.

## Decisão
**Estender** o padrão de tema do `solana` para **duas paletas (claro + escuro) + toggle**:
- Toggle no header (ícone sol/lua).
- **Persistência** entre execuções (no app: storage do eframe + `serde`; no protótipo HTML: `localStorage`).
- Respeitar a **preferência do sistema** na primeira abertura.
- Em egui: `ctx.set_visuals(Visuals::light()/dark())` (ou `set_theme`/`ThemePreference`), com paletas próprias espelhando os protótipos.

## Justificativa
- Requisito explícito do usuário.
- O modelo do `solana` (cores semânticas centralizadas em um `ThemeKit`) suporta bem duas paletas — é uma extensão natural, não uma reescrita.

## Consequências
- Manter **duas** paletas consistentes (não só uma).
- As cores semânticas (vermelho destrutivo, verde sucesso) precisam funcionar nos dois temas.
