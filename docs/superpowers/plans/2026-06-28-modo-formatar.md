# Modo Formatar real (Linux) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ligar o modo *Formatar* a uma formatação real no Linux: tabela de partição (GPT/MBR) + 1 partição + mkfs do filesystem escolhido com rótulo, via udisks2.

**Architecture:** Hexagonal. Novos value objects `PartitionScheme`/`FilesystemKind` (domain), `FormatOptions`/`FormatError`/porta `DeviceFormatter` (application), caso de uso `FormatDevice`, adapter `Udisks2Formatter` (infra: `Block.Format` → `PartitionTable.CreatePartition` → format da partição). A UI dispara por `UiCommands::format` no modal; o `app` spawna a task e reusa o `WriteState` compartilhado (estado "Formatando…").

**Tech Stack:** Rust 2024, egui/eframe 0.35, tokio, async-trait, `zbus 5` (blocking), udisks2.

## Global Constraints

- Edição Rust 2024, `rust-version = 1.88`; crates `domain → application → infrastructure → ui → app`.
- **OOP estrito:** sem função livre exceto `fn main`; helpers são associated functions de struct.
- **Código em inglês**; comentários, logs e textos de UI em **pt-BR** (acentuação correta).
- **Zero campos `pub`** em structs (getters); `cargo xtask pub-fields`.
- **Máx. 199 linhas por arquivo `.rs`**; `cargo xtask line-limit`.
- `unsafe_code = forbid`; sem `unwrap`/`expect`/`panic` fora de `#[cfg(test)]`.
- `missing_docs = deny`, `unreachable_pub = deny` — todo item público documentado.
- Testes em arquivo irmão: `foo.rs` → `foo/tests.rs` com `#[cfg(test)] mod tests;` no fim de `foo.rs`.
- Gate antes de cada commit: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask check`.
- A GUI nunca roda como root; rótulo validado por `domain::VolumeLabel` (1–11 chars). O modo Boot (gravação) deve continuar intacto.

---

### Task 1: domain `PartitionScheme` e `FilesystemKind`

**Files:**
- Create: `crates/domain/src/format_options.rs`
- Create: `crates/domain/src/format_options/tests.rs`
- Modify: `crates/domain/src/lib.rs`

**Interfaces:**
- Produces:
  - `domain::PartitionScheme` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum PartitionScheme { Gpt, Mbr }`.
  - `domain::FilesystemKind` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum FilesystemKind { Fat32, Ntfs, ExFat, Ext4 }`.

- [ ] **Step 1: Write the failing test**

`crates/domain/src/format_options/tests.rs`:
```rust
use super::{FilesystemKind, PartitionScheme};

#[test]
fn scheme_variants_distinct() {
    assert_ne!(PartitionScheme::Gpt, PartitionScheme::Mbr);
}

#[test]
fn filesystem_variants_distinct() {
    assert_ne!(FilesystemKind::Fat32, FilesystemKind::Ntfs);
    assert_ne!(FilesystemKind::ExFat, FilesystemKind::Ext4);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p domain format_options`
Expected: FAIL (module missing).

- [ ] **Step 3: Implement**

`crates/domain/src/format_options.rs`:
```rust
//! Value objects para a formatação: esquema de partição e filesystem.

/// Esquema da tabela de partição.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionScheme {
    /// GUID Partition Table.
    Gpt,
    /// Master Boot Record (DOS).
    Mbr,
}

/// Tipo de filesystem a criar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemKind {
    /// FAT32.
    Fat32,
    /// NTFS.
    Ntfs,
    /// exFAT.
    ExFat,
    /// ext4.
    Ext4,
}

#[cfg(test)]
mod tests;
```

Add to `crates/domain/src/lib.rs` (module + re-export, following the existing pattern):
```rust
mod format_options;
pub use format_options::{FilesystemKind, PartitionScheme};
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p domain format_options`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/domain/src/format_options.rs crates/domain/src/format_options/tests.rs crates/domain/src/lib.rs
git commit -m "feat(domain): PartitionScheme e FilesystemKind"
```

---

