# Formatador FAT32 nativo (Rust puro) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reescrever o modo Formatar em Rust nativo (GPT/MBR + FAT32), sem `mkfs`/`Block.Format`, oferecendo só FAT32 na UI.

**Architecture:** Privilégio via udisks2 `OpenDevice` (fd, como na Fase 4). Sobre o fd: `gpt`/`mbrman` escrevem a tabela (1 partição alinhada a 1 MiB); `fatfs` cria o FAT32 numa janela `OffsetVolume` que recorta a partição. A geometria, a janela e a formatação FAT32/GPT são testáveis em Rust puro (buffers `Cursor`); só a obtenção do fd é casca fina (validação por loopback).

**Tech Stack:** Rust 2024, `gpt`, `fatfs`, `mbrman`, `zbus` (blocking), tokio, async-trait.

## Global Constraints

- Edição Rust 2024, `rust-version = 1.88`; crates `domain → application → infrastructure → ui → app`.
- **OOP estrito:** sem função livre exceto `fn main`; helpers são associated functions de struct.
- Código em inglês; comentários/logs/UI em **pt-BR** com acentuação.
- **Zero campos `pub`** (getters); **máx. 199 linhas/arquivo**; `unsafe_code = forbid`; sem `unwrap`/`expect`/`panic` fora de `#[cfg(test)]`; `missing_docs`/`unreachable_pub` = erro.
- Testes em arquivo irmão `foo/tests.rs` com `#[cfg(test)] mod tests;` no fim de `foo.rs`.
- Gate antes de cada commit: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask check`.
- A GUI nunca roda como root (só o fd vem do udisks/polkit). Sem ferramentas externas (`mkfs`/`Block.Format`).

---

### Task 1: dependências + `OffsetVolume` (janela de IO)

**Files:**
- Modify: `Cargo.toml` (workspace deps), `crates/infrastructure/Cargo.toml`
- Create: `crates/infrastructure/src/linux/offset_volume.rs`
- Create: `crates/infrastructure/src/linux/offset_volume/tests.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Produces: `infrastructure::linux::OffsetVolume<T>` — wrapper `Read + Write + Seek` que confina um `T: Read + Write + Seek` à janela `[start, start+len)`. `pub(crate) fn new(inner: T, start: u64, len: u64) -> Self`. `Seek` é relativo ao início da janela; leituras/escritas não passam de `len`.

- [ ] **Step 1: Add dependencies**

Root `Cargo.toml` `[workspace.dependencies]`:
```toml
gpt = "4"
fatfs = "0.3"
mbrman = "0.5"
```
`crates/infrastructure/Cargo.toml` `[dependencies]`:
```toml
gpt = { workspace = true }
fatfs = { workspace = true }
mbrman = { workspace = true }
```

- [ ] **Step 2: Write the failing test**

`crates/infrastructure/src/linux/offset_volume/tests.rs`:
```rust
use super::OffsetVolume;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

fn base() -> Cursor<Vec<u8>> {
    Cursor::new(vec![0u8; 100])
}

#[test]
fn writes_and_reads_within_window() {
    let mut v = OffsetVolume::new(base(), 10, 20);
    v.seek(SeekFrom::Start(0)).unwrap();
    assert_eq!(v.write(&[1, 2, 3]).unwrap(), 3);
    v.seek(SeekFrom::Start(0)).unwrap();
    let mut buf = [0u8; 3];
    v.read_exact(&mut buf).unwrap();
    assert_eq!(buf, [1, 2, 3]);
}

#[test]
fn write_is_clamped_to_window_len() {
    let mut v = OffsetVolume::new(base(), 10, 4);
    v.seek(SeekFrom::Start(2)).unwrap();
    // só cabem 2 bytes (posições 2 e 3) antes do fim da janela.
    let n = v.write(&[9, 9, 9, 9]).unwrap();
    assert_eq!(n, 2);
}

#[test]
fn maps_offset_into_base() {
    let mut v = OffsetVolume::new(base(), 10, 20);
    v.seek(SeekFrom::Start(5)).unwrap();
    v.write_all(&[7]).unwrap();
    let inner = v.into_inner();
    assert_eq!(inner.into_inner()[15], 7); // 10 (start) + 5
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test -p infrastructure offset_volume`
Expected: FAIL (tipo ausente).

