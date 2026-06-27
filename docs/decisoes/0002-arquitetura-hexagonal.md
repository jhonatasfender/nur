# ADR 0002 — Arquitetura hexagonal em workspace de crates

**Status:** Aceita · **Data:** 2026-06-26

## Contexto
Seguir a arquitetura do `solana`: workspace Cargo com camadas hexagonais (Ports & Adapters), com a regra de dependência **imposta pelo grafo de crates**. Ver `../pesquisa/01` §2.

## Decisão
Workspace com os crates:
- `domain` — núcleo puro: `Dispositivo`, `Imagem/ISO`, `OpçãoDeFormato`, value objects (tamanho, rótulo). Sem IO.
- `application` — casos de uso (`ListarDispositivos`, `Formatar`, `GravarImagem`, `VerificarIntegridade`) e **portas (traits)**: `DiskService`, `DeviceWatcher`, `ImageReader`, `PrivilegeElevator`, `BootableWriter` (com `IsoInspector` + estratégias). Depende só de `domain`.
- `infrastructure` — adapters concretos de IO por SO. Vê `domain`+`application`.
- `app` — composition root + binário(s); injeta adapters reais ou stubs.
- `ui` — presenter egui; fala só com `domain`+`application` via `Arc<dyn Trait>` + builder. **Nunca** depende de `infrastructure`.
- `tools/xtask` — fora do workspace de produção (limite de linhas etc.).

Injeção via `Arc<dyn Trait>` + **builder** (`with_*`) para trocar stub (preview/screenshots) por implementação real, como em `RoiApp` no `solana`.

## Justificativa
- A UI roda com dados falsos (preview) ou reais sem mudar a moldura.
- Fronteiras testáveis e isoladas; o IO de disco (arriscado) fica confinado na `infrastructure`.
- Encaixa perfeitamente o "gravador inteligente" (ADR 0005): `BootableWriter` é uma porta com estratégias.

## Consequências
- Mais cerimônia inicial (vários crates), compensada por fronteiras claras.
- Cada crate liga só as deps que tem permissão de usar (reforço de fronteira).