### Task 2: `FormatError`, `FormatOptions`, porta `DeviceFormatter`, `UiCommands::format`

**Files:**
- Modify: `crates/application/src/errors.rs`
- Modify: `crates/application/src/errors/tests.rs`
- Create: `crates/application/src/ports/format.rs`
- Create: `crates/application/src/ports/device_formatter.rs`
- Modify: `crates/application/src/ports/ui_commands.rs`
- Modify: `crates/application/src/ports/mod.rs`

**Interfaces:**
- Consumes: `domain::{DevicePath, PartitionScheme, FilesystemKind, VolumeLabel}`.
- Produces:
  - `FormatError { Unauthorized, DeviceBusy, ToolMissing(String), Backend(String) }`.
  - `FormatOptions { scheme: PartitionScheme, filesystem: FilesystemKind, label: VolumeLabel, quick: bool }` (private fields + getters `scheme()`, `filesystem()`, `label()`, `quick()`, ctor `new`).
  - `DeviceFormatter` (`#[async_trait]`): `async fn format(&self, device: &DevicePath, options: &FormatOptions) -> Result<(), FormatError>`.
  - `UiCommands` gains `fn format(&self, device: DevicePath, options: FormatOptions)`.

- [ ] **Step 1: Write the failing test**

Append to `crates/application/src/errors/tests.rs`:
```rust
#[test]
fn format_error_messages_are_in_ptbr() {
    use super::FormatError;
    assert_eq!(FormatError::Unauthorized.to_string(), "autorização negada");
    assert_eq!(FormatError::DeviceBusy.to_string(), "dispositivo ocupado");
    assert_eq!(
        FormatError::ToolMissing("exFAT".to_owned()).to_string(),
        "instale as ferramentas para formatar em exFAT"
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p application errors`
Expected: FAIL (`FormatError` not found).

- [ ] **Step 3: Implement**

Append to `crates/application/src/errors.rs` (before the trailing `#[cfg(test)] mod tests;`):
```rust
/// Falhas ao formatar o dispositivo.
#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    /// O polkit negou a autorização.
    #[error("autorização negada")]
    Unauthorized,
    /// O dispositivo está em uso.
    #[error("dispositivo ocupado")]
    DeviceBusy,
    /// Falta a ferramenta de mkfs para o filesystem escolhido.
    #[error("instale as ferramentas para formatar em {0}")]
    ToolMissing(String),
    /// Falha do backend (udisks/D-Bus).
    #[error("falha ao formatar: {0}")]
    Backend(String),
}
```

`crates/application/src/ports/format.rs`:
```rust
//! Opções de formatação escolhidas pelo usuário.

use domain::{FilesystemKind, PartitionScheme, VolumeLabel};

/// Como formatar o dispositivo.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    scheme: PartitionScheme,
    filesystem: FilesystemKind,
    label: VolumeLabel,
    quick: bool,
}

impl FormatOptions {
    /// Cria as opções.
    #[must_use]
    pub fn new(
        scheme: PartitionScheme,
        filesystem: FilesystemKind,
        label: VolumeLabel,
        quick: bool,
    ) -> Self {
        Self {
            scheme,
            filesystem,
            label,
            quick,
        }
    }

    /// Esquema de partição.
    #[must_use]
    pub fn scheme(&self) -> PartitionScheme {
        self.scheme
    }

    /// Filesystem a criar.
    #[must_use]
    pub fn filesystem(&self) -> FilesystemKind {
        self.filesystem
    }

    /// Rótulo do volume.
    #[must_use]
    pub fn label(&self) -> &VolumeLabel {
        &self.label
    }

    /// Formatação rápida (sem zerar o disco).
    #[must_use]
    pub fn quick(&self) -> bool {
        self.quick
    }
}
```