- [ ] **Step 4: Implement**

`crates/infrastructure/src/linux/offset_volume.rs`:
```rust
//! Janela de IO: confina um Read+Write+Seek a `[start, start+len)`.
//! Usada para entregar ao `fatfs` apenas a região da partição do device.

use std::io::{Read, Result, Seek, SeekFrom, Write};

/// Recorte `[start, start+len)` sobre um dispositivo/arquivo.
pub(crate) struct OffsetVolume<T> {
    inner: T,
    start: u64,
    len: u64,
    pos: u64,
}

impl<T: Read + Write + Seek> OffsetVolume<T> {
    /// Cria a janela.
    pub(crate) fn new(inner: T, start: u64, len: u64) -> Self {
        Self {
            inner,
            start,
            len,
            pos: 0,
        }
    }

    /// Devolve o IO interno.
    pub(crate) fn into_inner(self) -> T {
        self.inner
    }

    // Quantos bytes ainda cabem da posição atual até o fim da janela.
    fn remaining(&self) -> u64 {
        self.len.saturating_sub(self.pos)
    }
}

impl<T: Read + Write + Seek> Read for OffsetVolume<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let cap = self.remaining().min(buf.len() as u64) as usize;
        if cap == 0 {
            return Ok(0);
        }
        self.inner.seek(SeekFrom::Start(self.start + self.pos))?;
        let n = self.inner.read(&mut buf[..cap])?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl<T: Read + Write + Seek> Write for OffsetVolume<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let cap = self.remaining().min(buf.len() as u64) as usize;
        if cap == 0 {
            return Ok(0);
        }
        self.inner.seek(SeekFrom::Start(self.start + self.pos))?;
        let n = self.inner.write(&buf[..cap])?;
        self.pos += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

impl<T: Read + Write + Seek> Seek for OffsetVolume<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let target = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::End(n) => self.len as i64 + n,
            SeekFrom::Current(n) => self.pos as i64 + n,
        };
        let clamped = target.clamp(0, self.len as i64);
        self.pos = clamped as u64;
        Ok(self.pos)
    }
}

#[cfg(test)]
mod tests;
```

Add to `crates/infrastructure/src/linux/mod.rs`:
```rust
mod offset_volume;
pub(crate) use offset_volume::OffsetVolume;
```

- [ ] **Step 5: Run to verify it passes**

Run: `cargo build -p infrastructure` then `cargo test -p infrastructure offset_volume`
Expected: builds (deps baixadas) e 3 testes PASS.

- [ ] **Step 6: License check + commit**

Run: `cargo deny check 2>&1 | tail -20` — se `gpt`/`fatfs`/`mbrman` trouxerem licença nova, adicione o SPDX exato ao `allow` do `deny.toml` (uma linha de justificativa).
```bash
git add Cargo.toml Cargo.lock crates/infrastructure/Cargo.toml crates/infrastructure/src/linux/offset_volume.rs crates/infrastructure/src/linux/offset_volume/tests.rs crates/infrastructure/src/linux/mod.rs deny.toml
git commit -m "feat(infrastructure): deps gpt/fatfs/mbrman + OffsetVolume (janela de IO)"
```

---

### Task 2: `Partitioner` — geometria + tabela GPT/MBR

