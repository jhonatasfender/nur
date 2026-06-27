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
- **xtask `check`**: roda `line-limit` (máx. ~199 linhas por `.rs`) **e `pub-fields`** (reprova qualquer campo `pub` — ver abaixo).
- **Estilo OOP estrito**: sem função livre exceto `main`; **zero campos `pub`** (campos privados + getters); Value Objects no lugar de primitivos.
- **Testes em arquivo irmão** (`foo.rs` → `foo/tests.rs`).

## Idioma (revisado em 2026-06-27)
**Código em inglês; português só em comentários, logs e textos de UI.**
- **Inglês**: nomes de tipos, traits, enums e variantes, structs, **campos**, métodos, funções, variáveis, constantes, **módulos/arquivos `.rs`** e **nomes de teste**.
- **Português**: comentários (`//`, `///`, `//!`), mensagens de log (`eprintln!`) e textos de UI mostrados ao usuário.
- Reverte a convenção pt-BR herdada do `solana`. (Os documentos em `docs/` seguem em pt-BR.)

## Enforcement de encapsulamento
O Rust **não tem `protected`** nem lint que proíba campos `pub`. O `unreachable_pub` (ligado) só força `pub`→`pub(crate)` onde aplicável. Para **exigir** "sem campos públicos", há uma regra própria **`PubFieldsRule`** no `tools/xtask` (`cargo xtask pub-fields`), rodada no CI via `cargo xtask check`, que falha se encontrar `pub <campo>:` em qualquer `.rs` de `crates/`.

## Desvio importante: `unsafe_code`
Manter **`forbid`** no workspace. Onde FFI for inevitável (adapters Windows/macOS — ver ADR 0003), **isolar** o `unsafe` no crate de infra daquele SO (relaxando o lint só ali) **ou** usar **shell-out seguro**. No Linux (zbus/udisks) é tudo Rust seguro.

## Justificativa
- Paridade de qualidade com o `solana`; código sem-pânico e sem `unsafe` por padrão é desejável num app que faz IO destrutivo.

## Consequências
- Disciplina alta (toda API pública documentada, arquivos pequenos, sem `unwrap`).
- O limite de 199 linhas força módulos pequenos e focados.