`crates/application/src/ports/device_formatter.rs`:
```rust
//! Porta de formatação de um dispositivo.

use crate::errors::FormatError;
use crate::ports::FormatOptions;
use domain::DevicePath;

/// Formata o dispositivo (tabela de partição + 1 partição + filesystem).
#[async_trait::async_trait]
pub trait DeviceFormatter: Send + Sync {
    /// Formata o `device` conforme as `options`.
    ///
    /// # Errors
    /// Retorna [`FormatError`] em falha de autorização, ferramenta ausente ou backend.
    async fn format(
        &self,
        device: &DevicePath,
        options: &FormatOptions,
    ) -> Result<(), FormatError>;
}
```

In `crates/application/src/ports/ui_commands.rs`, add to the trait:
```rust
    /// Formata o pendrive conforme as opções.
    fn format(&self, device: DevicePath, options: crate::ports::FormatOptions);
```

In `crates/application/src/ports/mod.rs`, register/export:
```rust
mod device_formatter;
mod format;
pub use device_formatter::DeviceFormatter;
pub use format::FormatOptions;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p application errors` then `cargo clippy -p application --all-targets -- -D warnings`
Expected: PASS / no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/application/src/errors.rs crates/application/src/errors/tests.rs crates/application/src/ports/
git commit -m "feat(application): FormatError, FormatOptions, DeviceFormatter, UiCommands::format"
```

---

### Task 3: caso de uso `FormatDevice`

**Files:**
- Create: `crates/application/src/use_cases/format_device.rs`
- Create: `crates/application/src/use_cases/format_device/tests.rs`
- Modify: `crates/application/src/use_cases/mod.rs`

**Interfaces:**
- Consumes: `DeviceFormatter`, `FormatOptions`, `ProgressSink`, `WriteProgress`, `WritePhase`, `FormatError`, `DevicePath`.
- Produces: `FormatDevice` with `new(formatter: Arc<dyn DeviceFormatter>) -> Self` and `async fn execute(&self, device: DevicePath, options: FormatOptions, sink: Arc<dyn ProgressSink>) -> Result<(), FormatError>` — reports `WritePhase::Preparing` then delegates to the formatter.

- [ ] **Step 1: Write the failing test**

`crates/application/src/use_cases/format_device/tests.rs`:
```rust
use super::FormatDevice;
use crate::errors::FormatError;
use crate::ports::{DeviceFormatter, FormatOptions, ProgressSink, WriteProgress};
use domain::{DevicePath, FilesystemKind, PartitionScheme, VolumeLabel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

struct SpyFormatter(Arc<AtomicBool>);
#[async_trait::async_trait]
impl DeviceFormatter for SpyFormatter {
    async fn format(
        &self,
        _d: &DevicePath,
        _o: &FormatOptions,
    ) -> Result<(), FormatError> {
        self.0.store(true, Ordering::SeqCst);
        Ok(())
    }
}

struct NoopSink;
impl ProgressSink for NoopSink {
    fn report(&self, _p: WriteProgress) {}
}

fn options() -> FormatOptions {
    FormatOptions::new(
        PartitionScheme::Gpt,
        FilesystemKind::Fat32,
        VolumeLabel::parse("BOOTUSB").unwrap(),
        true,
    )
}

#[tokio::test]
async fn execute_delegates_to_formatter() {
    let called = Arc::new(AtomicBool::new(false));
    let uc = FormatDevice::new(Arc::new(SpyFormatter(Arc::clone(&called))));
    let result = uc
        .execute(
            DevicePath::new("/dev/sdb".to_owned()),
            options(),
            Arc::new(NoopSink),
        )
        .await;
    assert!(result.is_ok());
    assert!(called.load(Ordering::SeqCst));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p application format_device`
Expected: FAIL (type missing).

- [ ] **Step 3: Implement**

`crates/application/src/use_cases/format_device.rs`:
```rust
//! Caso de uso: formatar um dispositivo.

use crate::errors::FormatError;
use crate::ports::{DeviceFormatter, FormatOptions, ProgressSink, WritePhase, WriteProgress};
use domain::DevicePath;
use std::sync::Arc;

/// Orquestra a formatação do dispositivo.
pub struct FormatDevice {
    formatter: Arc<dyn DeviceFormatter>,
}

impl FormatDevice {
    /// Cria o caso de uso com a porta injetada.
    #[must_use]
    pub fn new(formatter: Arc<dyn DeviceFormatter>) -> Self {
        Self { formatter }
    }

    /// Reporta o início e formata o dispositivo.
    ///
    /// # Errors
    /// Retorna [`FormatError`] se a formatação falhar.
    pub async fn execute(
        &self,
        device: DevicePath,
        options: FormatOptions,
        sink: Arc<dyn ProgressSink>,
    ) -> Result<(), FormatError> {
        sink.report(WriteProgress::new(WritePhase::Preparing, 0, 0));
        self.formatter.format(&device, &options).await
    }
}

#[cfg(test)]
mod tests;
```

Add to `crates/application/src/use_cases/mod.rs`:
```rust
mod format_device;
pub use format_device::FormatDevice;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p application format_device`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/application/src/use_cases/
git commit -m "feat(application): caso de uso FormatDevice"
```

---

### Task 4: `Udisks2Formatter` (infra)

**Files:**
- Create: `crates/infrastructure/src/linux/udisks2_formatter.rs`
- Create: `crates/infrastructure/src/linux/udisks2_formatter/tests.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Consumes: `DeviceFormatter`, `FormatOptions`, `FormatError`, `DevicePath`, `PartitionScheme`, `FilesystemKind`.
- Produces: `infrastructure::linux::Udisks2Formatter` with `new() -> Self` + `Default`; impl `DeviceFormatter`. Pure mapping `fn udisks_table(scheme: PartitionScheme) -> &'static str` and `fn udisks_fs(fs: FilesystemKind) -> &'static str` are testable.

**Mapeamento (testável):** `Gpt→"gpt"`, `Mbr→"dos"`; `Fat32→"vfat"`, `Ntfs→"ntfs"`, `ExFat→"exfat"`, `Ext4→"ext4"`.

**Notas do fluxo zbus (casca fina — validação manual/loopback):**
- `name = device.as_str()` (caminho completo `/dev/sdX`); object path do device = `/org/freedesktop/UDisks2/block_devices/<trim /dev/>`.
- zbus **blocking** dentro de `spawn_blocking`:
  1. `Block.Format(<table>, {})` no device — cria a tabela vazia (`org.freedesktop.UDisks2.Block`, método `Format`, args `(&str, HashMap<&str, Value>)`).
  2. `PartitionTable.CreatePartition(0u64, 0u64, "", "", {})` no mesmo objeto (`org.freedesktop.UDisks2.PartitionTable`) — offset 0, size 0 (= máximo); retorna o **object path `o`** da nova partição: `reply.body().deserialize::<zbus::zvariant::OwnedObjectPath>()`.
  3. No objeto da partição, `Block.Format(<fs>, options)` onde `options` contém `"label": Value::from(label.as_str())` e, se `!quick`, `"erase": Value::from("zero")`.
- Mapear erros do D-Bus: mensagem com `NotAuthorized` → `Unauthorized`; `Busy`/`mounted`/`in use` → `DeviceBusy`; mensagem com `not found`/`Failed to execute`/`No such file` (mkfs ausente) → `ToolMissing(<fs humano>)`; demais → `Backend(msg)`.
- A `async fn format` extrai os dados de `options`/`device` e roda tudo em `spawn_blocking`; mapeia `JoinError` → `Backend`.
- Mantenha o arquivo ≤199 linhas (helpers privados `format_table`, `create_partition`, `format_partition`, `classify_err`).

- [ ] **Step 1: Write the failing test (mapeamento puro)**

`crates/infrastructure/src/linux/udisks2_formatter/tests.rs`:
```rust
use super::Udisks2Formatter;
use domain::{FilesystemKind, PartitionScheme};

#[test]
fn maps_partition_table() {
    assert_eq!(Udisks2Formatter::udisks_table(PartitionScheme::Gpt), "gpt");
    assert_eq!(Udisks2Formatter::udisks_table(PartitionScheme::Mbr), "dos");
}

#[test]
fn maps_filesystem() {
    assert_eq!(Udisks2Formatter::udisks_fs(FilesystemKind::Fat32), "vfat");
    assert_eq!(Udisks2Formatter::udisks_fs(FilesystemKind::Ntfs), "ntfs");
    assert_eq!(Udisks2Formatter::udisks_fs(FilesystemKind::ExFat), "exfat");
    assert_eq!(Udisks2Formatter::udisks_fs(FilesystemKind::Ext4), "ext4");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p infrastructure udisks2_formatter`
Expected: FAIL (type missing).

- [ ] **Step 3: Implement**

Write `crates/infrastructure/src/linux/udisks2_formatter.rs` per the zbus notes, with the two pure mapping functions and the `DeviceFormatter` impl. Register in `crates/infrastructure/src/linux/mod.rs`:
```rust
mod udisks2_formatter;
pub use udisks2_formatter::Udisks2Formatter;
```
Reference `crates/infrastructure/src/linux/udisks2_block_writer.rs` for the established zbus blocking + `call_method` pattern.

- [ ] **Step 4: Build, lints, line-limit, mapping tests**

Run: `cargo test -p infrastructure udisks2_formatter`
Run: `cargo clippy -p infrastructure --all-targets -- -D warnings`
Run: `cargo xtask check`
Expected: mapping tests PASS; no warnings; file ≤199 lines.

- [ ] **Step 5: Manual validation (loopback, fora do CI)**

```bash
truncate -s 256M /tmp/fake.img
sudo losetup -fP /tmp/fake.img   # ex.: /dev/loop0
```
Run `nur`, selecione o loop device, modo Formatar, GPT + FAT32 + rótulo, Iniciar. Depois:
```bash
lsblk -f /dev/loop0   # confirma a partição e o FS
sudo losetup -d /dev/loop0
```
Testar também um FS sem `mkfs.*` instalado → `ToolMissing`. **Sem teste automatizado** aqui; a lógica testável é o mapeamento (Step 1).

- [ ] **Step 6: Commit**

```bash
git add crates/infrastructure/src/linux/udisks2_formatter.rs crates/infrastructure/src/linux/udisks2_formatter/tests.rs crates/infrastructure/src/linux/mod.rs
git commit -m "feat(infrastructure): Udisks2Formatter (tabela + partição + mkfs)"
```

---

### Task 5: wiring (`app`) + UI (modal, opções, status)

**Files:**
- Modify: `crates/app/src/commands.rs`
- Modify: `crates/ui/src/app/modal.rs`
- Modify: `crates/ui/src/app/options.rs`
- Modify: `crates/ui/src/app/status.rs`
- Modify: `crates/ui/src/app/tests.rs`

**Interfaces:**
- Consumes: `FormatDevice`, `Udisks2Formatter`, `UiCommands::format`, `FormatOptions`, `PartitionScheme`, `FilesystemKind`, `VolumeLabel`, `WriteState`, `Mode`.
- Produces: `AppCommands::format` (spawns the format task into the shared `write` lock); UI builds `FormatOptions` and routes the modal confirm by mode; the "Sistema alvo" select shows only in Boot; status texts become mode-aware; `ready()` requires a valid label in Format.

- [ ] **Step 1: `AppCommands::format`**

In `crates/app/src/commands.rs`:
- Add imports: `application::ports::FormatOptions`, `application::use_cases::FormatDevice`, `infrastructure::linux::Udisks2Formatter`.
- Implement the new trait method:
```rust
    fn format(&self, device: DevicePath, options: FormatOptions) {
        let ctx = self.ctx.clone();
        let write = Arc::clone(&self.write);
        self.runtime.spawn(async move {
            let uc = FormatDevice::new(Arc::new(Udisks2Formatter::new()));
            let sink: Arc<dyn ProgressSink> =
                Arc::new(AppProgressSink::new(Arc::clone(&write), ctx.clone()));
            let next = match uc.execute(device, options, sink).await {
                Ok(()) => WriteState::Done,
                Err(e) => WriteState::Failed(e.to_string()),
            };
            if let Ok(mut guard) = write.write() {
                *guard = next;
            }
            ctx.request_repaint();
        });
    }
```

- [ ] **Step 2: route the modal confirm by mode**

In `crates/ui/src/app/modal.rs`, replace the `confirm` block:
```rust
        if confirm {
            self.modal_open = false;
            if !device_path.is_empty() {
                self.dispatch(DevicePath::new(device_path));
            }
        }
```
And add a private helper on `NurApp` (in `modal.rs`):
```rust
    // Dispara a operação do modo atual (gravar ou formatar).
    fn dispatch(&self, device: DevicePath) {
        match self.mode {
            super::Mode::Boot => self.commands.start(device),
            super::Mode::Format => {
                if let Ok(label) = domain::VolumeLabel::parse(&self.label) {
                    let options = application::ports::FormatOptions::new(
                        self.partition_scheme(),
                        self.filesystem_kind(),
                        label,
                        self.quick_format,
                    );
                    self.commands.format(device, options);
                }
            }
        }
    }

    // Índices da UI → value objects de domínio.
    fn partition_scheme(&self) -> domain::PartitionScheme {
        if self.partition == 0 {
            domain::PartitionScheme::Gpt
        } else {
            domain::PartitionScheme::Mbr
        }
    }

    fn filesystem_kind(&self) -> domain::FilesystemKind {
        match self.filesystem {
            0 => domain::FilesystemKind::Fat32,
            1 => domain::FilesystemKind::Ntfs,
            2 => domain::FilesystemKind::ExFat,
            _ => domain::FilesystemKind::Ext4,
        }
    }
```
(`Mode` is `super::Mode`; `partition`/`filesystem`/`quick_format`/`label` are existing `NurApp` fields.)

- [ ] **Step 3: hide "Sistema alvo" in Format**

In `crates/ui/src/app/options.rs`, replace `options_section` body so the target select is Boot-only:
```rust
    pub(super) fn options_section(&mut self, ui: &mut egui::Ui, palette: Palette) {
        FieldLabel::show(ui, palette, "OPÇÕES DE FORMATO");
        if self.mode == Mode::Boot {
            ui.columns(2, |cols| {
                LabeledSelect::show(
                    &mut cols[0], palette, "partition", "Esquema de partição",
                    &PARTITIONS, &mut self.partition,
                );
                LabeledSelect::show(
                    &mut cols[1], palette, "target", "Sistema alvo",
                    &TARGETS, &mut self.target,
                );
            });
            ui.add_space(12.0);
            ui.columns(2, |cols| {
                LabeledSelect::show(
                    &mut cols[0], palette, "fs", "Sistema de arquivos",
                    &FILESYSTEMS, &mut self.filesystem,
                );
                LabeledInput::show(&mut cols[1], palette, "Rótulo do volume", &mut self.label);
            });
        } else {
            ui.columns(2, |cols| {
                LabeledSelect::show(
                    &mut cols[0], palette, "partition", "Esquema de partição",
                    &PARTITIONS, &mut self.partition,
                );
                LabeledSelect::show(
                    &mut cols[1], palette, "fs", "Sistema de arquivos",
                    &FILESYSTEMS, &mut self.filesystem,
                );
            });
            ui.add_space(12.0);
            LabeledInput::show(ui, palette, "Rótulo do volume", &mut self.label);
        }
        ui.add_space(12.0);
        Checkbox::show(ui, palette, "Formatação rápida", &mut self.quick_format);
    }
```

- [ ] **Step 4: mode-aware status texts + label-aware readiness**

In `crates/ui/src/app/status.rs`:
- In `status_text`, make Writing/Done depend on `self.mode`:
```rust
            WriteState::Writing { .. } if self.mode == Mode::Boot => {
                ("Gravando imagem\u{2026}".to_owned(), muted)
            }
            WriteState::Writing { .. } => ("Formatando\u{2026}".to_owned(), muted),
            WriteState::Preparing if self.mode == Mode::Format => {
                ("Formatando\u{2026}".to_owned(), muted)
            }
            WriteState::Preparing => ("Preparando dispositivo\u{2026}".to_owned(), muted),
            WriteState::Done if self.mode == Mode::Boot => {
                ("Pendrive bootável pronto!".to_owned(), palette.success())
            }
            WriteState::Done => ("Formatação concluída!".to_owned(), palette.success()),
```
(Keep the existing `Idle`/`Verifying`/`Failed`/`Cancelled` arms; place the new `Preparing`/`Writing`/`Done` arms accordingly. Ensure match arm order compiles — specific `if` guards before the catch-all of the same variant.)
- In `ready()`, require a valid label in Format:
```rust
    pub(super) fn ready(&self) -> bool {
        let mode_ok = match self.mode {
            Mode::Format => domain::VolumeLabel::parse(&self.label).is_ok(),
            Mode::Boot => self
                .state
                .selected_iso()
                .is_some_and(|s| s.kind() == IsoKind::Isohybrid),
        };
        self.selected.is_some() && mode_ok && !self.in_progress()
    }
```

- [ ] **Step 5: update the UI test fake**

In `crates/ui/src/app/tests.rs`, add to `CommandsFake`:
```rust
    fn format(&self, _device: DevicePath, _options: application::ports::FormatOptions) {}
```

- [ ] **Step 6: Build, test, lints, screenshots**

Run: `cargo build --bin nur`
Run: `cargo test --workspace`
Run: `cargo clippy --workspace --all-targets -- -D warnings`
Run: `cargo xtask check`
Capture the Format scenario and confirm "Sistema alvo" is gone:
```bash
NUR_CAPTURE=/tmp/nur-format2.png NUR_DEMO=format NUR_THEME=light \
LIBGL_ALWAYS_SOFTWARE=1 WINIT_UNIX_BACKEND=x11 \
timeout 60 xvfb-run -a -s "-screen 0 900x1000x24" ./target/debug/nur
```
Read `/tmp/nur-format2.png` and confirm the options layout (partition + fs, label, quick) without "Sistema alvo".

- [ ] **Step 7: Commit**

```bash
git add crates/app/src/ crates/ui/src/
git commit -m "feat: modo formatar real (wiring + UI mode-aware)"
```

---

## Self-Review (preenchido pelo autor do plano)

**Spec coverage:**
- Tabela + 1 partição + mkfs + label → Task 4 (fluxo udisks). Esquema GPT/MBR + FS + label da UI → Tasks 1/2/5. Ocultar "Sistema alvo" no Format → Task 5 (Step 3). Quick vs erase zero → Task 4 (options). Reusa `WriteState` (Preparing→"Formatando…"→Done/Failed), sem cancelamento → Tasks 3/5. Mesmo modal "APAGAR" → Task 5 (Step 2). Erros (Unauthorized/DeviceBusy/ToolMissing/Backend) → Task 2 + mapeados na Task 4. Rótulo via `VolumeLabel` → Tasks 2/5. Modo Boot intacto → Task 5 (branch por modo). Validação loopback → Task 4 manual. Qualidade → cada task.
- **Sem gaps.**

**Type consistency:** `PartitionScheme`/`FilesystemKind` (Task 1) usados em 2/4/5; `FormatOptions::new(scheme, filesystem, label, quick)` (Task 2) construído na Task 5 e consumido em 3/4; `DeviceFormatter::format(&DevicePath, &FormatOptions)` (Task 2) impl na Task 4, chamado na Task 3; `UiCommands::format(DevicePath, FormatOptions)` (Task 2) impl na Task 5 e no fake; `FormatDevice::execute(device, options, sink)` (Task 3) chamado na Task 5; `udisks_table`/`udisks_fs` (Task 4) testados na própria task.

**Notas ao executor:**
- A casca `Udisks2Formatter` (Task 4) não tem teste unitário do fluxo D-Bus — a lógica testável é o mapeamento. Siga o padrão zbus do `Udisks2BlockWriter` (Fase 4).
- `VolumeLabel` limita o rótulo a 11 chars (FAT) para todos os FS — limitação conhecida e aceitável neste incremento; `ready()` desabilita "Iniciar" se o rótulo for inválido.
- Atenção à ordem dos braços do `match` em `status_text` (guardas `if` específicas antes do braço genérico da mesma variante).
