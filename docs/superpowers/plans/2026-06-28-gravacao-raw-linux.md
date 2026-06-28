# Gravação raw da ISO (Linux) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Gravar de verdade uma ISO Linux isohybrid num pendrive (modo Boot), com seleção nativa de ISO, detecção do tipo, abertura privilegiada via udisks2/polkit, progresso real, cancelamento e verificação.

**Architecture:** Hexagonal. A UI (presenter) lê estado por `UiState` e dispara ações por uma nova porta `UiCommands`. O `app` (composition root) spawna tasks tokio que rodam o caso de uso `CreateBootable` (orquestra `IsoInspector` + `BootableWriter`) e publicam progresso num `Arc<RwLock<WriteState>>` que a UI lê — mesma ponte da Fase 3. A lógica arriscada (zbus/fd) fica fina; a lógica de cópia+verificação mora num `RawCopier` testável sobre `Read`/`Write`/`Seek` genéricos.

**Tech Stack:** Rust 2024, egui/eframe 0.35, tokio, async-trait, `zbus 5` (blocking, para `OpenDevice`), `rfd` (diálogo nativo de arquivo).

## Global Constraints

- Edição Rust 2024, `rust-version = 1.88`; workspace de crates `domain → application → infrastructure → ui → app`.
- **OOP estrito:** sem função livre exceto `fn main`; helpers são associated functions de struct.
- **Código em inglês**; comentários, logs e textos de UI em **pt-BR** (com acentuação correta).
- **Zero campos `pub`** em structs (getters); enforçado por `cargo xtask pub-fields`.
- **Máx. 199 linhas por arquivo `.rs`**; enforçado por `cargo xtask line-limit`.
- `unsafe_code = forbid` (sem `unsafe`); sem `unwrap`/`expect`/`panic` fora de `#[cfg(test)]`.
- `missing_docs = deny`, `unreachable_pub = deny` — todo item público documentado.
- Testes em arquivo irmão: `foo.rs` → `foo/tests.rs` com `#[cfg(test)] mod tests;` no fim de `foo.rs`.
- Clippy `all=deny` + `pedantic=warn`; o gate é `cargo clippy --workspace --all-targets -- -D warnings`.
- A GUI **nunca** roda como root; privilégio só via udisks2/polkit `Block.OpenDevice`.
- Gate completo antes de cada commit relevante: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask check`.

---

### Task 1: `domain::IsoKind`

**Files:**
- Create: `crates/domain/src/iso_kind.rs`
- Create: `crates/domain/src/iso_kind/tests.rs`
- Modify: `crates/domain/src/lib.rs`

**Interfaces:**
- Produces: `domain::IsoKind` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum IsoKind { Isohybrid, Unsupported }`.

- [ ] **Step 1: Write the failing test**

`crates/domain/src/iso_kind/tests.rs`:
```rust
use super::IsoKind;

#[test]
fn variants_are_distinct() {
    assert_ne!(IsoKind::Isohybrid, IsoKind::Unsupported);
    assert_eq!(IsoKind::Isohybrid, IsoKind::Isohybrid);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p domain iso_kind`
Expected: FAIL (`unresolved import super::IsoKind`).

- [ ] **Step 3: Implement**

`crates/domain/src/iso_kind.rs`:
```rust
//! Classificação de uma imagem ISO quanto à gravação.

/// Como a ISO pode ser gravada no pendrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsoKind {
    /// Isohybrid: gravável por cópia raw byte-a-byte (maioria das ISOs Linux).
    Isohybrid,
    /// Não gravável por raw neste incremento (ex.: Windows/UDF).
    Unsupported,
}

#[cfg(test)]
mod tests;
```

Add to `crates/domain/src/lib.rs` (a module declaration and re-export, following the existing pattern):
```rust
mod iso_kind;
pub use iso_kind::IsoKind;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p domain iso_kind`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/domain/src/iso_kind.rs crates/domain/src/iso_kind/tests.rs crates/domain/src/lib.rs
git commit -m "feat(domain): IsoKind (isohybrid vs unsupported)"
```

---

### Task 2: erros `IsoError` e `WriteError`

**Files:**
- Modify: `crates/application/src/errors.rs`

**Interfaces:**
- Produces:
  - `application::errors::IsoError` — `#[derive(Debug, thiserror::Error)] pub enum IsoError { Io(String) }`.
  - `application::errors::WriteError` — `pub enum WriteError { Unauthorized, DeviceBusy, DeviceTooSmall, Io(String), VerificationMismatch, Cancelled }`.

- [ ] **Step 1: Write the failing test**

Append to a new `#[cfg(test)] mod tests` block isn't used in `errors.rs` today; instead add a sibling test file. Create `crates/application/src/errors/tests.rs`:
```rust
use super::{IsoError, WriteError};

#[test]
fn write_error_messages_are_in_ptbr() {
    assert_eq!(WriteError::Unauthorized.to_string(), "autorização negada");
    assert_eq!(WriteError::Cancelled.to_string(), "operação cancelada");
    assert_eq!(
        WriteError::DeviceTooSmall.to_string(),
        "o dispositivo é menor que a imagem"
    );
}

#[test]
fn iso_error_wraps_message() {
    assert_eq!(
        IsoError::Io("x".to_owned()).to_string(),
        "falha ao ler a ISO: x"
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p application errors`
Expected: FAIL (types not found).

- [ ] **Step 3: Implement**

Append to `crates/application/src/errors.rs`:
```rust
/// Falhas ao inspecionar/ler a imagem ISO.
#[derive(Debug, thiserror::Error)]
pub enum IsoError {
    /// Falha de IO ao ler a ISO.
    #[error("falha ao ler a ISO: {0}")]
    Io(String),
}

/// Falhas ao gravar a imagem no dispositivo.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    /// O polkit negou a autorização para abrir o dispositivo.
    #[error("autorização negada")]
    Unauthorized,
    /// O dispositivo está em uso (montado ou aberto por outro processo).
    #[error("dispositivo ocupado")]
    DeviceBusy,
    /// O dispositivo é menor que a imagem.
    #[error("o dispositivo é menor que a imagem")]
    DeviceTooSmall,
    /// Falha de IO durante a gravação ou a leitura de verificação.
    #[error("falha de gravação: {0}")]
    Io(String),
    /// A releitura do dispositivo não bate com a imagem.
    #[error("verificação falhou: o conteúdo gravado difere da imagem")]
    VerificationMismatch,
    /// O usuário cancelou a operação.
    #[error("operação cancelada")]
    Cancelled,
}
```

