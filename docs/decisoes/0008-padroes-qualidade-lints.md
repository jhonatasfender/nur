# ADR 0008 — Padrões de qualidade e lints herdados do solana

**Status:** Aceita · **Data:** 2026-06-26

## Contexto
O usuário quer seguir os **lints e padrões de qualidade** do `solana`. Detalhes em `../pesquisa/01` §3/§4.

## Decisão
Aplicar (quase verbatim) o pacote de qualidade do `solana`:

**`[workspace.lints.clippy]`:**
- `all = deny`, `pedantic = warn` (nursery OFF).
- Sem-pânico: `unwrap_used`/`expect_used`/`panic = deny`.
- `allow`s conscientes equivalentes (casts, `module_name_repetitions`, etc.), revisados para o nosso domínio.

**`[workspace.lints.rust]`:** `unreachable_pub`, `unnameable_types`, `missing_docs`, `unused_qualifications`, `redundant_imports`, etc. = `deny`; **`unsafe_code = forbid`**.

**`clippy.toml`:** libera `unwrap`/`expect`/`panic` **só em testes** (escopo `#[cfg(test)]`, não `#[allow]` no código).

**Outros:**
- `deny.toml` (advisories/licenças/sources só crates.io).
- CI com jobs paralelos: fmt, clippy (`-D warnings`), test, deny, doc, machete, spell, line-limit.
- **xtask line-limit**: máx. ~199 linhas por `.rs`.
- **Estilo OOP estrito**: sem função livre exceto `main`; campos privados + getters; Value Objects no lugar de primitivos.
- **Testes em arquivo irmão** (`foo.rs` → `foo/tests.rs`).
- Comentários em **pt-BR** explicando o "porquê".

## Desvio importante: `unsafe_code`
Manter **`forbid`** no workspace. Onde FFI for inevitável (adapters Windows/macOS — ver ADR 0003), **isolar** o `unsafe` no crate de infra daquele SO (relaxando o lint só ali) **ou** usar **shell-out seguro**. No Linux (zbus/udisks) é tudo Rust seguro.

## Justificativa
- Paridade de qualidade com o `solana`; código sem-pânico e sem `unsafe` por padrão é desejável num app que faz IO destrutivo.

## Consequências
- Disciplina alta (toda API pública documentada, arquivos pequenos, sem `unwrap`).
- O limite de 199 linhas força módulos pequenos e focados.