**Files:**
- Create: `crates/infrastructure/src/linux/partitioner.rs`
- Create: `crates/infrastructure/src/linux/partitioner/tests.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Consumes: `domain::PartitionScheme`.
- Produces: `infrastructure::linux::Partitioner` com:
  - `pub(crate) fn single_partition(device_bytes: u64) -> (u64, u64)` — `(start, len)` em bytes: `start = 1 MiB` (1_048_576), `len = (device_bytes - start)` arredondado para baixo a múltiplo de 512; retorna `(start, 0)` se o device não comporta.
  - `pub(crate) fn write_table<T: Read + Write + Seek>(dev: &mut T, scheme: PartitionScheme, start: u64, len: u64) -> io::Result<()>` — escreve GPT (via `gpt`) ou MBR (via `mbrman`) com 1 partição FAT32 em `[start, start+len)`.

**Notas:** a API exata do `gpt`/`mbrman` varia por versão; ajuste à versão resolvida. GPT: MBR protetor + header + 1 entrada `BASIC`/dados cobrindo a partição. MBR: 1 entrada tipo `0x0C` (FAT32 LBA), `start/len` em setores de 512, marcada como bootável não é necessário.

- [ ] **Step 1: Write the failing test (geometria — pura)**

`crates/infrastructure/src/linux/partitioner/tests.rs`:
```rust
use super::Partitioner;

#[test]
fn single_partition_aligns_to_1mib_and_fills() {
    let (start, len) = Partitioner::single_partition(64 * 1024 * 1024);
    assert_eq!(start, 1024 * 1024);
    assert_eq!(len, 63 * 1024 * 1024);
    assert_eq!(len % 512, 0);
}