And add at the very end of `crates/application/src/errors.rs`:
```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p application errors`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/application/src/errors.rs crates/application/src/errors/tests.rs
git commit -m "feat(application): IsoError e WriteError"
```

---

### Task 3: tipos de gravação (`write` module)

**Files:**
- Create: `crates/application/src/ports/write.rs`
- Create: `crates/application/src/ports/write/tests.rs`
- Modify: `crates/application/src/ports/mod.rs`

**Interfaces:**
- Consumes: `domain::{ByteSize, DevicePath, IsoKind}`.
- Produces (all in `application::ports`):
  - `WritePhase` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum WritePhase { Preparing, Writing, Verifying }`.
  - `WriteProgress` — `pub struct WriteProgress { phase, done: u64, total: u64 }` (private fields + getters `phase()`, `done()`, `total()`, ctor `new`).
  - `WriteState` — `#[derive(Debug, Clone, PartialEq)] pub enum WriteState { Idle, Preparing, Writing { done: u64, total: u64 }, Verifying { done: u64, total: u64 }, Done, Failed(String), Cancelled }`.
  - `WriteRequest` — `pub struct WriteRequest { iso_path: PathBuf, device: DevicePath }` (private + getters `iso_path()`, `device()`, ctor `new`).
  - `IsoSelection` — `pub struct IsoSelection { name: String, size: ByteSize, kind: IsoKind }` (private + getters `name()`, `size()`, `kind()`, ctor `new`).
  - `CancelFlag` — `#[derive(Clone, Default)] pub struct CancelFlag(Arc<AtomicBool>)` with `new()`, `cancel(&self)`, `is_cancelled(&self) -> bool`.
  - `ProgressSink` — `pub trait ProgressSink: Send + Sync { fn report(&self, progress: WriteProgress); }`.

- [ ] **Step 1: Write the failing test**

`crates/application/src/ports/write/tests.rs`:
```rust
use super::{CancelFlag, WritePhase, WriteProgress};

#[test]
fn cancel_flag_starts_unset_and_latches() {
    let flag = CancelFlag::new();
    assert!(!flag.is_cancelled());
    flag.cancel();
    assert!(flag.is_cancelled());
}

#[test]
fn cancel_flag_clone_shares_state() {
    let flag = CancelFlag::new();
    let clone = flag.clone();
    flag.cancel();
    assert!(clone.is_cancelled());
}

#[test]
fn write_progress_exposes_fields() {
    let p = WriteProgress::new(WritePhase::Writing, 10, 100);
    assert_eq!(p.phase(), WritePhase::Writing);
    assert_eq!(p.done(), 10);
    assert_eq!(p.total(), 100);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p application ports::write`
Expected: FAIL (module missing).

- [ ] **Step 3: Implement**

`crates/application/src/ports/write.rs`:
```rust
//! Tipos e portas auxiliares da gravação (estado, progresso, cancelamento).

use domain::{ByteSize, DevicePath, IsoKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Fase em andamento de uma gravação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritePhase {
    /// Abrindo/preparando o dispositivo.
    Preparing,
    /// Escrevendo a imagem.
    Writing,
    /// Relendo e comparando com a imagem.
    Verifying,
}

/// Progresso instantâneo reportado pelo gravador.
#[derive(Debug, Clone, Copy)]
pub struct WriteProgress {
    phase: WritePhase,
    done: u64,
    total: u64,
}

impl WriteProgress {
    /// Cria um progresso.
    #[must_use]
    pub fn new(phase: WritePhase, done: u64, total: u64) -> Self {
        Self { phase, done, total }
    }

    /// Fase atual.
    #[must_use]
    pub fn phase(&self) -> WritePhase {
        self.phase
    }

    /// Bytes processados na fase.
    #[must_use]
    pub fn done(&self) -> u64 {
        self.done
    }

    /// Total de bytes da fase.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.total
    }
}

/// Estado da gravação lido pela UI.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteState {
    /// Nada em andamento.
    Idle,
    /// Preparando o dispositivo.
    Preparing,
    /// Gravando (determinado).
    Writing {
        /// Bytes gravados.
        done: u64,
        /// Total de bytes.
        total: u64,
    },
    /// Verificando (determinado).
    Verifying {
        /// Bytes verificados.
        done: u64,
        /// Total de bytes.
        total: u64,
    },
    /// Concluído com sucesso.
    Done,
    /// Falhou (mensagem para o usuário).
    Failed(String),
    /// Cancelado pelo usuário.
    Cancelled,
}

/// Pedido de gravação: qual ISO em qual dispositivo.
#[derive(Debug, Clone)]
pub struct WriteRequest {
    iso_path: PathBuf,
    device: DevicePath,
}

impl WriteRequest {
    /// Cria o pedido.
    #[must_use]
    pub fn new(iso_path: PathBuf, device: DevicePath) -> Self {
        Self { iso_path, device }
    }

    /// Caminho da imagem ISO.
    #[must_use]
    pub fn iso_path(&self) -> &Path {
        &self.iso_path
    }

    /// Dispositivo de destino.
    #[must_use]
    pub fn device(&self) -> &DevicePath {
        &self.device
    }
}

/// ISO escolhida pelo usuário (projeção para a UI).
#[derive(Debug, Clone)]
pub struct IsoSelection {
    name: String,
    size: ByteSize,
    kind: IsoKind,
}

impl IsoSelection {
    /// Cria a seleção.
    #[must_use]
    pub fn new(name: String, size: ByteSize, kind: IsoKind) -> Self {
        Self { name, size, kind }
    }

    /// Nome do arquivo (sem diretório).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Tamanho da ISO.
    #[must_use]
    pub fn size(&self) -> ByteSize {
        self.size
    }

    /// Classificação da ISO.
    #[must_use]
    pub fn kind(&self) -> IsoKind {
        self.kind
    }
}

/// Sinalizador de cancelamento compartilhável entre threads.
#[derive(Clone, Default)]
pub struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    /// Cria um sinalizador não-acionado.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Aciona o cancelamento (latched).
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Indica se o cancelamento foi solicitado.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Destino do progresso reportado pelo gravador.
pub trait ProgressSink: Send + Sync {
    /// Recebe um progresso instantâneo.
    fn report(&self, progress: WriteProgress);
}

#[cfg(test)]
mod tests;
```

Modify `crates/application/src/ports/mod.rs` — add module and re-exports:
```rust
mod write;
pub use write::{
    CancelFlag, IsoSelection, ProgressSink, WritePhase, WriteProgress, WriteRequest, WriteState,
};
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p application ports::write`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/application/src/ports/write.rs crates/application/src/ports/write/tests.rs crates/application/src/ports/mod.rs
git commit -m "feat(application): tipos de gravação (WriteState, progresso, cancelamento)"
```

---

### Task 4: portas `IsoInspector`, `BootableWriter`, `UiCommands` e extensão de `UiState`

**Files:**
- Create: `crates/application/src/ports/iso_inspector.rs`
- Create: `crates/application/src/ports/bootable_writer.rs`
- Create: `crates/application/src/ports/ui_commands.rs`
- Modify: `crates/application/src/ports/ui_state.rs`
- Modify: `crates/application/src/ports/mod.rs`

**Interfaces:**
- Consumes: `domain::{DevicePath, IsoKind}`, `application::errors::{IsoError, WriteError}`, `application::ports::{WriteRequest, WriteState, IsoSelection, ProgressSink, CancelFlag}`.
- Produces:
  - `IsoInspector` (`#[async_trait]`): `async fn classify(&self, iso: &Path) -> Result<IsoKind, IsoError>`.
  - `BootableWriter` (`#[async_trait]`): `async fn write(&self, request: &WriteRequest, sink: Arc<dyn ProgressSink>, cancel: &CancelFlag) -> Result<(), WriteError>`.
  - `UiCommands`: `fn pick_iso(&self)`, `fn start(&self, device: DevicePath)`, `fn cancel(&self)`.
  - `UiState` gains **default** methods `write_state(&self) -> WriteState { WriteState::Idle }` and `selected_iso(&self) -> Option<IsoSelection> { None }` (defaults keep existing impls compiling).

