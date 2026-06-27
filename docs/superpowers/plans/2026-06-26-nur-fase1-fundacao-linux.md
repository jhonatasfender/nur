# Nur — Fase 1: Fundação e Arquitetura (Linux) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Levantar o workspace hexagonal do Nur compilando e rodando: o binário `nur` abre uma janela egui com tema claro/escuro, listando dispositivos vindos de um adapter stub através das portas da arquitetura — com todo o ferramental de qualidade (lints, xtask line-limit, CI) ativo.

**Architecture:** Workspace Cargo com camadas hexagonais (`domain` → `application` (portas) → `infrastructure` (adapters) → `app` (composition root) → `ui` (presenter egui)). A regra de dependência é imposta pelo grafo de crates. A UI consome o domínio só via `Arc<dyn Trait>` injetado por um builder, permitindo trocar stub por implementação real sem mexer na moldura. Esta fase entrega a fatia vertical mínima (listar dispositivos stub na tela), estabelecendo todas as fronteiras.

**Tech Stack:** Rust edição 2024 · egui/eframe 0.35 (glow, wayland+x11) · tokio · thiserror/anyhow · serde · `cargo xtask` para lints customizados.

## Global Constraints

Copiado verbatim das decisões (ver `docs/decisoes/`). Vale para TODA task:

- **Edição Rust 2024**, MSRV 1.88+ (ADR 0001).
- **`unsafe_code = "forbid"`** no workspace (ADR 0008). Nenhum `unsafe` nesta fase (Linux/zbus é Rust seguro).
- **Sem-pânico em produção:** `clippy::unwrap_used`/`expect_used`/`panic = "deny"`. `unwrap`/`expect`/`panic!` permitidos **só em testes** (via `clippy.toml`).
- **`missing_docs = "deny"`**: todo item público precisa de `///`; todo crate precisa de `//!`.
- **`unreachable_pub = "deny"`**: usar a visibilidade mais restrita (`pub(crate)` quando não for API externa).
- **Limite de ~199 linhas por arquivo `.rs`** (xtask line-limit).
- **OOP estrito:** sem função livre exceto `fn main`. Helpers viram associated fn de struct/enum.
- **Testes em arquivo irmão:** `foo.rs` declara `#[cfg(test)] mod tests;`; os testes vão em `foo/tests.rs` com `use super::*;`.
- **Idioma (revisado 2026-06-27, ver ADR 0008):** **código em inglês** (identificadores, módulos/arquivos, nomes de teste); **pt-BR** só em comentários, logs e textos de UI.
- **Encapsulamento:** zero campos `pub` (campos privados + getters), exigido por `cargo xtask pub-fields`.
- **`ui` nunca depende de `infrastructure`.** Só `app` conhece todos os crates.

---

## File Structure

```
Cargo.toml                       # workspace: members, package, dependencies, lints
clippy.toml                      # libera unwrap/expect/panic em testes
deny.toml                        # advisories/licenças/sources
.cargo/config.toml               # alias `cargo xtask`
.github/workflows/ci.yml         # fmt, clippy, test, line-limit
tools/xtask/                     # workspace separado (não conta no line-limit de produção)
  Cargo.toml
  src/main.rs                    # entrypoint do xtask (despacha subcomandos)
  src/line_limit/rule.rs         # regra de limite de linhas (+ rule/tests.rs)
crates/
  domain/
    Cargo.toml
    src/lib.rs                   # reexports + //!
    src/byte_size.rs             # ByteSize (+ byte_size/tests.rs)
    src/caminho_dispositivo.rs   # CaminhoDispositivo (+ tests)
    src/rotulo_volume.rs         # RotuloVolume (+ tests)
    src/dispositivo.rs           # Dispositivo (+ tests)
  application/
    Cargo.toml
    src/lib.rs
    src/erros.rs                 # ErroDisco
    src/ports/mod.rs             # reexport das portas
    src/ports/disk_service.rs    # trait DiskService
    src/ports/ui_state.rs        # trait UiState + DispositivoView
    src/use_cases/mod.rs
    src/use_cases/listar_dispositivos.rs  # ListarDispositivos (+ tests)
  infrastructure/
    Cargo.toml
    src/lib.rs
    src/stub/mod.rs
    src/stub/disk_service_stub.rs # DiskServiceStub (+ tests)
  ui/
    Cargo.toml
    src/lib.rs
    src/theme/mod.rs
    src/theme/palette.rs         # Palette (claro/escuro) (+ tests)
    src/theme/theme_kit.rs       # ThemeKit::install / ThemePreference (+ tests)
    src/app.rs                   # NurApp (+ tests)
  app/
    Cargo.toml
    src/main.rs                  # binário `nur`
    src/window.rs                # cria runtime tokio + eframe
```

---

### Task 1: Workspace + xtask line-limit

**Files:**
- Create: `Cargo.toml`, `clippy.toml`, `deny.toml`, `.cargo/config.toml`
- Create: `tools/xtask/Cargo.toml`, `tools/xtask/src/main.rs`, `tools/xtask/src/line_limit/rule.rs`
- Test: `tools/xtask/src/line_limit/rule/tests.rs`

**Interfaces:**
- Produces: `LineLimitRule::check(raiz: &std::path::Path) -> Result<Vec<String>, std::io::Error>` — retorna a lista de arquivos `.rs` que excedem o limite (vazia = tudo OK). `LineLimitRule::LIMIT: usize = 200` (máx. 199 linhas).

- [ ] **Step 1: Criar o workspace root `Cargo.toml`**

```toml
[workspace]
resolver = "3"
members = [
    "crates/domain",
    "crates/application",
    "crates/infrastructure",
    "crates/ui",
    "crates/app",
]

[workspace.package]
edition = "2024"
rust-version = "1.88"
license = "MIT"
authors = ["Jonatas <jhonatas.fender@gmail.com>"]

[workspace.dependencies]
thiserror = "2"
anyhow = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
egui = "0.35"
eframe = { version = "0.35", default-features = false, features = ["default_fonts", "glow", "wayland", "x11"] }
serde = { version = "1", features = ["derive"] }

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
module_name_repetitions = "allow"
must_use_candidate = "allow"
cast_precision_loss = "allow"
cast_possible_truncation = "allow"
cast_sign_loss = "allow"

[workspace.lints.rust]
missing_docs = "deny"
unreachable_pub = "deny"
unsafe_code = "forbid"
```