#[test]
fn single_partition_zero_len_when_too_small() {
    let (_start, len) = Partitioner::single_partition(512 * 1024); // < 1 MiB
    assert_eq!(len, 0);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p infrastructure partitioner`
Expected: FAIL (tipo ausente).

- [ ] **Step 3: Implement**

`crates/infrastructure/src/linux/partitioner.rs` — `single_partition` (pura, exatamente como os testes exigem) e `write_table` (GPT via `gpt`, MBR via `mbrman`). Estrutura sugerida:
```rust
//! Escreve a tabela de partição (GPT/MBR) com 1 partição FAT32 cobrindo o disco.

use domain::PartitionScheme;
use std::io::{Read, Seek, Write};

const ALIGN: u64 = 1024 * 1024; // 1 MiB
const SECTOR: u64 = 512;

/// Particionador nativo (sem ferramentas externas).
pub(crate) struct Partitioner;

impl Partitioner {
    /// `(start, len)` em bytes para 1 partição cobrindo o disco (start a 1 MiB).
    pub(crate) fn single_partition(device_bytes: u64) -> (u64, u64) {
        if device_bytes <= ALIGN {
            return (ALIGN, 0);
        }
        let len = ((device_bytes - ALIGN) / SECTOR) * SECTOR;
        (ALIGN, len)
    }

    /// Escreve a tabela escolhida com 1 partição FAT32 em `[start, start+len)`.
    pub(crate) fn write_table<T: Read + Write + Seek>(
        dev: &mut T,
        scheme: PartitionScheme,
        start: u64,
        len: u64,
    ) -> std::io::Result<()> {
        match scheme {
            PartitionScheme::Gpt => Self::write_gpt(dev, start, len),
            PartitionScheme::Mbr => Self::write_mbr(dev, start, len),
        }
    }

    // write_gpt / write_mbr: usar os crates `gpt` / `mbrman` conforme a versão
    // resolvida. Round-trip validado nos testes (ler de volta com o mesmo crate).
}

#[cfg(test)]
mod tests;
```
Implemente `write_gpt`/`write_mbr` com os crates. Adicione testes de round-trip:
```rust
// (acrescente em tests.rs)
use super::Partitioner;
use domain::PartitionScheme;
use std::io::Cursor;

#[test]
fn gpt_roundtrip_has_one_partition() {
    let mut dev = Cursor::new(vec![0u8; 64 * 1024 * 1024]);
    let (start, len) = Partitioner::single_partition(64 * 1024 * 1024);
    Partitioner::write_table(&mut dev, PartitionScheme::Gpt, start, len).unwrap();
    // Releia com o crate `gpt` e confirme exatamente 1 partição cobrindo a região.
    // (ajuste a leitura à API da versão do crate `gpt`.)
}
```
(Faça o equivalente para MBR com `mbrman`.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p infrastructure partitioner`
Run: `cargo clippy -p infrastructure --all-targets -- -D warnings`
Expected: testes PASS, sem warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/infrastructure/src/linux/partitioner.rs crates/infrastructure/src/linux/partitioner/tests.rs crates/infrastructure/src/linux/mod.rs
git commit -m "feat(infrastructure): Partitioner (geometria + tabela GPT/MBR)"
```

---

### Task 3: criação de FAT32 (`fatfs`)

**Files:**
- Create: `crates/infrastructure/src/linux/fat32.rs`
- Create: `crates/infrastructure/src/linux/fat32/tests.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Consumes: `domain::VolumeLabel`.
- Produces: `infrastructure::linux::Fat32` com `pub(crate) fn format<T: Read + Write + Seek>(volume: &mut T, label: &VolumeLabel) -> io::Result<()>` — formata `volume` inteiro como FAT32 com o rótulo (≤11 bytes, padding com espaço).

- [ ] **Step 1: Write the failing test (FAT32 real em memória)**

`crates/infrastructure/src/linux/fat32/tests.rs`:
```rust
use super::Fat32;
use domain::VolumeLabel;
use std::io::Cursor;

#[test]
fn formats_fat32_with_label() {
    // FAT32 exige ~33 MiB+; usamos 64 MiB.
    let mut buf = Cursor::new(vec![0u8; 64 * 1024 * 1024]);
    let label = VolumeLabel::parse("BOOTUSB").unwrap();
    Fat32::format(&mut buf, &label).unwrap();

    // Reabre com fatfs e confere tipo + rótulo.
    let fs = fatfs::FileSystem::new(&mut buf, fatfs::FsOptions::new()).unwrap();
    assert_eq!(fs.fat_type(), fatfs::FatType::Fat32);
    let read_label = fs.volume_label();
    assert_eq!(read_label.trim_end(), "BOOTUSB");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p infrastructure fat32`
Expected: FAIL (tipo ausente).

- [ ] **Step 3: Implement**

`crates/infrastructure/src/linux/fat32.rs`:
```rust
//! Criação de FAT32 via crate `fatfs` (Rust puro, sem mkfs).

use domain::VolumeLabel;
use std::io::{Read, Seek, Write};

/// Formata um volume como FAT32.
pub(crate) struct Fat32;

impl Fat32 {
    /// Formata `volume` inteiro como FAT32 com o rótulo (padding a 11 bytes).
    pub(crate) fn format<T: Read + Write + Seek>(
        volume: &mut T,
        label: &VolumeLabel,
    ) -> std::io::Result<()> {
        let mut name = [b' '; 11];
        for (slot, byte) in name.iter_mut().zip(label.as_str().bytes()) {
            *slot = byte;
        }
        let options = fatfs::FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32)
            .volume_label(name);
        fatfs::format_volume(volume, options)
    }
}

#[cfg(test)]
mod tests;
```
Add to `crates/infrastructure/src/linux/mod.rs`:
```rust
mod fat32;
pub(crate) use fat32::Fat32;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p infrastructure fat32`
Expected: PASS (formatou FAT32 e releu rótulo).

- [ ] **Step 5: Commit**

```bash
git add crates/infrastructure/src/linux/fat32.rs crates/infrastructure/src/linux/fat32/tests.rs crates/infrastructure/src/linux/mod.rs
git commit -m "feat(infrastructure): criação de FAT32 nativa (fatfs)"
```

---

### Task 4: `NativeFatFormatter` (fd via udisks + orquestração)

**Files:**
- Create: `crates/infrastructure/src/linux/native_fat_formatter.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Consumes: `application::ports::{DeviceFormatter, FormatOptions}`, `application::errors::FormatError`, `domain::DevicePath`, `Partitioner`, `Fat32`, `OffsetVolume`.
- Produces: `infrastructure::linux::NativeFatFormatter` com `new()` + `Default`; impl `DeviceFormatter`.

**Notas (casca fina — validação por loopback, sem teste unitário):**
- `open_device(name) -> std::fs::File`: igual ao `Udisks2BlockWriter` — `Block.OpenDevice("rw", {flags: O_EXCL|O_SYNC|O_CLOEXEC})` via zbus blocking → `OwnedFd` → `File::from(OwnedFd)` (sem `unsafe`). Reaproveite o mesmo mapeamento de erro (`Unauthorized`/`DeviceBusy`/`Backend`).
- `device_size(name) -> u64`: ler `/sys/block/<name>/size` × 512 (mesmo do `Udisks2BlockWriter`).
- Fluxo bloqueante (`run`): `name = device.as_str().trim_start_matches("/dev/")`; `(start, len) = Partitioner::single_partition(device_size)`; se `len == 0` → `FormatError::Backend("dispositivo muito pequeno para FAT32")`; abre o fd; se `!quick` zera `[start, start+len)` (escreve zeros em blocos de 4 MiB); `Partitioner::write_table(&mut file, scheme, start, len)`; cria `OffsetVolume::new(&mut file, start, len)` e `Fat32::format(&mut window, label)`; `file.sync_all()`.
- `async fn format`: extrai `scheme/label/quick` de `options` (ignora qualquer campo de FS — sempre FAT32) e roda `run` em `tokio::task::spawn_blocking`; mapeia `JoinError` → `Backend`.
- Mantenha ≤199 linhas (helpers privados `open_device`, `device_size`, `zero_region`, `run`). Espelhe o padrão zbus do `udisks2_block_writer.rs`.

- [ ] **Step 1: Implement**

Escreva `native_fat_formatter.rs` conforme as notas e registre:
```rust
mod native_fat_formatter;
pub use native_fat_formatter::NativeFatFormatter;
```
(em `crates/infrastructure/src/linux/mod.rs`)

- [ ] **Step 2: Build + lints + line-limit**

Run: `cargo build -p infrastructure`
Run: `cargo clippy -p infrastructure --all-targets -- -D warnings`
Run: `cargo xtask check`
Expected: builds; sem warnings; ≤199 linhas.

- [ ] **Step 3: Manual validation (loopback, fora do CI)**

```bash
truncate -s 256M /tmp/fmt.img
sudo losetup -fP /tmp/fmt.img      # ex.: /dev/loop0
```
Rode `nur`, modo Formatar, GPT + rótulo, Iniciar (aponte para o loop device). Depois:
```bash
lsblk -f /dev/loop0     # deve mostrar 1 partição vfat com o rótulo
sudo blkid /dev/loop0p1
sudo losetup -d /dev/loop0
```
**Sem teste automatizado aqui** (precisa udisks + device); a lógica está testada em `OffsetVolume`/`Partitioner`/`Fat32`.

- [ ] **Step 4: Commit**

```bash
git add crates/infrastructure/src/linux/native_fat_formatter.rs crates/infrastructure/src/linux/mod.rs
git commit -m "feat(infrastructure): NativeFatFormatter (GPT/MBR + FAT32 via fd udisks)"
```

---

### Task 5: religar o `app` ao formatador nativo

**Files:**
- Modify: `crates/app/src/commands.rs`

**Interfaces:**
- Consumes: `infrastructure::linux::NativeFatFormatter`.

- [ ] **Step 1: Trocar o formatador**

Em `crates/app/src/commands.rs`:
- No import de `infrastructure::linux`, troque `Udisks2Formatter` por `NativeFatFormatter`.
- Em `fn format`, troque `FormatDevice::new(Arc::new(Udisks2Formatter::new()))` por `FormatDevice::new(Arc::new(NativeFatFormatter::new()))`.

- [ ] **Step 2: Build + lints**

Run: `cargo build --bin nur`
Run: `cargo clippy -p app --all-targets -- -D warnings`
Expected: builds; sem warnings (o `Udisks2Formatter` ainda existe nesta etapa — removido na Task 6).

- [ ] **Step 3: Commit**

```bash
git add crates/app/src/commands.rs
git commit -m "feat(app): usa NativeFatFormatter no modo Formatar"
```

---

### Task 6: limpeza — remover FS extra, `Udisks2Formatter`, `ToolMissing`

**Files:**
- Delete: `crates/infrastructure/src/linux/udisks2_formatter.rs` e `crates/infrastructure/src/linux/udisks2_formatter/` (tests)
- Modify: `crates/infrastructure/src/linux/mod.rs`
- Modify: `crates/domain/src/format_options.rs`, `crates/domain/src/format_options/tests.rs`, `crates/domain/src/lib.rs`
- Modify: `crates/application/src/ports/format.rs`, `crates/application/src/errors.rs`, `crates/application/src/errors/tests.rs`
- Modify: `crates/application/src/use_cases/format_device/tests.rs`
- Modify: `crates/ui/src/app/options.rs`, `crates/ui/src/app/modal.rs`

**Interfaces:**
- Produces: `FormatOptions::new(scheme: PartitionScheme, label: VolumeLabel, quick: bool)` (sem `filesystem`); `FilesystemKind` deixa de existir; `FormatError` sem `ToolMissing`.

- [ ] **Step 1: domain — remover `FilesystemKind`**

Em `crates/domain/src/format_options.rs`, remova o enum `FilesystemKind` (mantém `PartitionScheme`). Em `crates/domain/src/format_options/tests.rs`, remova o teste `filesystem_variants_distinct`. Em `crates/domain/src/lib.rs`, troque `pub use format_options::{FilesystemKind, PartitionScheme};` por `pub use format_options::PartitionScheme;`.

- [ ] **Step 2: application — `FormatOptions` sem `filesystem`, `FormatError` sem `ToolMissing`**

Em `crates/application/src/ports/format.rs`: remova o import `FilesystemKind`, o campo `filesystem`, o parâmetro do `new` e o getter `filesystem()`. Resultado:
```rust
use domain::{PartitionScheme, VolumeLabel};

/// Como formatar o dispositivo.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    scheme: PartitionScheme,
    label: VolumeLabel,
    quick: bool,
}