- [ ] **Step 1: Write the failing test**

Add to `crates/application/src/ports/ui_state/tests.rs` (file already exists) a test that the defaults hold for a minimal impl:
```rust
#[test]
fn ui_state_defaults_are_idle_and_none() {
    struct Minimal;
    impl UiState for Minimal {
        fn device_list(&self) -> DeviceListState {
            DeviceListState::Loading
        }
    }
    let s = Minimal;
    assert_eq!(s.write_state(), application::ports::WriteState::Idle);
    assert!(s.selected_iso().is_none());
}
```
(Adjust imports at top of that test file to bring `UiState`, `DeviceListState`, and `WriteState` into scope — `use super::*;` plus `use crate::ports::WriteState;`.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p application ui_state`
Expected: FAIL (methods don't exist yet).

- [ ] **Step 3: Implement**

In `crates/application/src/ports/ui_state.rs`, extend the trait (keep `device_list` required, add the two defaulted methods):
```rust
use crate::ports::{IsoSelection, WriteState};

/// Estado lido pela UI a cada frame.
pub trait UiState: Send + Sync {
    /// Estado atual da lista de dispositivos.
    fn device_list(&self) -> DeviceListState;

    /// Estado atual da gravação (padrão: ociosa).
    fn write_state(&self) -> WriteState {
        WriteState::Idle
    }

    /// ISO selecionada pelo usuário, se houver (padrão: nenhuma).
    fn selected_iso(&self) -> Option<IsoSelection> {
        None
    }
}
```

`crates/application/src/ports/iso_inspector.rs`:
```rust
//! Porta de inspeção de ISO (classifica o tipo da imagem).

use crate::errors::IsoError;
use domain::IsoKind;
use std::path::Path;

/// Inspeciona uma imagem ISO e a classifica.
#[async_trait::async_trait]
pub trait IsoInspector: Send + Sync {
    /// Classifica a ISO no caminho dado.
    ///
    /// # Errors
    /// Retorna [`IsoError`] se a leitura falhar.
    async fn classify(&self, iso: &Path) -> Result<IsoKind, IsoError>;
}
```

`crates/application/src/ports/bootable_writer.rs`:
```rust
//! Porta de gravação de ISO bootável num dispositivo.

use crate::errors::WriteError;
use crate::ports::{CancelFlag, ProgressSink, WriteRequest};
use std::sync::Arc;

/// Grava a imagem no dispositivo, reportando progresso e respeitando cancelamento.
#[async_trait::async_trait]
pub trait BootableWriter: Send + Sync {
    /// Grava e verifica a imagem do `request` no dispositivo de destino.
    ///
    /// # Errors
    /// Retorna [`WriteError`] em falha de autorização, IO, verificação ou cancelamento.
    async fn write(
        &self,
        request: &WriteRequest,
        sink: Arc<dyn ProgressSink>,
        cancel: &CancelFlag,
    ) -> Result<(), WriteError>;
}
```

`crates/application/src/ports/ui_commands.rs`:
```rust
//! Porta de comandos disparados pela UI (lado de escrita do app).

use domain::DevicePath;

/// Ações que a UI dispara; implementadas no composition root.
pub trait UiCommands: Send + Sync {
    /// Abre o diálogo nativo para escolher uma ISO e a inspeciona.
    fn pick_iso(&self);

    /// Inicia a gravação da ISO selecionada no dispositivo dado.
    fn start(&self, device: DevicePath);

    /// Solicita o cancelamento da gravação em andamento.
    fn cancel(&self);
}
```

Modify `crates/application/src/ports/mod.rs` — add modules and re-exports:
```rust
mod bootable_writer;
mod iso_inspector;
mod ui_commands;
pub use bootable_writer::BootableWriter;
pub use iso_inspector::IsoInspector;
pub use ui_commands::UiCommands;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p application` then `cargo clippy -p application --all-targets -- -D warnings`
Expected: PASS / no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/application/src/ports/
git commit -m "feat(application): portas IsoInspector, BootableWriter, UiCommands + UiState estendido"
```

---

### Task 5: caso de uso `CreateBootable`

**Files:**
- Create: `crates/application/src/use_cases/create_bootable.rs`
- Create: `crates/application/src/use_cases/create_bootable/tests.rs`
- Modify: `crates/application/src/use_cases/mod.rs`

**Interfaces:**
- Consumes: `IsoInspector`, `BootableWriter`, `WriteRequest`, `ProgressSink`, `CancelFlag`, `IsoKind`, `WriteError`.
- Produces: `CreateBootable` with `new(inspector: Arc<dyn IsoInspector>, writer: Arc<dyn BootableWriter>) -> Self` and `async fn execute(&self, request: WriteRequest, sink: Arc<dyn ProgressSink>, cancel: CancelFlag) -> Result<(), WriteError>`. On `IsoKind::Unsupported` returns `WriteError::Io("ISO não suportada para gravação raw…")` (so the app surfaces a clear `Failed`).

- [ ] **Step 1: Write the failing test**

`crates/application/src/use_cases/create_bootable/tests.rs`:
```rust
use super::CreateBootable;
use crate::errors::{IsoError, WriteError};
use crate::ports::{
    BootableWriter, CancelFlag, IsoInspector, ProgressSink, WriteProgress, WriteRequest,
};
use domain::{DevicePath, IsoKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct FixedInspector(IsoKind);
#[async_trait::async_trait]
impl IsoInspector for FixedInspector {
    async fn classify(&self, _iso: &Path) -> Result<IsoKind, IsoError> {
        Ok(self.0)
    }
}

struct SpyWriter(Arc<AtomicBool>);
#[async_trait::async_trait]
impl BootableWriter for SpyWriter {
    async fn write(
        &self,
        _r: &WriteRequest,
        _s: Arc<dyn ProgressSink>,
        _c: &CancelFlag,
    ) -> Result<(), WriteError> {
        self.0.store(true, Ordering::SeqCst);
        Ok(())
    }
}

struct NoopSink;
impl ProgressSink for NoopSink {
    fn report(&self, _p: WriteProgress) {}
}

fn request() -> WriteRequest {
    WriteRequest::new(PathBuf::from("/tmp/x.iso"), DevicePath::new("/dev/sdb".to_owned()))
}

#[tokio::test]
async fn isohybrid_invokes_writer() {
    let called = Arc::new(AtomicBool::new(false));
    let uc = CreateBootable::new(
        Arc::new(FixedInspector(IsoKind::Isohybrid)),
        Arc::new(SpyWriter(Arc::clone(&called))),
    );
    let result = uc
        .execute(request(), Arc::new(NoopSink), CancelFlag::new())
        .await;
    assert!(result.is_ok());
    assert!(called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn unsupported_is_rejected_without_writing() {
    let called = Arc::new(AtomicBool::new(false));
    let uc = CreateBootable::new(
        Arc::new(FixedInspector(IsoKind::Unsupported)),
        Arc::new(SpyWriter(Arc::clone(&called))),
    );
    let result = uc
        .execute(request(), Arc::new(NoopSink), CancelFlag::new())
        .await;
    assert!(result.is_err());
    assert!(!called.load(Ordering::SeqCst));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p application create_bootable`
Expected: FAIL (type missing).

- [ ] **Step 3: Implement**

`crates/application/src/use_cases/create_bootable.rs`:
```rust
//! Caso de uso: criar um pendrive bootável a partir de uma ISO.

use crate::errors::WriteError;
use crate::ports::{BootableWriter, CancelFlag, IsoInspector, ProgressSink, WriteRequest};
use domain::IsoKind;
use std::sync::Arc;

/// Orquestra a classificação da ISO e a gravação no dispositivo.
pub struct CreateBootable {
    inspector: Arc<dyn IsoInspector>,
    writer: Arc<dyn BootableWriter>,
}

impl CreateBootable {
    /// Cria o caso de uso com as portas injetadas.
    #[must_use]
    pub fn new(inspector: Arc<dyn IsoInspector>, writer: Arc<dyn BootableWriter>) -> Self {
        Self { inspector, writer }
    }

    /// Classifica a ISO e, se gravável por raw, grava e verifica.
    ///
    /// # Errors
    /// Retorna [`WriteError`] se a ISO não for suportada ou se a gravação falhar.
    pub async fn execute(
        &self,
        request: WriteRequest,
        sink: Arc<dyn ProgressSink>,
        cancel: CancelFlag,
    ) -> Result<(), WriteError> {
        let kind = self
            .inspector
            .classify(request.iso_path())
            .await
            .map_err(|e| WriteError::Io(e.to_string()))?;
        if kind == IsoKind::Unsupported {
            return Err(WriteError::Io(
                "ISO não suportada para gravação raw (provavelmente Windows); \
                 modo de extração ainda não disponível"
                    .to_owned(),
            ));
        }
        self.writer.write(&request, sink, &cancel).await
    }
}

#[cfg(test)]
mod tests;
```

Add to `crates/application/src/use_cases/mod.rs`:
```rust
mod create_bootable;
pub use create_bootable::CreateBootable;
```
(Keep the existing `ListDevices` export.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p application create_bootable`
Expected: PASS (both tests).

- [ ] **Step 5: Commit**

```bash
git add crates/application/src/use_cases/
git commit -m "feat(application): caso de uso CreateBootable (classifica + grava)"
```

---

### Task 6: `IsoFileInspector` (infra, detecção isohybrid)

**Files:**
- Create: `crates/infrastructure/src/linux/iso_file_inspector.rs`
- Create: `crates/infrastructure/src/linux/iso_file_inspector/tests.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Consumes: `IsoInspector`, `IsoError`, `IsoKind`.
- Produces: `infrastructure::linux::IsoFileInspector` with `new() -> Self` + `Default`; impl `IsoInspector`. Internal pure classifier `fn classify_bytes(mbr: &[u8; 512], pvd_cd001: bool) -> IsoKind` testable directly.

**Detecção (de `docs/pesquisa/03` §1):** isohybrid quando o byte 510==0x55 e 511==0xAA **e** existe ≥1 entrada de partição não-vazia nas 4 entradas de 16 bytes a partir de 0x1BE (uma entrada é "não-vazia" se algum dos 16 bytes ≠ 0). O marcador `CD001` em 0x8001 confirma ISO9660 (lido à parte). Regra: `0x55AA + ≥1 partição` → `Isohybrid`; senão `Unsupported`.

- [ ] **Step 1: Write the failing test**

`crates/infrastructure/src/linux/iso_file_inspector/tests.rs`:
```rust
use super::IsoFileInspector;
use domain::IsoKind;

fn mbr_with_partition() -> [u8; 512] {
    let mut b = [0u8; 512];
    b[510] = 0x55;
    b[511] = 0xAA;
    // 1ª entrada de partição não-vazia (offset 0x1BE), tipo 0x83.
    b[0x1BE + 4] = 0x83;
    b
}

#[test]
fn isohybrid_when_signature_and_partition_present() {
    assert_eq!(
        IsoFileInspector::classify_bytes(&mbr_with_partition(), true),
        IsoKind::Isohybrid
    );
}

#[test]
fn unsupported_when_no_signature() {
    let mut b = mbr_with_partition();
    b[510] = 0x00;
    assert_eq!(IsoFileInspector::classify_bytes(&b, true), IsoKind::Unsupported);
}

#[test]
fn unsupported_when_no_partition() {
    let mut b = [0u8; 512];
    b[510] = 0x55;
    b[511] = 0xAA;
    assert_eq!(IsoFileInspector::classify_bytes(&b, true), IsoKind::Unsupported);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p infrastructure iso_file_inspector`
Expected: FAIL (type missing).

- [ ] **Step 3: Implement**

`crates/infrastructure/src/linux/iso_file_inspector.rs`:
```rust
//! Inspeção de ISO lendo os primeiros setores (detecção isohybrid). Rust puro.

use application::errors::IsoError;
use application::ports::IsoInspector;
use domain::IsoKind;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Classifica ISOs lendo o MBR (0x1BE/0x55AA) e o PVD (`CD001` em 0x8001).
pub struct IsoFileInspector;

impl IsoFileInspector {
    /// Cria o inspetor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Regra: assinatura 0x55AA + ≥1 partição não-vazia em 0x1BE → isohybrid.
    fn classify_bytes(mbr: &[u8; 512], _cd001: bool) -> IsoKind {
        let signature = mbr[510] == 0x55 && mbr[511] == 0xAA;
        let has_partition = (0..4).any(|i| {
            let start = 0x1BE + i * 16;
            mbr[start..start + 16].iter().any(|&b| b != 0)
        });
        if signature && has_partition {
            IsoKind::Isohybrid
        } else {
            IsoKind::Unsupported
        }
    }

    fn read_and_classify(path: PathBuf) -> Result<IsoKind, IsoError> {
        let mut file = std::fs::File::open(&path).map_err(|e| IsoError::Io(e.to_string()))?;
        let mut mbr = [0u8; 512];
        file.read_exact(&mut mbr)
            .map_err(|e| IsoError::Io(e.to_string()))?;
        // CD001 em 0x8001 (setor 16). Best-effort: ausência não impede a regra.
        let mut cd = [0u8; 5];
        let cd001 = file
            .seek(SeekFrom::Start(0x8001))
            .and_then(|_| file.read_exact(&mut cd).map(|()| &cd == b"CD001"))
            .unwrap_or(false);
        Ok(Self::classify_bytes(&mbr, cd001))
    }
}

impl Default for IsoFileInspector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl IsoInspector for IsoFileInspector {
    async fn classify(&self, iso: &Path) -> Result<IsoKind, IsoError> {
        let path = iso.to_path_buf();
        tokio::task::spawn_blocking(move || Self::read_and_classify(path))
            .await
            .map_err(|e| IsoError::Io(e.to_string()))?
    }
}

#[cfg(test)]
mod tests;
```

Add to `crates/infrastructure/src/linux/mod.rs`:
```rust
mod iso_file_inspector;
pub use iso_file_inspector::IsoFileInspector;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p infrastructure iso_file_inspector`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/infrastructure/src/linux/iso_file_inspector.rs crates/infrastructure/src/linux/iso_file_inspector/tests.rs crates/infrastructure/src/linux/mod.rs
git commit -m "feat(infrastructure): IsoFileInspector (detecção isohybrid)"
```

---

### Task 7: `RawCopier` — cópia + verificação testável

**Files:**
- Create: `crates/infrastructure/src/linux/raw_copier.rs`
- Create: `crates/infrastructure/src/linux/raw_copier/tests.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Consumes: `ProgressSink`, `CancelFlag`, `WriteProgress`, `WritePhase`, `WriteError`.
- Produces: `RawCopier` with associated functions operating over generic IO:
  - `fn copy<R: Read, W: Write>(source: &mut R, dest: &mut W, total: u64, sink: &dyn ProgressSink, cancel: &CancelFlag) -> Result<(), WriteError>` — copies in 4 MiB chunks, reports `WritePhase::Writing` progress, checks `cancel` between chunks (returns `WriteError::Cancelled`), flushes at end.
  - `fn verify<A: Read, B: Read>(written: &mut A, original: &mut B, total: u64, sink: &dyn ProgressSink, cancel: &CancelFlag) -> Result<(), WriteError>` — re-reads `total` bytes from `written`, compares against `original`; mismatch → `WriteError::VerificationMismatch`; reports `WritePhase::Verifying`.
- The const `CHUNK: usize = 4 * 1024 * 1024;` lives here.

- [ ] **Step 1: Write the failing test**

`crates/infrastructure/src/linux/raw_copier/tests.rs`:
```rust
use super::RawCopier;
use application::errors::WriteError;
use application::ports::{CancelFlag, ProgressSink, WriteProgress};
use std::io::Cursor;
use std::sync::Mutex;

struct RecordingSink(Mutex<Vec<u64>>);
impl ProgressSink for RecordingSink {
    fn report(&self, p: WriteProgress) {
        if let Ok(mut v) = self.0.lock() {
            v.push(p.done());
        }
    }
}

#[test]
fn copy_writes_all_bytes_and_reports_progress() {
    let data = vec![7u8; 10 * 1024 * 1024]; // 10 MiB → 3 chunks
    let mut src = Cursor::new(data.clone());
    let mut dst: Vec<u8> = Vec::new();
    let sink = RecordingSink(Mutex::new(Vec::new()));
    let cancel = CancelFlag::new();
    let r = RawCopier::copy(&mut src, &mut dst, data.len() as u64, &sink, &cancel);
    assert!(r.is_ok());
    assert_eq!(dst, data);
    let reported = sink.0.lock().unwrap();
    assert_eq!(*reported.last().unwrap(), data.len() as u64);
}

#[test]
fn copy_aborts_when_cancelled() {
    let data = vec![1u8; 10 * 1024 * 1024];
    let mut src = Cursor::new(data.clone());
    let mut dst: Vec<u8> = Vec::new();
    let sink = RecordingSink(Mutex::new(Vec::new()));
    let cancel = CancelFlag::new();
    cancel.cancel();
    let r = RawCopier::copy(&mut src, &mut dst, data.len() as u64, &sink, &cancel);
    assert!(matches!(r, Err(WriteError::Cancelled)));
}

#[test]
fn verify_detects_mismatch() {
    let original = vec![5u8; 8 * 1024 * 1024];
    let mut corrupted = original.clone();
    corrupted[100] = 0;
    let sink = RecordingSink(Mutex::new(Vec::new()));
    let cancel = CancelFlag::new();
    let r = RawCopier::verify(
        &mut Cursor::new(corrupted),
        &mut Cursor::new(original.clone()),
        original.len() as u64,
        &sink,
        &cancel,
    );
    assert!(matches!(r, Err(WriteError::VerificationMismatch)));
}

#[test]
fn verify_passes_on_identical() {
    let original = vec![9u8; 8 * 1024 * 1024];
    let sink = RecordingSink(Mutex::new(Vec::new()));
    let cancel = CancelFlag::new();
    let r = RawCopier::verify(
        &mut Cursor::new(original.clone()),
        &mut Cursor::new(original.clone()),
        original.len() as u64,
        &sink,
        &cancel,
    );
    assert!(r.is_ok());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p infrastructure raw_copier`
Expected: FAIL (type missing).

- [ ] **Step 3: Implement**

`crates/infrastructure/src/linux/raw_copier.rs`:
```rust
//! Cópia raw com progresso/cancelamento e verificação por releitura.
//! Genérico sobre IO para ser testável sem dispositivo real.

use application::errors::WriteError;
use application::ports::{CancelFlag, ProgressSink, WritePhase, WriteProgress};
use std::io::{Read, Write};

const CHUNK: usize = 4 * 1024 * 1024;

/// Rotinas de cópia/verificação sobre `Read`/`Write` genéricos.
pub struct RawCopier;

impl RawCopier {
    /// Copia `total` bytes de `source` para `dest` em blocos de 4 MiB.
    pub fn copy<R: Read, W: Write>(
        source: &mut R,
        dest: &mut W,
        total: u64,
        sink: &dyn ProgressSink,
        cancel: &CancelFlag,
    ) -> Result<(), WriteError> {
        let mut buf = vec![0u8; CHUNK];
        let mut done: u64 = 0;
        loop {
            if cancel.is_cancelled() {
                return Err(WriteError::Cancelled);
            }
            let n = source.read(&mut buf).map_err(|e| WriteError::Io(e.to_string()))?;
            if n == 0 {
                break;
            }
            dest.write_all(&buf[..n]).map_err(|e| WriteError::Io(e.to_string()))?;
            done += n as u64;
            sink.report(WriteProgress::new(WritePhase::Writing, done, total));
        }
        dest.flush().map_err(|e| WriteError::Io(e.to_string()))?;
        Ok(())
    }

    /// Relê `total` bytes de `written` e compara com `original`.
    pub fn verify<A: Read, B: Read>(
        written: &mut A,
        original: &mut B,
        total: u64,
        sink: &dyn ProgressSink,
        cancel: &CancelFlag,
    ) -> Result<(), WriteError> {
        let mut a = vec![0u8; CHUNK];
        let mut b = vec![0u8; CHUNK];
        let mut done: u64 = 0;
        while done < total {
            if cancel.is_cancelled() {
                return Err(WriteError::Cancelled);
            }
            let want = CHUNK.min((total - done) as usize);
            written.read_exact(&mut a[..want]).map_err(|e| WriteError::Io(e.to_string()))?;
            original.read_exact(&mut b[..want]).map_err(|e| WriteError::Io(e.to_string()))?;
            if a[..want] != b[..want] {
                return Err(WriteError::VerificationMismatch);
            }
            done += want as u64;
            sink.report(WriteProgress::new(WritePhase::Verifying, done, total));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
```

Add to `crates/infrastructure/src/linux/mod.rs`:
```rust
mod raw_copier;
pub use raw_copier::RawCopier;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p infrastructure raw_copier`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/infrastructure/src/linux/raw_copier.rs crates/infrastructure/src/linux/raw_copier/tests.rs crates/infrastructure/src/linux/mod.rs
git commit -m "feat(infrastructure): RawCopier (cópia + verificação testável)"
```

---

### Task 8: `Udisks2BlockWriter` (infra, fd via polkit)

**Files:**
- Create: `crates/infrastructure/src/linux/udisks2_block_writer.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`
- Modify: `Cargo.toml` (workspace deps) e `crates/infrastructure/Cargo.toml`
- Modify (se necessário): `deny.toml`

**Interfaces:**
- Consumes: `BootableWriter`, `WriteRequest`, `ProgressSink`, `CancelFlag`, `WriteError`, `WritePhase`/`WriteProgress`, `RawCopier`.
- Produces: `infrastructure::linux::Udisks2BlockWriter` with `new() -> Self` + `Default`; impl `BootableWriter`.

**Notas de implementação (camada fina — validação manual, sem teste unitário):**
- Abre o device via D-Bus, método `org.freedesktop.UDisks2.Block.OpenDevice` no objeto `/org/freedesktop/UDisks2/block_devices/<nome>` (derive `<nome>` de `request.device()` removendo o prefixo `/dev/`), usando **`zbus::blocking`** dentro de `tokio::task::spawn_blocking` (isola o async-io do zbus do runtime tokio — a dor da Fase 3 não se aplica: é 1 chamada). Argumentos: `("rw", options)` onde `options: HashMap<&str, zvariant::Value>` contém `"flags"` = `O_EXCL | O_CLOEXEC | O_SYNC` (`const FLAGS: i32 = 0x80 | 0x8_0000 | 0x1000;` — confira os valores de `/usr/include/asm-generic/fcntl.h` no alvo; comente-os).
- O reply traz um fd: `reply.body().deserialize::<zvariant::OwnedFd>()` → converta para `std::os::fd::OwnedFd` → `std::fs::File::from(owned)` (**`From<OwnedFd>`, sem `unsafe`**). Ajuste a conversão exata à versão instalada do `zbus`/`zvariant` (a API de fd mudou entre versões).
- Mapeie erros do polkit: se a mensagem D-Bus indicar `NotAuthorized` → `WriteError::Unauthorized`; `Busy`/`Device is mounted` → `WriteError::DeviceBusy`; demais → `WriteError::Io(msg)`.
- Antes de abrir: leia o tamanho da ISO (`std::fs::metadata(iso).len()`) e o tamanho do device (`/sys/block/<nome>/size` × 512); se `iso > device` → `WriteError::DeviceTooSmall`.
- Reporte `WritePhase::Preparing` antes de abrir.
- Grave: abra a ISO (`File::open`), chame `RawCopier::copy(&mut iso, &mut dev_file, iso_len, sink, cancel)`, depois `dev_file.sync_all()`.
- Verifique: **reabra** o device para leitura (nova `OpenDevice("rw"|"r")` ou `File::open("/dev/<nome>")` — reabrir evita cache do fd de escrita), reabra a ISO, chame `RawCopier::verify(&mut dev_read, &mut iso2, iso_len, sink, cancel)`.
- Tudo o que toca zbus/IO roda dentro de `spawn_blocking`; a `async fn write` apenas aguarda o join.

- [ ] **Step 1: Add dependencies**

In root `Cargo.toml` `[workspace.dependencies]` add:
```toml
zbus = "5"
```
In `crates/infrastructure/Cargo.toml` `[dependencies]` add:
```toml
zbus = { workspace = true }
```

- [ ] **Step 2: Implement the writer**

Write `crates/infrastructure/src/linux/udisks2_block_writer.rs` following the notes above. Keep the file ≤199 lines (split a private `open_device` helper and a private `device_size` helper as associated functions if needed; if it would exceed, move helpers into a sibling `udisks2_block_writer/open.rs` submodule). Register it in `crates/infrastructure/src/linux/mod.rs`:
```rust
mod udisks2_block_writer;
pub use udisks2_block_writer::Udisks2BlockWriter;
```

- [ ] **Step 3: Build + lints + license check**

Run: `cargo build -p infrastructure`
Run: `cargo clippy -p infrastructure --all-targets -- -D warnings`
Run: `cargo deny check 2>&1 | tail -20`
Expected: builds; no warnings. If `cargo deny` fails on a new license pulled by `zbus`, add that exact SPDX id to the `allow` list in `deny.toml` and document why (one line). Re-run until clean.

- [ ] **Step 4: Manual validation (registre no PR, fora do CI)**

With a spare USB stick and a Linux ISO:
```bash
cargo run --bin nur
# selecionar a ISO, selecionar o pendrive, confirmar APAGAR, gravar
```
Confirm the polkit prompt appears, progress advances, verification passes. Then boot the stick in QEMU:
```bash
qemu-system-x86_64 -m 2048 -drive file=/dev/sdX,format=raw,if=virtio -boot menu=on
```
Expected: a ISO boota. **Não há teste automatizado nesta task** (precisa de device + polkit); a lógica testável está no `RawCopier` (Task 7).

- [ ] **Step 5: Commit**

```bash
git add crates/infrastructure/src/linux/udisks2_block_writer.rs crates/infrastructure/src/linux/mod.rs Cargo.toml crates/infrastructure/Cargo.toml deny.toml
git commit -m "feat(infrastructure): Udisks2BlockWriter (fd via polkit + cópia/verify)"
```

---

### Task 9: `RfdIsoPicker` (infra, diálogo nativo)

**Files:**
- Create: `crates/infrastructure/src/picker/mod.rs`
- Create: `crates/infrastructure/src/picker/rfd_iso_picker.rs`
- Modify: `crates/infrastructure/src/lib.rs`
- Modify: `Cargo.toml` (workspace) e `crates/infrastructure/Cargo.toml`

**Interfaces:**
- Produces: `infrastructure::picker::RfdIsoPicker` with `new() -> Self` + `Default`, and `async fn pick(&self) -> Option<PathBuf>` (opens `rfd::AsyncFileDialog`, filter `iso`/`img`, returns the chosen path or `None`).

**Note:** não há porta dedicada para o picker (ele é orquestrado pelo `AppCommands` diretamente, junto do `IsoInspector`). É uma camada fina; sem teste unitário (diálogo nativo) — validação manual na Task 11/12.

- [ ] **Step 1: Add dependency**

Root `Cargo.toml` `[workspace.dependencies]`:
```toml
rfd = "0.15"
```
`crates/infrastructure/Cargo.toml` `[dependencies]`:
```toml
rfd = { workspace = true }
```

- [ ] **Step 2: Implement**

`crates/infrastructure/src/picker/rfd_iso_picker.rs`:
```rust
//! Seletor de arquivo ISO usando o diálogo nativo (rfd).

use std::path::PathBuf;

/// Abre o diálogo nativo para escolher uma imagem.
pub struct RfdIsoPicker;

impl RfdIsoPicker {
    /// Cria o seletor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Abre o diálogo e devolve o caminho escolhido, se houver.
    pub async fn pick(&self) -> Option<PathBuf> {
        rfd::AsyncFileDialog::new()
            .add_filter("Imagens", &["iso", "img"])
            .pick_file()
            .await
            .map(|h| h.path().to_path_buf())
    }
}

impl Default for RfdIsoPicker {
    fn default() -> Self {
        Self::new()
    }
}
```

`crates/infrastructure/src/picker/mod.rs`:
```rust
//! Seletor de arquivos (diálogo nativo).

mod rfd_iso_picker;
pub use rfd_iso_picker::RfdIsoPicker;
```

Add to `crates/infrastructure/src/lib.rs`:
```rust
pub mod picker;
```

- [ ] **Step 3: Build + lints + license check**

Run: `cargo build -p infrastructure`
Run: `cargo clippy -p infrastructure --all-targets -- -D warnings`
Run: `cargo deny check 2>&1 | tail -20`
Expected: builds; no warnings; adjust `deny.toml` for any new license from `rfd`/`ashpd` as in Task 8.

- [ ] **Step 4: Commit**

```bash
git add crates/infrastructure/src/picker/ crates/infrastructure/src/lib.rs Cargo.toml crates/infrastructure/Cargo.toml deny.toml
git commit -m "feat(infrastructure): RfdIsoPicker (diálogo nativo de ISO)"
```

---

### Task 10: wiring no `app` (`AppCommands` + `LiveUiState` estendido)

**Files:**
- Modify: `crates/app/src/window.rs`
- Create: `crates/app/src/commands.rs`
- Modify: `crates/app/src/main.rs` (somente se a assinatura de `Window::open` mudar)

**Interfaces:**
- Consumes: `CreateBootable`, `IsoFileInspector`, `Udisks2BlockWriter`, `RfdIsoPicker`, `WriteState`, `IsoSelection`, `CancelFlag`, `ProgressSink`, `WriteProgress`, `UiCommands`, `UiState`, `DeviceListState`, `IsoKind`, `ByteSize`, `DevicePath`.
- Produces:
  - `LiveUiState` gains `write: Arc<RwLock<WriteState>>` and `iso: Arc<RwLock<Option<IsoSelection>>>`; overrides `write_state()` and `selected_iso()`.
  - `AppCommands` impl `UiCommands` (in `crates/app/src/commands.rs`): holds `runtime: tokio::runtime::Handle`, `ctx: egui::Context`, the shared `write`/`iso` locks, a `cancel: CancelFlag`, and the ports (`RfdIsoPicker`, `IsoFileInspector` as `Arc<dyn IsoInspector>`, `CreateBootable`). `pick_iso` spawns a task: `picker.pick().await` → on `Some(path)`, `inspector.classify` → store `IsoSelection { name=file_name, size=metadata.len, kind }` and keep the chosen `PathBuf` in a shared `Arc<RwLock<Option<PathBuf>>>`; `request_repaint`. `start(device)` spawns a task running `CreateBootable::execute(WriteRequest::new(path, device), sink, cancel)`, where `sink` is an `AppProgressSink` that maps `WriteProgress` → `WriteState` into the lock; set terminal `WriteState::Done|Failed|Cancelled` from the result; `request_repaint` on each update. `cancel` calls `self.cancel.cancel()`.

**Implementation notes:**
- Add a private `struct AppProgressSink { write: Arc<RwLock<WriteState>>, ctx: egui::Context }` implementing `ProgressSink`: on `report`, write `WriteState::Writing{done,total}` / `Verifying{done,total}` / `Preparing` from `progress.phase()`, then `ctx.request_repaint()`.
- `pick_iso`/`start` must reset `cancel` to a fresh `CancelFlag` at the start of a run (store it behind a lock so `cancel()` can reach the active one). Simplest: keep `cancel: Arc<RwLock<CancelFlag>>`; on `start`, replace with `CancelFlag::new()` and clone it into the task; `cancel()` reads and calls `.cancel()`.
- The ISO path chosen by `pick_iso` and consumed by `start` lives in a shared `Arc<RwLock<Option<PathBuf>>>`.
- Keep each file ≤199 lines; `window.rs` is already ~117 lines — putting `AppCommands`/`AppProgressSink` in `commands.rs` keeps both within budget.

- [ ] **Step 1: Implement `LiveUiState` extension + `AppCommands`**

Extend `LiveUiState` in `window.rs` to carry and expose the new shared state, and write `crates/app/src/commands.rs` with `AppCommands` + `AppProgressSink` per the notes. In `window.rs`, build both in the eframe creator closure and pass `Arc<dyn UiCommands>` into `NurApp` (see Task 11 for the `NurApp` ctor change), sharing the same `Arc<RwLock<…>>` instances between `LiveUiState` and `AppCommands`. Add `mod commands;` to `crates/app/src/main.rs`.

- [ ] **Step 2: Build**

Run: `cargo build -p app`
Expected: builds once `NurApp::new` accepts the commands argument (Task 11). If doing Task 10 before 11, temporarily pass the commands via a builder and finalize wiring in Task 11; otherwise implement 10 and 11 together before building. **Recommended:** implement Task 11 in the same session and build once at the end of Task 11.

- [ ] **Step 3: Lints**

Run: `cargo clippy -p app --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 4: Commit**

```bash
git add crates/app/src/
git commit -m "feat(app): AppCommands + LiveUiState com write_state/selected_iso"
```

---

### Task 11: religar a UI (estado real + comandos), remover simulação

**Files:**
- Modify: `crates/ui/src/app.rs` (campos, ctor, remove `tick`/`Phase`/`progress`)
- Modify: `crates/ui/src/app/status.rs` (barra/rótulos a partir de `write_state`)
- Modify: `crates/ui/src/app/modal.rs` (confirmar → `commands.start`)
- Modify: `crates/ui/src/app/options.rs` (`iso_section` → `pick_iso` + `selected_iso`)
- Modify: `crates/ui/src/app/draw.rs` (passar device path ao iniciar; botão Cancelar)
- Modify: `crates/ui/src/app/tests.rs` e `crates/ui/src/app/demo.rs` (fakes implementam os novos métodos / `UiCommands`)

**Interfaces:**
- Consumes: `application::ports::{UiState, UiCommands, WriteState, IsoSelection}`, `domain::{IsoKind, DevicePath}`.
- Produces: `NurApp::new(state, commands, screenshots)` — **a assinatura ganha `commands: Arc<dyn UiCommands>`** como 2º parâmetro.

**Key changes:**
- `NurApp` fields: remove `phase: Phase`, `progress: f32`, `iso_selected: bool`; add `commands: Arc<dyn UiCommands>`. Keep `mode`, `selected`, options fields, `modal_open`, `confirm_text`. Delete the `Phase` enum and the `tick` method; `logic()` stops calling `tick` (the bridge drives repaints via `ctx.request_repaint()` from `AppCommands`).
- `status.rs`: derive the bar fraction and labels from `self.state.write_state()`:
  - `WriteState::Idle` → "Pronto"/instrução, barra 0.
  - `Preparing` → "Preparando dispositivo…", barra indeterminada (anime como antes ou deixe 0 com texto).
  - `Writing{done,total}` → `done/total`, "Gravando imagem…".
  - `Verifying{done,total}` → `done/total`, "Verificando…".
  - `Done` → 100%, "Pendrive bootável pronto!", cor success.
  - `Failed(msg)` → texto do erro em `palette.destructive()`.
  - `Cancelled` → "Cancelado — pendrive incompleto, regrave.".
- `ready()`: `self.selected.is_some()` **e** (`mode == Format` **ou** (`selected_iso().is_some()` **e** `selected_iso().kind() == IsoKind::Isohybrid`)).
- `options.rs` `iso_section`: o texto principal vem de `self.state.selected_iso()` (nome · tamanho humanizado) ou o placeholder; o clique chama `self.commands.pick_iso()`. Se `kind() == Unsupported`, mostre o aviso (`palette.destructive()`): "ISO não-isohybrid — gravação raw indisponível; ISOs Windows precisam do modo extração, ainda não disponível".
- `modal.rs`: no `confirm`, em vez de `self.phase = Preparing`, chame `self.commands.start(DevicePath::new(device_path))` (já há `device_path: String` calculado no modal).
- `draw.rs`/`status.rs` footer: quando `write_state` for `Writing`/`Verifying`/`Preparing`, troque o botão "Iniciar" por **"Cancelar"** chamando `self.commands.cancel()`.
- `tests.rs`/`demo.rs`: o `UiStateFake` ganha os métodos novos (pode usar os defaults — não precisa sobrescrever); crie um `CommandsFake` (no-op) implementando `UiCommands` e ajuste `NurApp::new(...)` nas construções de teste/preview e em `window.rs`/`build_app`.

- [ ] **Step 1: Update the `UiStateFake`/add `CommandsFake` and fix the builder test**

In `crates/ui/src/app/tests.rs`, add:
```rust
struct CommandsFake;
impl application::ports::UiCommands for CommandsFake {
    fn pick_iso(&self) {}
    fn start(&self, _device: domain::DevicePath) {}
    fn cancel(&self) {}
}
```
and update `builder_sets_theme` to `NurApp::new(Arc::new(UiStateFake), Arc::new(CommandsFake), Arc::new(NoopWriter))`.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ui`
Expected: FAIL to compile (`NurApp::new` arity changed) — drives the ctor change.

- [ ] **Step 3: Implement the UI changes**

Apply the field/ctor/status/modal/options/draw changes above. Update `demo.rs` if it constructs `NurApp` or sets `phase`/`progress` (replace demo scenarios that set `Phase`/`progress` with ones that feed a `WriteState` via a demo `UiState` — keep the existing `DemoScenario` names by mapping them to `WriteState` instead of the removed `Phase`). Update `crates/app/src/window.rs` `build_app` to pass the `AppCommands` from Task 10.

- [ ] **Step 4: Build, test, lints, line-limit**

Run: `cargo build --bin nur`
Run: `cargo test --workspace`
Run: `cargo clippy --workspace --all-targets -- -D warnings`
Run: `cargo xtask check`
Expected: all green; no file >199 lines.

- [ ] **Step 5: Visual validation (screenshots)**

Capture each scenario headless and inspect the PNG against the prototype:
```bash
for s in ready running format; do
  NUR_CAPTURE=/tmp/nur-$s.png NUR_DEMO=$s NUR_THEME=light \
  LIBGL_ALWAYS_SOFTWARE=1 WINIT_UNIX_BACKEND=x11 \
  timeout 60 xvfb-run -a -s "-screen 0 900x1000x24" ./target/debug/nur
done
```
Read `/tmp/nur-ready.png`, `/tmp/nur-running.png`, `/tmp/nur-format.png` and confirm the progress bar, status text and (un)supported warning render correctly.

- [ ] **Step 6: Commit**

```bash
git add crates/ui/src/ crates/app/src/window.rs
git commit -m "feat(ui): religa estado real de gravação (write_state) e comandos (pick/start/cancel)"
```

---

## Self-Review (preenchido pelo autor do plano)

**Spec coverage:**
- Seleção de ISO (rfd) → Task 9 + 11. Detecção isohybrid → Task 6, bloqueio de Unsupported → Task 5 + 11. udisks2/polkit fd sem unsafe → Task 8. Progresso real → Task 7 (RawCopier) + 10 (sink) + 11 (barra). Cancelamento → Task 3 (CancelFlag) + 7 + 10 + 11. Verificação sempre → Task 7 + 8. WriteState/UiState/UiCommands → Tasks 3/4. Erros (Unauthorized/DeviceBusy/DeviceTooSmall/VerificationMismatch/Cancelled) → Task 2 + mapeados na Task 8. Só modo Boot (Format intacto) → Task 11. Validação QEMU → Task 8 manual. Gates de qualidade → cada task.
- **Sem gaps identificados.**

**Type consistency:** `IsoKind` (domain) usado em 1/3/4/5/6; `WriteState`/`WriteProgress`/`WritePhase`/`WriteRequest`/`IsoSelection`/`CancelFlag`/`ProgressSink` definidos na Task 3 e consumidos consistentemente em 4–11; `NurApp::new(state, commands, screenshots)` fixado na Task 11 e usado no wiring da Task 10/`window.rs`.

**Riscos conhecidos repassados ao executor:**
- Tasks 10 e 11 são acopladas pela mudança de assinatura de `NurApp::new` — **implementar juntas e compilar ao fim da 11** (anotado na Task 10, Step 2).
- A conversão `zvariant fd → std OwnedFd` (Task 8) depende da versão do `zbus` instalada — ajustar localmente; a parte testável está no `RawCopier`.