- [ ] **Step 2: Criar `clippy.toml`, `deny.toml`, `.cargo/config.toml`**

`clippy.toml`:
```toml
# Pânico é proibido em produção, mas liberado em testes (escopo #[cfg(test)]).
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
```

`deny.toml`:
```toml
[advisories]
version = 2

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib", "Unicode-3.0"]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

`.cargo/config.toml`:
```toml
[alias]
xtask = "run --manifest-path tools/xtask/Cargo.toml --release --"
```

- [ ] **Step 3: Criar o crate `tools/xtask` (workspace separado)**

`tools/xtask/Cargo.toml`:
```toml
[package]
name = "xtask"
version = "0.0.0"
edition = "2024"
publish = false

# O '[workspace]' vazio desacopla o xtask do workspace de produção,
# para que os 'members' sejam exatamente as camadas da arquitetura.
[workspace]

[lints.clippy]
all = { level = "deny", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
```

- [ ] **Step 4: Escrever o teste que falha (`tools/xtask/src/line_limit/rule/tests.rs`)**

```rust
use super::*;
use std::io::Write;

#[test]
fn aponta_arquivo_acima_do_limite() {
    let dir = std::env::temp_dir().join("nur_xtask_test_grande");
    std::fs::create_dir_all(&dir).unwrap();
    let arquivo = dir.join("grande.rs");
    let mut f = std::fs::File::create(&arquivo).unwrap();
    for _ in 0..LineLimitRule::LIMIT {
        writeln!(f, "// linha").unwrap();
    }
    let violacoes = LineLimitRule::check(&dir).unwrap();
    assert!(violacoes.iter().any(|v| v.contains("grande.rs")));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn aceita_arquivo_dentro_do_limite() {
    let dir = std::env::temp_dir().join("nur_xtask_test_pequeno");
    std::fs::create_dir_all(&dir).unwrap();
    let mut f = std::fs::File::create(dir.join("pequeno.rs")).unwrap();
    writeln!(f, "// só uma linha").unwrap();
    let violacoes = LineLimitRule::check(&dir).unwrap();
    assert!(violacoes.is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}
```

- [ ] **Step 5: Rodar o teste e ver falhar**

Run: `cargo test --manifest-path tools/xtask/Cargo.toml`
Expected: FAIL com erro de compilação ("cannot find `LineLimitRule`").

- [ ] **Step 6: Implementar `tools/xtask/src/line_limit/rule.rs`**

```rust
//! Regra de limite de linhas por arquivo `.rs`.

use std::path::Path;

/// Verifica que nenhum arquivo `.rs` ultrapassa o limite de linhas.
pub struct LineLimitRule;

impl LineLimitRule {
    /// Limite (exclusivo): arquivos com `LIMIT` ou mais linhas violam a regra.
    pub const LIMIT: usize = 200;

    /// Varre `raiz` recursivamente e retorna os caminhos que excedem o limite.
    pub fn check(raiz: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut violacoes = Vec::new();
        Self::varrer(raiz, &mut violacoes)?;
        Ok(violacoes)
    }

    fn varrer(dir: &Path, acc: &mut Vec<String>) -> Result<(), std::io::Error> {
        for entrada in std::fs::read_dir(dir)? {
            let caminho = entrada?.path();
            if caminho.is_dir() {
                if caminho.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                Self::varrer(&caminho, acc)?;
            } else if caminho.extension().is_some_and(|e| e == "rs") {
                let linhas = std::fs::read_to_string(&caminho)?.lines().count();
                if linhas >= Self::LIMIT {
                    acc.push(format!("{} ({linhas} linhas)", caminho.display()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 7: Criar o entrypoint `tools/xtask/src/main.rs`**

```rust
//! Ferramenta de build do Nur (lints customizados).

mod line_limit {
    pub mod rule;
}

use line_limit::rule::LineLimitRule;

fn main() -> std::process::ExitCode {
    let comando = std::env::args().nth(1).unwrap_or_default();
    match comando.as_str() {
        "line-limit" => executar_line_limit(),
        _ => {
            eprintln!("uso: cargo xtask line-limit");
            std::process::ExitCode::FAILURE
        }
    }
}

fn executar_line_limit() -> std::process::ExitCode {
    let raiz = Path::new("crates");
    match LineLimitRule::check(raiz) {
        Ok(v) if v.is_empty() => {
            println!("line-limit: OK");
            std::process::ExitCode::SUCCESS
        }
        Ok(v) => {
            for item in v {
                eprintln!("EXCEDE 199 linhas: {item}");
            }
            std::process::ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

use std::path::Path;
```

- [ ] **Step 8: Rodar o teste e ver passar**

Run: `cargo test --manifest-path tools/xtask/Cargo.toml`
Expected: PASS (2 testes).

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml clippy.toml deny.toml .cargo tools
git commit -m "feat(xtask): workspace + regra de limite de linhas"
```

---

### Task 2: domain — ByteSize

**Files:**
- Create: `crates/domain/Cargo.toml`, `crates/domain/src/lib.rs`, `crates/domain/src/byte_size.rs`
- Test: `crates/domain/src/byte_size/tests.rs`

**Interfaces:**
- Produces: `ByteSize::from_bytes(u64) -> ByteSize`, `ByteSize::as_bytes(self) -> u64`, `ByteSize::humanize(self) -> String`. Deriva `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord`.

- [ ] **Step 1: Criar `crates/domain/Cargo.toml` e `lib.rs`**

`crates/domain/Cargo.toml`:
```toml
[package]
name = "domain"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
thiserror.workspace = true

[lints]
workspace = true
```

`crates/domain/src/lib.rs`:
```rust
//! Núcleo de domínio do Nur: modelos e value objects puros, sem IO.

mod byte_size;

pub use byte_size::ByteSize;
```

- [ ] **Step 2: Escrever o teste que falha (`crates/domain/src/byte_size/tests.rs`)**

```rust
use super::*;

#[test]
fn humaniza_bytes_pequenos() {
    assert_eq!(ByteSize::from_bytes(512).humanize(), "512 B");
}

#[test]
fn humaniza_gigabytes() {
    assert_eq!(ByteSize::from_bytes(32_000_000_000).humanize(), "32.0 GB");
}

#[test]
fn preserva_contagem_de_bytes() {
    assert_eq!(ByteSize::from_bytes(1024).as_bytes(), 1024);
}
```

- [ ] **Step 3: Rodar o teste e ver falhar**

Run: `cargo test -p domain`
Expected: FAIL ("cannot find `ByteSize`" / módulo de testes sem alvo).

- [ ] **Step 4: Implementar `crates/domain/src/byte_size.rs`**

```rust
//! Tamanho em bytes como value object, com formatação humana.

/// Tamanho de armazenamento em bytes (base decimal para exibição).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteSize(u64);

impl ByteSize {
    /// Cria a partir de uma contagem de bytes.
    #[must_use]
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Retorna a contagem de bytes.
    #[must_use]
    pub const fn as_bytes(self) -> u64 {
        self.0
    }

    /// Formata em unidade humana (ex.: "32.0 GB"). Base decimal (1000).
    #[must_use]
    pub fn humanize(self) -> String {
        const UNIDADES: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut valor = self.0 as f64;
        let mut indice = 0;
        while valor >= 1000.0 && indice < UNIDADES.len() - 1 {
            valor /= 1000.0;
            indice += 1;
        }
        if indice == 0 {
            format!("{} B", self.0)
        } else {
            format!("{valor:.1} {}", UNIDADES[indice])
        }
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Rodar o teste e ver passar**

Run: `cargo test -p domain`
Expected: PASS (3 testes).

- [ ] **Step 6: Commit**

```bash
git add crates/domain
git commit -m "feat(domain): value object ByteSize"
```

---

### Task 3: domain — CaminhoDispositivo e RotuloVolume

**Files:**
- Create: `crates/domain/src/caminho_dispositivo.rs`, `crates/domain/src/rotulo_volume.rs`
- Modify: `crates/domain/src/lib.rs`
- Test: `crates/domain/src/caminho_dispositivo/tests.rs`, `crates/domain/src/rotulo_volume/tests.rs`

**Interfaces:**
- Produces: `CaminhoDispositivo::new(String) -> CaminhoDispositivo`, `.as_str(&self) -> &str`. `RotuloVolume::parse(&str) -> Result<RotuloVolume, RotuloInvalido>`, `.as_str(&self) -> &str`; `RotuloInvalido` é erro `thiserror`. Rótulo válido: 1–11 chars ASCII (limite FAT). Ambos derivam `Debug, Clone, PartialEq, Eq`.

- [ ] **Step 1: Escrever testes que falham**

`crates/domain/src/caminho_dispositivo/tests.rs`:
```rust
use super::*;

#[test]
fn expoe_o_caminho() {
    let c = CaminhoDispositivo::new("/dev/sdb".to_owned());
    assert_eq!(c.as_str(), "/dev/sdb");
}
```

`crates/domain/src/rotulo_volume/tests.rs`:
```rust
use super::*;

#[test]
fn aceita_rotulo_valido() {
    let r = RotuloVolume::parse("BOOTUSB").unwrap();
    assert_eq!(r.as_str(), "BOOTUSB");
}

#[test]
fn rejeita_vazio() {
    assert!(RotuloVolume::parse("").is_err());
}

#[test]
fn rejeita_acima_de_11_chars() {
    assert!(RotuloVolume::parse("ABCDEFGHIJKL").is_err());
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p domain`
Expected: FAIL (tipos inexistentes).

- [ ] **Step 3: Implementar `crates/domain/src/caminho_dispositivo.rs`**

```rust
//! Caminho de um dispositivo de bloco (ex.: `/dev/sdb`).

/// Caminho de dispositivo de bloco no sistema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaminhoDispositivo(String);

impl CaminhoDispositivo {
    /// Cria a partir de um caminho já validado pelo adapter de SO.
    #[must_use]
    pub fn new(caminho: String) -> Self {
        Self(caminho)
    }

    /// Retorna o caminho como string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Implementar `crates/domain/src/rotulo_volume.rs`**

```rust
//! Rótulo de volume com validação (limite FAT de 11 caracteres).

/// Erro de rótulo de volume inválido.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RotuloInvalido {
    /// O rótulo estava vazio.
    #[error("rótulo vazio")]
    Vazio,
    /// O rótulo excede 11 caracteres.
    #[error("rótulo excede 11 caracteres")]
    MuitoLongo,
}

/// Rótulo de volume válido (1–11 caracteres).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotuloVolume(String);

impl RotuloVolume {
    /// Valida e cria um rótulo. Erros: vazio ou acima de 11 caracteres.
    ///
    /// # Errors
    /// Retorna [`RotuloInvalido`] quando a string não respeita o limite.
    pub fn parse(texto: &str) -> Result<Self, RotuloInvalido> {
        if texto.is_empty() {
            return Err(RotuloInvalido::Vazio);
        }
        if texto.chars().count() > 11 {
            return Err(RotuloInvalido::MuitoLongo);
        }
        Ok(Self(texto.to_owned()))
    }

    /// Retorna o rótulo como string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Atualizar `crates/domain/src/lib.rs`**

```rust
//! Núcleo de domínio do Nur: modelos e value objects puros, sem IO.

mod byte_size;
mod caminho_dispositivo;
mod rotulo_volume;

pub use byte_size::ByteSize;
pub use caminho_dispositivo::CaminhoDispositivo;
pub use rotulo_volume::{RotuloInvalido, RotuloVolume};
```

- [ ] **Step 6: Rodar e ver passar**

Run: `cargo test -p domain`
Expected: PASS (4 novos testes).

- [ ] **Step 7: Commit**

```bash
git add crates/domain
git commit -m "feat(domain): CaminhoDispositivo e RotuloVolume"
```

---

### Task 4: domain — Dispositivo

**Files:**
- Create: `crates/domain/src/dispositivo.rs`
- Modify: `crates/domain/src/lib.rs`
- Test: `crates/domain/src/dispositivo/tests.rs`

**Interfaces:**
- Produces: `Dispositivo::new(caminho: CaminhoDispositivo, modelo: String, tamanho: ByteSize, removivel: bool) -> Dispositivo` com getters `caminho()`, `modelo()`, `tamanho()`, `removivel()` e `descricao() -> String` (ex.: "SanDisk Ultra — 32.0 GB (/dev/sdb)"). Deriva `Debug, Clone, PartialEq, Eq`.

- [ ] **Step 1: Escrever o teste que falha**

`crates/domain/src/dispositivo/tests.rs`:
```rust
use super::*;

#[test]
fn monta_descricao_legivel() {
    let d = Dispositivo::new(
        CaminhoDispositivo::new("/dev/sdb".to_owned()),
        "SanDisk Ultra".to_owned(),
        ByteSize::from_bytes(32_000_000_000),
        true,
    );
    assert_eq!(d.descricao(), "SanDisk Ultra — 32.0 GB (/dev/sdb)");
    assert!(d.removivel());
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p domain`
Expected: FAIL ("cannot find `Dispositivo`").

- [ ] **Step 3: Implementar `crates/domain/src/dispositivo.rs`**

```rust
//! Dispositivo de bloco detectado (pendrive) como agregado de domínio.

use crate::{ByteSize, CaminhoDispositivo};

/// Um dispositivo de armazenamento detectado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispositivo {
    caminho: CaminhoDispositivo,
    modelo: String,
    tamanho: ByteSize,
    removivel: bool,
}

impl Dispositivo {
    /// Cria um dispositivo a partir dos dados do adapter de SO.
    #[must_use]
    pub fn new(
        caminho: CaminhoDispositivo,
        modelo: String,
        tamanho: ByteSize,
        removivel: bool,
    ) -> Self {
        Self { caminho, modelo, tamanho, removivel }
    }

    /// Caminho do dispositivo.
    #[must_use]
    pub fn caminho(&self) -> &CaminhoDispositivo {
        &self.caminho
    }

    /// Modelo do dispositivo.
    #[must_use]
    pub fn modelo(&self) -> &str {
        &self.modelo
    }

    /// Tamanho do dispositivo.
    #[must_use]
    pub fn tamanho(&self) -> ByteSize {
        self.tamanho
    }

    /// Indica se o dispositivo é removível.
    #[must_use]
    pub fn removivel(&self) -> bool {
        self.removivel
    }

    /// Descrição legível para a UI (modelo — tamanho (caminho)).
    #[must_use]
    pub fn descricao(&self) -> String {
        format!(
            "{} — {} ({})",
            self.modelo,
            self.tamanho.humanize(),
            self.caminho.as_str()
        )
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Atualizar `crates/domain/src/lib.rs`** (adicionar `mod dispositivo;` e `pub use dispositivo::Dispositivo;`)

```rust
//! Núcleo de domínio do Nur: modelos e value objects puros, sem IO.

mod byte_size;
mod caminho_dispositivo;
mod dispositivo;
mod rotulo_volume;

pub use byte_size::ByteSize;
pub use caminho_dispositivo::CaminhoDispositivo;
pub use dispositivo::Dispositivo;
pub use rotulo_volume::{RotuloInvalido, RotuloVolume};
```

- [ ] **Step 5: Rodar e ver passar**

Run: `cargo test -p domain`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/domain
git commit -m "feat(domain): agregado Dispositivo"
```

---

### Task 5: application — erros e porta DiskService

**Files:**
- Create: `crates/application/Cargo.toml`, `crates/application/src/lib.rs`, `crates/application/src/erros.rs`, `crates/application/src/ports/mod.rs`, `crates/application/src/ports/disk_service.rs`

**Interfaces:**
- Produces: `ErroDisco` (enum `thiserror`, variantes `Indisponivel(String)`). Trait `DiskService { fn listar_dispositivos(&self) -> Result<Vec<Dispositivo>, ErroDisco>; }` (síncrono nesta fase; o stub é instantâneo).

- [ ] **Step 1: Criar `crates/application/Cargo.toml`**

```toml
[package]
name = "application"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
domain = { path = "../domain" }
thiserror.workspace = true

[lints]
workspace = true
```

- [ ] **Step 2: Criar `crates/application/src/erros.rs`**

```rust
//! Erros da camada de aplicação.

/// Falhas ao interagir com o serviço de disco.
#[derive(Debug, thiserror::Error)]
pub enum ErroDisco {
    /// O backend de disco está indisponível ou falhou.
    #[error("serviço de disco indisponível: {0}")]
    Indisponivel(String),
}
```

- [ ] **Step 3: Criar `crates/application/src/ports/disk_service.rs`**

```rust
//! Porta de acesso ao disco (implementada na infraestrutura).

use crate::erros::ErroDisco;
use domain::Dispositivo;

/// Serviço de disco: enumera dispositivos e (futuramente) grava/formata.
pub trait DiskService: Send + Sync {
    /// Lista os dispositivos removíveis disponíveis.
    ///
    /// # Errors
    /// Retorna [`ErroDisco`] se o backend falhar.
    fn listar_dispositivos(&self) -> Result<Vec<Dispositivo>, ErroDisco>;
}
```

- [ ] **Step 4: Criar `crates/application/src/ports/mod.rs`**

```rust
//! Portas (traits) que a aplicação define e a infraestrutura implementa.

mod disk_service;

pub use disk_service::DiskService;
```

- [ ] **Step 5: Criar `crates/application/src/lib.rs`**

```rust
//! Casos de uso e portas do Nur (regra de negócio orquestrada).

pub mod erros;
pub mod ports;
```

- [ ] **Step 6: Compilar**

Run: `cargo build -p application`
Expected: compila sem erros.

- [ ] **Step 7: Commit**

```bash
git add crates/application
git commit -m "feat(application): erros e porta DiskService"
```

---

### Task 6: application — porta UiState e DispositivoView

**Files:**
- Create: `crates/application/src/ports/ui_state.rs`
- Modify: `crates/application/src/ports/mod.rs`

**Interfaces:**
- Produces: `DispositivoView { pub caminho: String, pub descricao: String }` (struct de apresentação, `Clone`). Trait `UiState: Send + Sync { fn dispositivos(&self) -> Vec<DispositivoView>; }`.

- [ ] **Step 1: Criar `crates/application/src/ports/ui_state.rs`**

```rust
//! Porta de estado da UI: o que a tela lê para se desenhar.

/// Projeção de um dispositivo para exibição na UI (sem tipos de domínio).
#[derive(Debug, Clone)]
pub struct DispositivoView {
    /// Caminho do dispositivo (ex.: `/dev/sdb`).
    pub caminho: String,
    /// Descrição legível (modelo — tamanho (caminho)).
    pub descricao: String,
}

/// Estado lido pela UI a cada frame.
pub trait UiState: Send + Sync {
    /// Lista de dispositivos para popular o seletor.
    fn dispositivos(&self) -> Vec<DispositivoView>;
}
```

- [ ] **Step 2: Atualizar `crates/application/src/ports/mod.rs`**

```rust
//! Portas (traits) que a aplicação define e a infraestrutura implementa.

mod disk_service;
mod ui_state;

pub use disk_service::DiskService;
pub use ui_state::{DispositivoView, UiState};
```

- [ ] **Step 3: Compilar**

Run: `cargo build -p application`
Expected: compila.

- [ ] **Step 4: Commit**

```bash
git add crates/application
git commit -m "feat(application): porta UiState e DispositivoView"
```

---

### Task 7: application — caso de uso ListarDispositivos

**Files:**
- Create: `crates/application/src/use_cases/mod.rs`, `crates/application/src/use_cases/listar_dispositivos.rs`
- Modify: `crates/application/src/lib.rs`
- Test: `crates/application/src/use_cases/listar_dispositivos/tests.rs`

**Interfaces:**
- Consumes: `DiskService`, `Dispositivo`.
- Produces: `ListarDispositivos::new(servico: Arc<dyn DiskService>) -> Self`; `.executar(&self) -> Result<Vec<DispositivoView>, ErroDisco>` (mapeia `Dispositivo` → `DispositivoView`).

- [ ] **Step 1: Escrever o teste que falha (com fake de DiskService)**

`crates/application/src/use_cases/listar_dispositivos/tests.rs`:
```rust
use super::*;
use domain::{ByteSize, CaminhoDispositivo, Dispositivo};
use std::sync::Arc;

struct DiskServiceFake;
impl DiskService for DiskServiceFake {
    fn listar_dispositivos(&self) -> Result<Vec<Dispositivo>, ErroDisco> {
        Ok(vec![Dispositivo::new(
            CaminhoDispositivo::new("/dev/sdb".to_owned()),
            "SanDisk Ultra".to_owned(),
            ByteSize::from_bytes(32_000_000_000),
            true,
        )])
    }
}

#[test]
fn mapeia_dispositivos_para_views() {
    let uc = ListarDispositivos::new(Arc::new(DiskServiceFake));
    let views = uc.executar().unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].caminho, "/dev/sdb");
    assert!(views[0].descricao.contains("SanDisk Ultra"));
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p application`
Expected: FAIL ("cannot find `ListarDispositivos`").

- [ ] **Step 3: Implementar `crates/application/src/use_cases/listar_dispositivos.rs`**

```rust
//! Caso de uso: listar dispositivos disponíveis para exibição.

use crate::erros::ErroDisco;
use crate::ports::{DiskService, DispositivoView};
use std::sync::Arc;

/// Lista dispositivos e os projeta para a UI.
pub struct ListarDispositivos {
    servico: Arc<dyn DiskService>,
}

impl ListarDispositivos {
    /// Cria o caso de uso com a porta de disco injetada.
    #[must_use]
    pub fn new(servico: Arc<dyn DiskService>) -> Self {
        Self { servico }
    }

    /// Executa a listagem e mapeia para [`DispositivoView`].
    ///
    /// # Errors
    /// Propaga [`ErroDisco`] do backend.
    pub fn executar(&self) -> Result<Vec<DispositivoView>, ErroDisco> {
        let dispositivos = self.servico.listar_dispositivos()?;
        Ok(dispositivos
            .into_iter()
            .map(|d| DispositivoView {
                caminho: d.caminho().as_str().to_owned(),
                descricao: d.descricao(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Criar `crates/application/src/use_cases/mod.rs`**

```rust
//! Casos de uso da aplicação.

mod listar_dispositivos;

pub use listar_dispositivos::ListarDispositivos;
```

- [ ] **Step 5: Atualizar `crates/application/src/lib.rs`**

```rust
//! Casos de uso e portas do Nur (regra de negócio orquestrada).

pub mod erros;
pub mod ports;
pub mod use_cases;
```

- [ ] **Step 6: Rodar e ver passar**

Run: `cargo test -p application`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/application
git commit -m "feat(application): caso de uso ListarDispositivos"
```

---

### Task 8: infrastructure — DiskServiceStub

**Files:**
- Create: `crates/infrastructure/Cargo.toml`, `crates/infrastructure/src/lib.rs`, `crates/infrastructure/src/stub/mod.rs`, `crates/infrastructure/src/stub/disk_service_stub.rs`
- Test: `crates/infrastructure/src/stub/disk_service_stub/tests.rs`

**Interfaces:**
- Consumes: `DiskService`, `ErroDisco`, `Dispositivo`.
- Produces: `DiskServiceStub::new() -> Self` implementando `DiskService` com 2 dispositivos canônicos.

- [ ] **Step 1: Criar `crates/infrastructure/Cargo.toml`**

```toml
[package]
name = "infrastructure"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
domain = { path = "../domain" }
application = { path = "../application" }

[lints]
workspace = true
```

- [ ] **Step 2: Escrever o teste que falha**

`crates/infrastructure/src/stub/disk_service_stub/tests.rs`:
```rust
use super::*;
use application::ports::DiskService;

#[test]
fn devolve_dois_dispositivos_canonicos() {
    let stub = DiskServiceStub::new();
    let dispositivos = stub.listar_dispositivos().unwrap();
    assert_eq!(dispositivos.len(), 2);
    assert_eq!(dispositivos[0].caminho().as_str(), "/dev/sdb");
}
```

- [ ] **Step 3: Rodar e ver falhar**

Run: `cargo test -p infrastructure`
Expected: FAIL ("cannot find `DiskServiceStub`").

- [ ] **Step 4: Implementar `crates/infrastructure/src/stub/disk_service_stub.rs`**

```rust
//! Adapter stub do DiskService (dados canônicos para preview/desenvolvimento).

use application::erros::ErroDisco;
use application::ports::DiskService;
use domain::{ByteSize, CaminhoDispositivo, Dispositivo};

/// Implementação stub que devolve dispositivos fixos (sem tocar o SO).
pub struct DiskServiceStub;

impl DiskServiceStub {
    /// Cria o stub.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiskServiceStub {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskService for DiskServiceStub {
    fn listar_dispositivos(&self) -> Result<Vec<Dispositivo>, ErroDisco> {
        Ok(vec![
            Dispositivo::new(
                CaminhoDispositivo::new("/dev/sdb".to_owned()),
                "SanDisk Ultra".to_owned(),
                ByteSize::from_bytes(32_000_000_000),
                true,
            ),
            Dispositivo::new(
                CaminhoDispositivo::new("/dev/sdc".to_owned()),
                "Kingston DataTraveler".to_owned(),
                ByteSize::from_bytes(16_000_000_000),
                true,
            ),
        ])
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Criar `crates/infrastructure/src/stub/mod.rs` e `lib.rs`**

`crates/infrastructure/src/stub/mod.rs`:
```rust
//! Adapters stub para desenvolvimento e preview.

mod disk_service_stub;

pub use disk_service_stub::DiskServiceStub;
```

`crates/infrastructure/src/lib.rs`:
```rust
//! Adapters concretos de IO do Nur (por SO e stubs).

pub mod stub;
```

- [ ] **Step 6: Rodar e ver passar**

Run: `cargo test -p infrastructure`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/infrastructure
git commit -m "feat(infrastructure): DiskServiceStub"
```

---

### Task 9: ui — Palette (claro/escuro)

**Files:**
- Create: `crates/ui/Cargo.toml`, `crates/ui/src/lib.rs`, `crates/ui/src/theme/mod.rs`, `crates/ui/src/theme/palette.rs`
- Test: `crates/ui/src/theme/palette/tests.rs`

**Interfaces:**
- Produces: `Palette { pub fundo: egui::Color32, pub superficie: egui::Color32, pub texto: egui::Color32, pub destrutivo: egui::Color32, pub sucesso: egui::Color32 }`; `Palette::clara() -> Palette`, `Palette::escura() -> Palette`. Espelha os tokens dos protótipos HTML.

- [ ] **Step 1: Criar `crates/ui/Cargo.toml`**

```toml
[package]
name = "ui"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
domain = { path = "../domain" }
application = { path = "../application" }
egui.workspace = true
eframe.workspace = true
serde = { workspace = true }

[lints]
workspace = true
```

- [ ] **Step 2: Escrever o teste que falha**

`crates/ui/src/theme/palette/tests.rs`:
```rust
use super::*;

#[test]
fn temas_tem_fundos_diferentes() {
    assert_ne!(Palette::clara().fundo, Palette::escura().fundo);
}

#[test]
fn sucesso_e_verde_nos_dois_temas() {
    // Verde de sucesso é o mesmo token (#16A34A) nos dois temas.
    assert_eq!(Palette::clara().sucesso, Palette::escura().sucesso);
}
```

- [ ] **Step 3: Rodar e ver falhar**

Run: `cargo test -p ui`
Expected: FAIL ("cannot find `Palette`").

- [ ] **Step 4: Implementar `crates/ui/src/theme/palette.rs`**

```rust
//! Paletas de cores claro/escuro, espelhando os protótipos HTML.

use egui::Color32;

/// Conjunto de cores de um tema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Cor de fundo da janela.
    pub fundo: Color32,
    /// Cor das superfícies/cards.
    pub superficie: Color32,
    /// Cor de texto primário.
    pub texto: Color32,
    /// Acento destrutivo (vermelho).
    pub destrutivo: Color32,
    /// Acento de sucesso (verde).
    pub sucesso: Color32,
}

impl Palette {
    /// Tema claro (fundo cinza-claro, superfície branca).
    #[must_use]
    pub const fn clara() -> Self {
        Self {
            fundo: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            superficie: Color32::WHITE,
            texto: Color32::from_rgb(0x11, 0x18, 0x27),
            destrutivo: Color32::from_rgb(0xDC, 0x26, 0x26),
            sucesso: Color32::from_rgb(0x16, 0xA3, 0x4A),
        }
    }

    /// Tema escuro (fundo quase preto, superfície cinza-escuro).
    #[must_use]
    pub const fn escura() -> Self {
        Self {
            fundo: Color32::from_rgb(0x0A, 0x0A, 0x0A),
            superficie: Color32::from_rgb(0x11, 0x18, 0x27),
            texto: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            destrutivo: Color32::from_rgb(0xDC, 0x26, 0x26),
            sucesso: Color32::from_rgb(0x16, 0xA3, 0x4A),
        }
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Criar `crates/ui/src/theme/mod.rs` e `lib.rs`**

`crates/ui/src/theme/mod.rs`:
```rust
//! Tema do Nur: paletas e instalação de estilo.

mod palette;

pub use palette::Palette;
```

`crates/ui/src/lib.rs`:
```rust
//! Camada de apresentação (egui) do Nur.

pub mod theme;
```

- [ ] **Step 6: Rodar e ver passar**

Run: `cargo test -p ui`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/ui
git commit -m "feat(ui): paletas claro/escuro"
```

---

### Task 10: ui — ThemeKit e ThemePreference

**Files:**
- Create: `crates/ui/src/theme/theme_kit.rs`
- Modify: `crates/ui/src/theme/mod.rs`
- Test: `crates/ui/src/theme/theme_kit/tests.rs`

**Interfaces:**
- Consumes: `Palette`.
- Produces: `enum ThemePreference { Claro, Escuro }` (deriva `serde::Serialize/Deserialize`, `Clone, Copy, PartialEq, Eq`); `ThemePreference::alternar(self) -> Self`; `ThemePreference::palette(self) -> Palette`; `ThemeKit::install(ctx: &egui::Context, pref: ThemePreference)`.

- [ ] **Step 1: Escrever o teste que falha**

`crates/ui/src/theme/theme_kit/tests.rs`:
```rust
use super::*;

#[test]
fn alterna_entre_claro_e_escuro() {
    assert_eq!(ThemePreference::Claro.alternar(), ThemePreference::Escuro);
    assert_eq!(ThemePreference::Escuro.alternar(), ThemePreference::Claro);
}

#[test]
fn preferencia_resolve_palette() {
    assert_eq!(ThemePreference::Escuro.palette(), Palette::escura());
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p ui`
Expected: FAIL.

- [ ] **Step 3: Implementar `crates/ui/src/theme/theme_kit.rs`**

```rust
//! Preferência de tema e instalação dos `Visuals` no contexto egui.

use crate::theme::Palette;

/// Preferência de tema escolhida pelo usuário (persistível).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemePreference {
    /// Tema claro.
    Claro,
    /// Tema escuro.
    Escuro,
}

impl ThemePreference {
    /// Alterna entre claro e escuro.
    #[must_use]
    pub const fn alternar(self) -> Self {
        match self {
            Self::Claro => Self::Escuro,
            Self::Escuro => Self::Claro,
        }
    }

    /// Paleta correspondente a esta preferência.
    #[must_use]
    pub const fn palette(self) -> Palette {
        match self {
            Self::Claro => Palette::clara(),
            Self::Escuro => Palette::escura(),
        }
    }
}

/// Instala o estilo do Nur (cores) num contexto egui.
pub struct ThemeKit;

impl ThemeKit {
    /// Aplica os `Visuals` derivados da preferência ao contexto.
    pub fn install(ctx: &egui::Context, pref: ThemePreference) {
        let palette = pref.palette();
        let mut visuals = match pref {
            ThemePreference::Claro => egui::Visuals::light(),
            ThemePreference::Escuro => egui::Visuals::dark(),
        };
        visuals.panel_fill = palette.fundo;
        visuals.window_fill = palette.superficie;
        visuals.override_text_color = Some(palette.texto);
        ctx.set_visuals(visuals);
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Atualizar `crates/ui/src/theme/mod.rs`**

```rust
//! Tema do Nur: paletas e instalação de estilo.

mod palette;
mod theme_kit;

pub use palette::Palette;
pub use theme_kit::{ThemeKit, ThemePreference};
```

- [ ] **Step 5: Rodar e ver passar**

Run: `cargo test -p ui`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/ui
git commit -m "feat(ui): ThemeKit e ThemePreference"
```

---

### Task 11: ui — NurApp (struct, builder e painel central)

**Files:**
- Create: `crates/ui/src/app.rs`
- Modify: `crates/ui/src/lib.rs`
- Test: `crates/ui/src/app/tests.rs`

**Interfaces:**
- Consumes: `UiState`, `DispositivoView`, `ThemePreference`, `ThemeKit`.
- Produces: `NurApp::new(estado: Arc<dyn UiState>) -> Self` (tema padrão Escuro); `NurApp::com_tema(self, pref: ThemePreference) -> Self` (builder); implementa `eframe::App::update`; getter de teste `pub(crate) fn tema(&self) -> ThemePreference`.

- [ ] **Step 1: Escrever o teste que falha**

`crates/ui/src/app/tests.rs`:
```rust
use super::*;
use application::ports::{DispositivoView, UiState};
use std::sync::Arc;

struct UiStateFake;
impl UiState for UiStateFake {
    fn dispositivos(&self) -> Vec<DispositivoView> {
        vec![DispositivoView { caminho: "/dev/sdb".to_owned(), descricao: "Teste — 32.0 GB (/dev/sdb)".to_owned() }]
    }
}

#[test]
fn builder_define_tema() {
    let app = NurApp::new(Arc::new(UiStateFake)).com_tema(ThemePreference::Claro);
    assert_eq!(app.tema(), ThemePreference::Claro);
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p ui`
Expected: FAIL.

- [ ] **Step 3: Implementar `crates/ui/src/app.rs`**

```rust
//! Aplicação egui do Nur (presenter; consome portas via Arc<dyn _>).

use crate::theme::{ThemeKit, ThemePreference};
use application::ports::UiState;
use std::sync::Arc;

/// App egui do Nur. Lê o estado por uma porta injetada.
pub struct NurApp {
    estado: Arc<dyn UiState>,
    tema: ThemePreference,
    tema_instalado: bool,
}

impl NurApp {
    /// Cria o app com o estado injetado (tema padrão: escuro).
    #[must_use]
    pub fn new(estado: Arc<dyn UiState>) -> Self {
        Self { estado, tema: ThemePreference::Escuro, tema_instalado: false }
    }

    /// Builder: define a preferência de tema inicial.
    #[must_use]
    pub fn com_tema(mut self, pref: ThemePreference) -> Self {
        self.tema = pref;
        self
    }

    #[cfg(test)]
    pub(crate) fn tema(&self) -> ThemePreference {
        self.tema
    }

    fn desenhar_central(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nur");
            ui.label("Formatador de Pendrive & Criador de Boot");
            if ui.button("Alternar tema").clicked() {
                self.tema = self.tema.alternar();
                self.tema_instalado = false;
            }
            ui.separator();
            ui.label("Dispositivos detectados:");
            for d in self.estado.dispositivos() {
                ui.label(d.descricao);
            }
        });
    }
}

impl eframe::App for NurApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.tema_instalado {
            ThemeKit::install(ctx, self.tema);
            self.tema_instalado = true;
        }
        self.desenhar_central(ctx);
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Atualizar `crates/ui/src/lib.rs`**

```rust
//! Camada de apresentação (egui) do Nur.

mod app;
pub mod theme;

pub use app::NurApp;
```

- [ ] **Step 5: Rodar e ver passar**

Run: `cargo test -p ui`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/ui
git commit -m "feat(ui): NurApp com builder de tema e painel central"
```

---

### Task 12: app — composition root e janela (binário `nur`)

**Files:**
- Create: `crates/app/Cargo.toml`, `crates/app/src/main.rs`, `crates/app/src/window.rs`

**Interfaces:**
- Consumes: `NurApp`, `DiskServiceStub`, `ListarDispositivos`, `UiState`, `DispositivoView`.
- Produces: binário `nur`. Um adapter local `UiStateAoVivo` que implementa `UiState` chamando `ListarDispositivos` (do stub) e cacheando o resultado.

- [ ] **Step 1: Criar `crates/app/Cargo.toml`**

```toml
[package]
name = "app"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
default-run = "nur"

[[bin]]
name = "nur"
path = "src/main.rs"

[dependencies]
domain = { path = "../domain" }
application = { path = "../application" }
infrastructure = { path = "../infrastructure" }
ui = { path = "../ui" }
eframe.workspace = true
egui.workspace = true
tokio.workspace = true
anyhow.workspace = true

[lints]
workspace = true
```

- [ ] **Step 2: Criar `crates/app/src/window.rs`**

```rust
//! Adapter de estado da UI e bootstrap da janela eframe.

use application::ports::{DispositivoView, UiState};
use application::use_cases::ListarDispositivos;
use infrastructure::stub::DiskServiceStub;
use std::sync::Arc;

/// Estado da UI alimentado pelo caso de uso (sobre o stub nesta fase).
pub struct UiStateAoVivo {
    dispositivos: Vec<DispositivoView>,
}

impl UiStateAoVivo {
    /// Monta o estado executando a listagem uma vez.
    ///
    /// # Errors
    /// Propaga falha do caso de uso.
    pub fn montar() -> anyhow::Result<Self> {
        let uc = ListarDispositivos::new(Arc::new(DiskServiceStub::new()));
        let dispositivos = uc.executar()?;
        Ok(Self { dispositivos })
    }
}

impl UiState for UiStateAoVivo {
    fn dispositivos(&self) -> Vec<DispositivoView> {
        self.dispositivos.clone()
    }
}

/// Abre a janela do Nur. Bloqueia até o usuário fechar.
///
/// # Errors
/// Retorna erro se o eframe falhar ao iniciar.
pub fn abrir(estado: Arc<dyn UiState>) -> anyhow::Result<()> {
    let opcoes = eframe::NativeOptions::default();
    eframe::run_native(
        "Nur",
        opcoes,
        Box::new(|_cc| Ok(Box::new(ui::NurApp::new(estado)))),
    )
    .map_err(|e| anyhow::anyhow!("falha ao iniciar a janela: {e}"))
}
```

- [ ] **Step 3: Criar `crates/app/src/main.rs`**

```rust
//! Composition root do Nur: monta os adapters e abre a janela.

mod window;

use std::sync::Arc;

fn main() -> std::process::ExitCode {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("erro ao criar runtime: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let _guard = runtime.enter();

    let estado = match window::UiStateAoVivo::montar() {
        Ok(estado) => Arc::new(estado),
        Err(e) => {
            eprintln!("erro ao montar estado: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    match window::abrir(estado) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
```

- [ ] **Step 4: Compilar o workspace inteiro**

Run: `cargo build`
Expected: compila todos os crates sem erros.

- [ ] **Step 5: Rodar o app**

Run: `cargo run -p app` (ou `cargo run --bin nur`)
Expected: abre uma janela "Nur" mostrando os 2 dispositivos stub e um botão "Alternar tema" que troca claro/escuro.

- [ ] **Step 6: Commit**

```bash
git add crates/app
git commit -m "feat(app): composition root e janela (binário nur)"
```

---

### Task 13: CI — workflow de qualidade

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:** nenhuma (infra de CI).

- [ ] **Step 1: Criar `.github/workflows/ci.yml`**

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
env:
  RUSTFLAGS: "-D warnings"
jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt }
      - run: cargo fmt --all -- --check
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy }
      - run: sudo apt-get update && sudo apt-get install -y libgtk-3-dev libxkbcommon-dev
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets -- -D warnings
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: sudo apt-get update && sudo apt-get install -y libgtk-3-dev libxkbcommon-dev
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace
  line-limit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo xtask line-limit
```

- [ ] **Step 2: Validar localmente os comandos do CI**

Run: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo xtask line-limit`
Expected: todos passam. (Corrigir quaisquer avisos pedantic apontados pelo clippy antes de commitar.)

- [ ] **Step 3: Commit**

```bash
git add .github
git commit -m "ci: fmt, clippy, test e line-limit"
```

---

## Definition of Done (Fase 1)

- [ ] `cargo run --bin nur` abre a janela do Nur com os 2 dispositivos stub e o botão de tema funcionando (claro/escuro).
- [ ] `cargo test --workspace` passa.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passa (sem-pânico, missing_docs, unsafe=forbid respeitados).
- [ ] `cargo xtask line-limit` passa (nenhum `.rs` ≥ 200 linhas).
- [ ] As fronteiras hexagonais estão estabelecidas: `ui` não depende de `infrastructure`; só `app` conhece todos os crates.

## Próximas fases (fora deste plano)

- **Plano 2 — UI completa:** traduzir o protótipo `desktop_app_for_form_1_1.html` para egui (seletor de dispositivo, modo bootável/formatar, drop de ISO, opções, modal "digite APAGAR", barra de progresso com fases, animações) ainda sobre adapters stub.
- **Plano 3 — Backend Linux real:** adapter `udisks2`/`zbus` (enumeração + `Block.OpenDevice` polkit), `IsoInspector` (detecção isohybrid/Windows), `RawWriteStrategy` com `fsync`/verificação, ponte tokio→egui para progresso. Substitui o stub pelo adapter real via o builder do `NurApp`.
- **Spike Windows** (paralelo): validar FAT32 + split do `install.wim` + boot em VM (ADR 0006).