impl FormatOptions {
    /// Cria as opções.
    #[must_use]
    pub fn new(scheme: PartitionScheme, label: VolumeLabel, quick: bool) -> Self {
        Self { scheme, label, quick }
    }

    /// Esquema de partição.
    #[must_use]
    pub fn scheme(&self) -> PartitionScheme {
        self.scheme
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
Em `crates/application/src/errors.rs`: remova a variante `ToolMissing(String)` de `FormatError`. Em `crates/application/src/errors/tests.rs`: remova as asserções de `ToolMissing` do teste `format_error_messages_are_in_ptbr` (mantenha as de `Unauthorized`/`DeviceBusy`).

- [ ] **Step 3: use case test — ajustar `FormatOptions::new`**

Em `crates/application/src/use_cases/format_device/tests.rs`, a função `options()` chama `FormatOptions::new(PartitionScheme::Gpt, FilesystemKind::Fat32, label, true)` — troque para `FormatOptions::new(PartitionScheme::Gpt, label, true)` e remova o import de `FilesystemKind`.

- [ ] **Step 4: infrastructure — remover o `Udisks2Formatter`**

Apague `crates/infrastructure/src/linux/udisks2_formatter.rs` e a pasta `udisks2_formatter/` (tests). Em `crates/infrastructure/src/linux/mod.rs`, remova `mod udisks2_formatter;` e `pub use udisks2_formatter::Udisks2Formatter;`.

- [ ] **Step 5: ui — remover o seletor de filesystem**

Em `crates/ui/src/app/modal.rs`: na `fn dispatch`, no braço `Mode::Format`, chame `FormatOptions::new(self.partition_scheme(), label, self.quick_format)` (sem `self.filesystem_kind()`); **remova** o helper `filesystem_kind`. Em `crates/ui/src/app/options.rs`: no ramo do modo Formatar, **remova** o `LabeledSelect` de "Sistema de arquivos" (deixe só "Esquema de partição" numa linha, depois "Rótulo do volume" e "Formatação rápida"); o campo `self.filesystem` deixa de ser usado no Format — se ficar sem nenhum uso no crate, remova o campo `filesystem` de `NurApp` e suas referências (o modo Boot ainda usa? confira: no Boot o `options_section` mostra `FILESYSTEMS`/`self.filesystem` — se mantém no Boot, **não** remova o campo; apenas pare de usá-lo no Format). Mantenha o `FILESYSTEMS`/seletor no modo Boot como está.

> Nota: o modo Boot (gravação raw) ainda exibe os controles de formato (decorativos) — fora do escopo deste incremento mexer nisso; só o modo **Formatar** muda aqui.

- [ ] **Step 6: Build, test, lints, xtask, fmt**

Run: `cargo build --bin nur`
Run: `cargo test --workspace`
Run: `cargo clippy --workspace --all-targets -- -D warnings`
Run: `cargo xtask check`
Run: `cargo fmt --check`
Run: `cargo machete` (se disponível) — confirma que nenhuma dep ficou órfã.
Expected: tudo verde; `FilesystemKind`/`ToolMissing`/`Udisks2Formatter` sumiram.

- [ ] **Step 7: Visual validation (screenshot)**

```bash
NUR_CAPTURE=/tmp/nur-fmt.png NUR_DEMO=format NUR_THEME=light \
LIBGL_ALWAYS_SOFTWARE=1 WINIT_UNIX_BACKEND=x11 \
timeout 60 xvfb-run -a -s "-screen 0 900x1000x24" ./target/debug/nur
```
Leia `/tmp/nur-fmt.png` e confirme que o modo Formatar mostra **Esquema de partição + Rótulo + Formatação rápida**, **sem** "Sistema de arquivos".

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor: modo Formatar só FAT32 (remove FilesystemKind, ToolMissing, Udisks2Formatter)"
```

---

## Self-Review (preenchido pelo autor do plano)

**Spec coverage:**
- FAT32 nativo via fatfs → Task 3. GPT/MBR nativo → Task 2. fd via udisks (sem Block.Format) + orquestração + quick/zero → Task 4. Janela da partição → Task 1 (OffsetVolume). UI sem seletor de FS → Task 6. Remoção de FilesystemKind/filesystem/ToolMissing/Udisks2Formatter → Task 6. Wiring → Task 5. Testabilidade em buffer (fatfs/gpt/offset/geometria) → Tasks 1–3. Validação loopback → Task 4. Deps + deny → Task 1. ADR 0010 já commitado.
- **Sem gaps.**

**Type consistency:** `OffsetVolume::new(inner,start,len)` (T1) usado na T4; `Partitioner::single_partition(bytes)->(u64,u64)` e `write_table(dev,scheme,start,len)` (T2) usados na T4; `Fat32::format(volume,&VolumeLabel)` (T3) usado na T4; `NativeFatFormatter` (T4) usado na T5; `FormatOptions::new(scheme,label,quick)` (T6) usado em modal.rs e no teste do use case; `DeviceFormatter::format(&DevicePath,&FormatOptions)` inalterado.

**Notas ao executor:**
- Ordem importa: Tasks 1–4 adicionam o novo; T5 religa; **T6 remove o antigo** (compila porque nada mais usa `FilesystemKind`/`ToolMissing`/`Udisks2Formatter`).
- As APIs de `gpt`/`mbrman`/`fatfs` variam por versão — as **assinaturas públicas e os testes** aqui são o alvo fixo; ajuste as chamadas internas dos crates à versão resolvida (round-trip nos testes confirma).
- T4 espelha o padrão zbus/fd do `udisks2_block_writer.rs` (Fase 4) — consulte-o.
