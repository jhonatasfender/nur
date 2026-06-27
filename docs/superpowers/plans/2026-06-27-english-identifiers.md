# English Identifiers + Field Encapsulation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Translate all Portuguese identifiers to English and enforce zero public fields across all crates.

**Architecture:** Bottom-up pass (domain → application → infrastructure → ui → app → xtask), rename files first, then update identifiers, then encapsulate structs. Each crate compiles before moving on.

**Tech Stack:** Rust 1.88 / edition 2024, egui 0.35, eframe 0.35, thiserror 2, cargo workspace.

## Global Constraints

- Edition 2024; Rust 1.88
- `unsafe_code = "forbid"`; `unwrap_used/expect_used/panic = "deny"` (except in tests)
- `missing_docs = "deny"`: every public item must have `///`; every file must have `//!`
- `unreachable_pub = "deny"`: use the most restrictive visibility possible
- `unnameable_types = "deny"`
- Zero `pub` fields — all struct fields private; constructors `new` + getters
- Max 199 lines per `.rs` file
- Tests in sibling file (`foo.rs` → `foo/tests.rs`)
- Comments, log messages (`eprintln!`/`println!`), and UI text stay in Portuguese
- Binary name `nur`, env var `NUR_CAPTURE`, crate names in Cargo.toml: unchanged
- Model data strings ("SanDisk Ultra", etc.): unchanged

---

### Task 1: Domain — rename files and translate identifiers

**Files:**
- Rename: `crates/domain/src/caminho_dispositivo.rs` → `crates/domain/src/device_path.rs`
- Rename: `crates/domain/src/caminho_dispositivo/tests.rs` → `crates/domain/src/device_path/tests.rs`
- Rename: `crates/domain/src/dispositivo.rs` → `crates/domain/src/device.rs`
- Rename: `crates/domain/src/dispositivo/tests.rs` → `crates/domain/src/device/tests.rs`
- Rename: `crates/domain/src/rotulo_volume.rs` → `crates/domain/src/volume_label.rs`
- Rename: `crates/domain/src/rotulo_volume/tests.rs` → `crates/domain/src/volume_label/tests.rs`
- Modify: `crates/domain/src/lib.rs`
- Modify: `crates/domain/src/byte_size.rs` (fix humanize + rename local vars)
- Modify: `crates/domain/src/byte_size/tests.rs` (English test names + new boundary test)

**Interfaces:**
- Produces: `DevicePath::new(String) -> Self`, `DevicePath::as_str(&self) -> &str`
- Produces: `Device::new(DevicePath, String, ByteSize, bool) -> Self`; getters `path()`, `model()`, `size()`, `removable()`, `description()`
- Produces: `VolumeLabel::parse(&str) -> Result<Self, InvalidLabel>`; `VolumeLabel::as_str()`
- Produces: `InvalidLabel` (variants `Empty`, `TooLong`)
- Produces: `ByteSize::humanize()` that maps `999_999` → `"1.0 MB"`

- [ ] **Step 1: Create device_path.rs**

```rust
//! Caminho de um dispositivo de bloco (ex.: `/dev/sdb`).

/// Caminho de dispositivo de bloco no sistema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevicePath(String);

impl DevicePath {
    /// Cria a partir de um caminho já validado pelo adapter de SO.
    #[must_use]
    pub fn new(path: String) -> Self {
        Self(path)
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

Write to `crates/domain/src/device_path.rs`.

- [ ] **Step 2: Create device_path/tests.rs**

```rust
use super::*;

#[test]
fn exposes_path() {
    let c = DevicePath::new("/dev/sdb".to_owned());
    assert_eq!(c.as_str(), "/dev/sdb");
}
```

- [ ] **Step 3: Create device.rs**

```rust
//! Dispositivo de bloco detectado (pendrive) como agregado de domínio.

use crate::{ByteSize, DevicePath};

/// Um dispositivo de armazenamento detectado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    path: DevicePath,
    model: String,
    size: ByteSize,
    removable: bool,
}

impl Device {
    /// Cria um dispositivo a partir dos dados do adapter de SO.
    #[must_use]
    pub fn new(
        path: DevicePath,
        model: String,
        size: ByteSize,
        removable: bool,
    ) -> Self {
        Self { path, model, size, removable }
    }

    /// Caminho do dispositivo.
    #[must_use]
    pub fn path(&self) -> &DevicePath {
        &self.path
    }

    /// Modelo do dispositivo.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Tamanho do dispositivo.
    #[must_use]
    pub fn size(&self) -> ByteSize {
        self.size
    }

    /// Indica se o dispositivo é removível.
    #[must_use]
    pub fn removable(&self) -> bool {
        self.removable
    }

    /// Descrição legível para a UI (modelo — tamanho (caminho)).
    #[must_use]
    pub fn description(&self) -> String {
        format!(
            "{} — {} ({})",
            self.model,
            self.size.humanize(),
            self.path.as_str()
        )
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Create device/tests.rs**

```rust
use super::*;

#[test]
fn builds_readable_description() {
    let d = Device::new(
        DevicePath::new("/dev/sdb".to_owned()),
        "SanDisk Ultra".to_owned(),
        ByteSize::from_bytes(32_000_000_000),
        true,
    );
    assert_eq!(d.description(), "SanDisk Ultra — 32.0 GB (/dev/sdb)");
    assert!(d.removable());
}
```

- [ ] **Step 5: Create volume_label.rs**

```rust
//! Rótulo de volume com validação (limite FAT de 11 caracteres).

/// Erro de rótulo de volume inválido.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InvalidLabel {
    /// O rótulo estava vazio.
    #[error("rótulo vazio")]
    Empty,
    /// O rótulo excede 11 caracteres.
    #[error("rótulo excede 11 caracteres")]
    TooLong,
}

/// Rótulo de volume válido (1–11 caracteres).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeLabel(String);

impl VolumeLabel {
    /// Valida e cria um rótulo. Erros: vazio ou acima de 11 caracteres.
    ///
    /// # Errors
    /// Retorna [`InvalidLabel`] quando a string não respeita o limite.
    pub fn parse(text: &str) -> Result<Self, InvalidLabel> {
        if text.is_empty() {
            return Err(InvalidLabel::Empty);
        }
        if text.chars().count() > 11 {
            return Err(InvalidLabel::TooLong);
        }
        Ok(Self(text.to_owned()))
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

- [ ] **Step 6: Create volume_label/tests.rs**

```rust
use super::*;

#[test]
fn accepts_valid_label() {
    let r = VolumeLabel::parse("BOOTUSB").unwrap();
    assert_eq!(r.as_str(), "BOOTUSB");
}

#[test]
fn rejects_empty() {
    assert!(VolumeLabel::parse("").is_err());
}

#[test]
fn rejects_above_11_chars() {
    assert!(VolumeLabel::parse("ABCDEFGHIJKL").is_err());
}
```

- [ ] **Step 7: Fix byte_size.rs humanize + rename locals**

Replace the `humanize` method to avoid "1000.0 KB" rounding:

```rust
/// Formata em unidade humana (ex.: "32.0 GB"). Base decimal (1000).
#[must_use]
pub fn humanize(self) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = self.0 as f64;
    let mut index = 0usize;
    while index < UNITS.len() - 1 {
        let next = value / 1000.0;
        if next < 1.0 {
            break;
        }
        // Avoid showing "1000.0 <unit>" due to rounding.
        if next >= 999.95 && index + 1 < UNITS.len() - 1 {
            value = next / 1000.0;
            index += 2;
        } else {
            value = next;
            index += 1;
        }
    }
    if index == 0 {
        format!("{} B", self.0)
    } else {
        format!("{value:.1} {}", UNITS[index])
    }
}
```

- [ ] **Step 8: Update byte_size/tests.rs with English names + boundary test**

```rust
use super::*;

#[test]
fn humanizes_small_bytes() {
    assert_eq!(ByteSize::from_bytes(512).humanize(), "512 B");
}

#[test]
fn humanizes_gigabytes() {
    assert_eq!(ByteSize::from_bytes(32_000_000_000).humanize(), "32.0 GB");
}

#[test]
fn preserves_byte_count() {
    assert_eq!(ByteSize::from_bytes(1024).as_bytes(), 1024);
}

#[test]
fn avoids_rounding_to_1000_kb() {
    // 999_999 bytes = 999.999 KB, which rounds to 1000.0 KB — must show "1.0 MB".
    assert_eq!(ByteSize::from_bytes(999_999).humanize(), "1.0 MB");
}
```

- [ ] **Step 9: Update domain/src/lib.rs**

```rust
//! Núcleo de domínio do Nur: modelos e value objects puros, sem IO.

mod byte_size;
mod device;
mod device_path;
mod volume_label;

pub use byte_size::ByteSize;
pub use device::Device;
pub use device_path::DevicePath;
pub use volume_label::{InvalidLabel, VolumeLabel};
```

- [ ] **Step 10: Delete old Portuguese-named files**

```bash
rm crates/domain/src/caminho_dispositivo.rs
rm -rf crates/domain/src/caminho_dispositivo/
rm crates/domain/src/dispositivo.rs
rm -rf crates/domain/src/dispositivo/
rm crates/domain/src/rotulo_volume.rs
rm -rf crates/domain/src/rotulo_volume/
```

- [ ] **Step 11: Compile domain**

```bash
cargo build -p domain
```
Expected: success.

---

### Task 2: Application — rename files + translate identifiers + encapsulate DeviceView

**Files:**
- Rename: `crates/application/src/erros.rs` → `crates/application/src/errors.rs`
- Rename: `crates/application/src/use_cases/listar_dispositivos.rs` → `crates/application/src/use_cases/list_devices.rs`
- Rename: `crates/application/src/use_cases/listar_dispositivos/tests.rs` → `crates/application/src/use_cases/list_devices/tests.rs`
- Modify: `crates/application/src/lib.rs`
- Modify: `crates/application/src/use_cases/mod.rs`
- Modify: `crates/application/src/ports/mod.rs`
- Modify: `crates/application/src/ports/disk_service.rs`
- Modify: `crates/application/src/ports/ui_state.rs`

**Interfaces:**
- Produces: `DiskError::Unavailable(String)`
- Produces: `DiskService::list_devices() -> Result<Vec<Device>, DiskError>`
- Produces: `DeviceView::new(String, String) -> Self`; getters `path() -> &str`, `description() -> &str`
- Produces: `UiState::devices() -> Vec<DeviceView>`
- Produces: `ListDevices::new(Arc<dyn DiskService>) -> Self`; `execute() -> Result<Vec<DeviceView>, DiskError>`

- [ ] **Step 1: Create errors.rs**

```rust
//! Erros da camada de aplicação.

/// Falhas ao interagir com o serviço de disco.
#[derive(Debug, thiserror::Error)]
pub enum DiskError {
    /// O backend de disco está indisponível ou falhou.
    #[error("serviço de disco indisponível: {0}")]
    Unavailable(String),
}
```

- [ ] **Step 2: Update ports/disk_service.rs**

```rust
//! Porta de acesso ao disco (implementada na infraestrutura).

use crate::errors::DiskError;
use domain::Device;

/// Serviço de disco: enumera dispositivos e (futuramente) grava/formata.
pub trait DiskService: Send + Sync {
    /// Lista os dispositivos removíveis disponíveis.
    ///
    /// # Errors
    /// Retorna [`DiskError`] se o backend falhar.
    fn list_devices(&self) -> Result<Vec<Device>, DiskError>;
}
```

- [ ] **Step 3: Update ports/ui_state.rs (DeviceView encapsulated)**

```rust
//! Porta de estado da UI: o que a tela lê para se desenhar.

/// Projeção de um dispositivo para exibição na UI (sem tipos de domínio).
#[derive(Debug, Clone)]
pub struct DeviceView {
    path: String,
    description: String,
}

impl DeviceView {
    /// Cria uma projeção com caminho e descrição.
    #[must_use]
    pub fn new(path: String, description: String) -> Self {
        Self { path, description }
    }

    /// Caminho do dispositivo (ex.: `/dev/sdb`).
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Descrição legível (modelo — tamanho (caminho)).
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Estado lido pela UI a cada frame.
pub trait UiState: Send + Sync {
    /// Lista de dispositivos para popular o seletor.
    fn devices(&self) -> Vec<DeviceView>;
}
```

- [ ] **Step 4: Update ports/mod.rs**

```rust
//! Portas (traits) que a aplicação define e a infraestrutura implementa.

mod disk_service;
mod ui_state;

pub use disk_service::DiskService;
pub use ui_state::{DeviceView, UiState};
```

- [ ] **Step 5: Create use_cases/list_devices.rs**

```rust
//! Caso de uso: listar dispositivos disponíveis para exibição.

use crate::errors::DiskError;
use crate::ports::{DeviceView, DiskService};
use std::sync::Arc;

/// Lista dispositivos e os projeta para a UI.
pub struct ListDevices {
    service: Arc<dyn DiskService>,
}

impl ListDevices {
    /// Cria o caso de uso com a porta de disco injetada.
    #[must_use]
    pub fn new(service: Arc<dyn DiskService>) -> Self {
        Self { service }
    }

    /// Executa a listagem e mapeia para [`DeviceView`].
    ///
    /// # Errors
    /// Propaga [`DiskError`] do backend.
    pub fn execute(&self) -> Result<Vec<DeviceView>, DiskError> {
        let devices = self.service.list_devices()?;
        Ok(devices
            .into_iter()
            .map(|d| DeviceView::new(d.path().as_str().to_owned(), d.description()))
            .collect())
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 6: Create use_cases/list_devices/tests.rs**

```rust
use super::*;
use domain::{ByteSize, Device, DevicePath};

struct DiskServiceFake;
impl DiskService for DiskServiceFake {
    fn list_devices(&self) -> Result<Vec<Device>, DiskError> {
        Ok(vec![Device::new(
            DevicePath::new("/dev/sdb".to_owned()),
            "SanDisk Ultra".to_owned(),
            ByteSize::from_bytes(32_000_000_000),
            true,
        )])
    }
}

#[test]
fn maps_devices_to_views() {
    let uc = ListDevices::new(Arc::new(DiskServiceFake));
    let views = uc.execute().unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].path(), "/dev/sdb");
    assert!(views[0].description().contains("SanDisk Ultra"));
}
```

- [ ] **Step 7: Update use_cases/mod.rs**

```rust
//! Casos de uso da aplicação.

mod list_devices;

pub use list_devices::ListDevices;
```

- [ ] **Step 8: Update application/src/lib.rs**

```rust
//! Casos de uso e portas do Nur (regra de negócio orquestrada).

pub mod errors;
pub mod ports;
pub mod use_cases;
```

- [ ] **Step 9: Delete old Portuguese-named files**

```bash
rm crates/application/src/erros.rs
rm crates/application/src/use_cases/listar_dispositivos.rs
rm -rf crates/application/src/use_cases/listar_dispositivos/
```

- [ ] **Step 10: Compile application**

```bash
cargo build -p application
```
Expected: success.

---

### Task 3: Infrastructure — update method name

**Files:**
- Modify: `crates/infrastructure/src/stub/disk_service_stub.rs`
- Modify: `crates/infrastructure/src/stub/disk_service_stub/tests.rs`

**Interfaces:**
- Consumes: `DiskService::list_devices()` from application
- Consumes: `Device`, `DevicePath`, `ByteSize` from domain

- [ ] **Step 1: Update disk_service_stub.rs**

```rust
//! Adapter stub do DiskService (dados canônicos para preview/desenvolvimento).

use application::errors::DiskError;
use application::ports::DiskService;
use domain::{ByteSize, Device, DevicePath};

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
    fn list_devices(&self) -> Result<Vec<Device>, DiskError> {
        Ok(vec![
            Device::new(
                DevicePath::new("/dev/sdb".to_owned()),
                "SanDisk Ultra".to_owned(),
                ByteSize::from_bytes(32_000_000_000),
                true,
            ),
            Device::new(
                DevicePath::new("/dev/sdc".to_owned()),
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

- [ ] **Step 2: Update disk_service_stub/tests.rs**

```rust
use super::*;

#[test]
fn returns_two_canonical_devices() {
    let stub = DiskServiceStub::new();
    let devices = stub.list_devices().unwrap();
    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].path().as_str(), "/dev/sdb");
}
```

- [ ] **Step 3: Compile infrastructure**

```bash
cargo build -p infrastructure
```
Expected: success.

---

### Task 4: UI — rename captura→capture, translate identifiers, encapsulate Palette

**Files:**
- Rename: `crates/ui/src/captura.rs` → `crates/ui/src/capture.rs`
- Rename: `crates/ui/src/captura/tests.rs` → `crates/ui/src/capture/tests.rs`
- Modify: `crates/ui/src/lib.rs`
- Modify: `crates/ui/src/app.rs`
- Modify: `crates/ui/src/app/tests.rs`
- Modify: `crates/ui/src/theme/palette.rs`
- Modify: `crates/ui/src/theme/palette/tests.rs`
- Modify: `crates/ui/src/theme/theme_kit.rs`
- Modify: `crates/ui/src/theme/theme_kit/tests.rs`
- Modify: `crates/ui/src/theme/mod.rs`

**Interfaces:**
- Produces: `Palette` with private fields; getters `background()`, `surface()`, `text()`, `destructive()`, `success()` (all `const fn -> Color32`); ctors `light()`, `dark()`
- Produces: `ThemePreference::{Light, Dark}`; `toggle()` method
- Produces: `Capturer` (was `Capturador`); methods `auto_enabled()`, `message()`, `process()`, private `request()`, `save_ready()`, `next_destination()`, `save_png()`
- Produces: `NurApp` fields `state`, `theme`, `theme_installed`, `capturer`; builder `with_theme()`

- [ ] **Step 1: Create capture.rs (was captura.rs, preserve eprintln! in Portuguese)**

```rust
//! Captura de tela da janela do Nur (tecla F12 e modo automático via env).

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Coordena capturas de tela da janela.
///
/// Dispara por **F12** (captura manual, numerada) ou pelo **modo automático**,
/// ativado pela variável de ambiente `NUR_CAPTURE=<arquivo.png>` — útil para
/// validar a UI de forma headless: renderiza alguns frames, salva o PNG e
/// sinaliza para a janela fechar.
pub struct Capturer {
    auto: Option<PathBuf>,
    auto_requested: bool,
    frames: u32,
    counter: u32,
    last_msg: Option<String>,
}

impl Capturer {
    /// Cria o capturador, lendo `NUR_CAPTURE` para o modo automático.
    #[must_use]
    pub fn new() -> Self {
        let auto = std::env::var_os("NUR_CAPTURE").map(PathBuf::from);
        Self {
            auto,
            auto_requested: false,
            frames: 0,
            counter: 0,
            last_msg: None,
        }
    }

    /// Indica se a captura automática está configurada.
    #[must_use]
    pub fn auto_enabled(&self) -> bool {
        self.auto.is_some()
    }

    /// Mensagem de status da última captura, se houver.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.last_msg.as_deref()
    }

    /// Processa um frame: trata F12 / modo automático e salva screenshots prontos.
    ///
    /// Retorna `true` quando a captura automática concluiu (a janela deve fechar).
    pub fn process(&mut self, ctx: &egui::Context) -> bool {
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            Self::request(ctx);
        }
        if self.auto.is_some() {
            // Failsafe: se o screenshot não chegar (ex.: sem framebuffer),
            // aborta em vez de girar para sempre.
            const MAX_FRAMES_AUTO: u32 = 600;
            self.frames += 1;
            // Espera a UI estabilizar antes de capturar no modo automático.
            if self.frames >= 3 && !self.auto_requested {
                Self::request(ctx);
                self.auto_requested = true;
            }
            if self.frames > MAX_FRAMES_AUTO {
                eprintln!("NUR_CAPTURE: screenshot não chegou após {MAX_FRAMES_AUTO} frames; abortando.");
                return true;
            }
            // egui é reativo; força frames para o screenshot chegar.
            ctx.request_repaint();
        }
        self.save_ready(ctx)
    }

    fn request(ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
    }

    fn save_ready(&mut self, ctx: &egui::Context) -> bool {
        let images: Vec<Arc<egui::ColorImage>> = ctx.input(|i| {
            i.raw
                .events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
                .collect()
        });
        let mut auto_done = false;
        for image in images {
            let dest = self.next_destination();
            match Self::save_png(&image, &dest) {
                Ok(()) => {
                    self.last_msg = Some(format!("captura salva em {}", dest.display()));
                    auto_done = self.auto.is_some();
                }
                Err(e) => self.last_msg = Some(format!("falha na captura: {e}")),
            }
        }
        auto_done
    }

    fn next_destination(&mut self) -> PathBuf {
        if let Some(path) = &self.auto {
            return path.clone();
        }
        self.counter += 1;
        PathBuf::from(format!("nur-screenshot-{:03}.png", self.counter))
    }

    fn save_png(image: &egui::ColorImage, dest: &Path) -> Result<(), image::ImageError> {
        let [width, height] = image.size;
        image::save_buffer(
            dest,
            image.as_raw(),
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgba8,
        )
    }
}

impl Default for Capturer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 2: Create capture/tests.rs**

```rust
use super::*;

#[test]
fn saves_png_of_simple_image() {
    let image = egui::ColorImage::filled([4, 4], egui::Color32::from_rgb(10, 20, 30));
    let dest = std::env::temp_dir().join("nur_capture_test.png");
    let _ = std::fs::remove_file(&dest);
    Capturer::save_png(&image, &dest).unwrap();
    let meta = std::fs::metadata(&dest).unwrap();
    assert!(meta.len() > 0);
    std::fs::remove_file(&dest).unwrap();
}

#[test]
fn manual_destination_increments_and_numbers() {
    let mut cap = Capturer {
        auto: None,
        auto_requested: false,
        frames: 0,
        counter: 0,
        last_msg: None,
    };
    let p1 = cap.next_destination();
    let p2 = cap.next_destination();
    assert_ne!(p1, p2);
    assert!(p1.to_string_lossy().contains("001"));
    assert!(p2.to_string_lossy().contains("002"));
}
```

- [ ] **Step 3: Update palette.rs (private fields + const getters)**

```rust
//! Paletas de cores claro/escuro, espelhando os protótipos HTML.

use egui::Color32;

/// Conjunto de cores de um tema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    background: Color32,
    surface: Color32,
    text: Color32,
    destructive: Color32,
    success: Color32,
}

impl Palette {
    /// Tema claro (fundo cinza-claro, superfície branca).
    #[must_use]
    pub const fn light() -> Self {
        Self {
            background: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            surface: Color32::WHITE,
            text: Color32::from_rgb(0x11, 0x18, 0x27),
            destructive: Color32::from_rgb(0xDC, 0x26, 0x26),
            success: Color32::from_rgb(0x16, 0xA3, 0x4A),
        }
    }

    /// Tema escuro (fundo quase preto, superfície cinza-escuro).
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            background: Color32::from_rgb(0x0A, 0x0A, 0x0A),
            surface: Color32::from_rgb(0x11, 0x18, 0x27),
            text: Color32::from_rgb(0xF3, 0xF4, 0xF6),
            destructive: Color32::from_rgb(0xDC, 0x26, 0x26),
            success: Color32::from_rgb(0x16, 0xA3, 0x4A),
        }
    }

    /// Cor de fundo da janela.
    #[must_use]
    pub const fn background(self) -> Color32 {
        self.background
    }

    /// Cor das superfícies/cards.
    #[must_use]
    pub const fn surface(self) -> Color32 {
        self.surface
    }

    /// Cor de texto primário.
    #[must_use]
    pub const fn text(self) -> Color32 {
        self.text
    }

    /// Acento destrutivo (vermelho).
    #[must_use]
    pub const fn destructive(self) -> Color32 {
        self.destructive
    }

    /// Acento de sucesso (verde).
    #[must_use]
    pub const fn success(self) -> Color32 {
        self.success
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Update palette/tests.rs**

```rust
use super::*;

#[test]
fn themes_have_different_backgrounds() {
    assert_ne!(Palette::light().background(), Palette::dark().background());
}

#[test]
fn success_is_green_in_both_themes() {
    // Verde de sucesso é o mesmo token (#16A34A) nos dois temas.
    assert_eq!(Palette::light().success(), Palette::dark().success());
}
```

- [ ] **Step 5: Update theme_kit.rs**

```rust
//! Preferência de tema e instalação dos `Visuals` no contexto egui.

use crate::theme::Palette;

/// Preferência de tema escolhida pelo usuário (persistível).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemePreference {
    /// Tema claro.
    Light,
    /// Tema escuro.
    Dark,
}

impl ThemePreference {
    /// Alterna entre claro e escuro.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    /// Paleta correspondente a esta preferência.
    #[must_use]
    pub const fn palette(self) -> Palette {
        match self {
            Self::Light => Palette::light(),
            Self::Dark => Palette::dark(),
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
            ThemePreference::Light => egui::Visuals::light(),
            ThemePreference::Dark => egui::Visuals::dark(),
        };
        visuals.panel_fill = palette.background();
        visuals.window_fill = palette.surface();
        visuals.override_text_color = Some(palette.text());
        ctx.set_visuals(visuals);
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 6: Update theme_kit/tests.rs**

```rust
use super::*;

#[test]
fn toggles_between_light_and_dark() {
    assert_eq!(ThemePreference::Light.toggle(), ThemePreference::Dark);
    assert_eq!(ThemePreference::Dark.toggle(), ThemePreference::Light);
}

#[test]
fn preference_resolves_palette() {
    assert_eq!(ThemePreference::Dark.palette(), Palette::dark());
}
```

- [ ] **Step 7: Update app.rs**

```rust
//! Aplicação egui do Nur (presenter; consome portas via `Arc<dyn _>`).

use crate::capture::Capturer;
use crate::theme::{ThemeKit, ThemePreference};
use application::ports::UiState;
use std::sync::Arc;

/// App egui do Nur. Lê o estado por uma porta injetada.
pub struct NurApp {
    state: Arc<dyn UiState>,
    theme: ThemePreference,
    theme_installed: bool,
    capturer: Capturer,
}

impl NurApp {
    /// Cria o app com o estado injetado (tema padrão: escuro).
    #[must_use]
    pub fn new(state: Arc<dyn UiState>) -> Self {
        Self {
            state,
            theme: ThemePreference::Dark,
            theme_installed: false,
            capturer: Capturer::new(),
        }
    }

    /// Builder: define a preferência de tema inicial.
    #[must_use]
    pub fn with_theme(mut self, pref: ThemePreference) -> Self {
        self.theme = pref;
        self
    }

    #[cfg(test)]
    pub(crate) fn theme(&self) -> ThemePreference {
        self.theme
    }
}

impl eframe::App for NurApp {
    /// Instala o tema antes de redesenhar (eframe 0.35: logic recebe ctx).
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.theme_installed {
            ThemeKit::install(ctx, self.theme);
            self.theme_installed = true;
        }
        // Captura de tela (F12 ou modo automático). Fecha ao concluir a captura automática.
        if self.capturer.process(ctx) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Desenha o painel central (eframe 0.35: ui recebe &mut Ui).
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Nur");
        ui.label("Formatador de Pendrive & Criador de Boot");
        if ui.button("Alternar tema").clicked() {
            self.theme = self.theme.toggle();
            self.theme_installed = false;
        }
        ui.separator();
        ui.label("Dispositivos detectados:");
        for d in self.state.devices() {
            ui.label(d.description());
        }
        ui.separator();
        ui.label("Pressione F12 para capturar a tela.");
        if let Some(msg) = self.capturer.message() {
            ui.label(msg);
        }
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 8: Update app/tests.rs**

```rust
use super::*;
use application::ports::DeviceView;

struct UiStateFake;
impl UiState for UiStateFake {
    fn devices(&self) -> Vec<DeviceView> {
        vec![DeviceView::new(
            "/dev/sdb".to_owned(),
            "Teste — 32.0 GB (/dev/sdb)".to_owned(),
        )]
    }
}

#[test]
fn builder_sets_theme() {
    let app = NurApp::new(Arc::new(UiStateFake)).with_theme(ThemePreference::Light);
    assert_eq!(app.theme(), ThemePreference::Light);
}
```

- [ ] **Step 9: Update ui/src/lib.rs**

```rust
//! Camada de apresentação (egui) do Nur.

mod app;
mod capture;
pub mod theme;

pub use app::NurApp;
pub use capture::Capturer;
```

- [ ] **Step 10: Delete old Portuguese-named files**

```bash
rm crates/ui/src/captura.rs
rm -rf crates/ui/src/captura/
```

- [ ] **Step 11: Compile ui**

```bash
cargo build -p ui
```
Expected: success.

---

### Task 5: App crate — translate identifiers

**Files:**
- Modify: `crates/app/src/main.rs`
- Modify: `crates/app/src/window.rs`

**Interfaces:**
- Consumes: `DeviceView`, `UiState`, `ListDevices` from application
- Consumes: `DiskServiceStub` from infrastructure
- Consumes: `NurApp` from ui

- [ ] **Step 1: Update window.rs**

```rust
//! Adapter de estado da UI e bootstrap da janela eframe.

use application::ports::{DeviceView, UiState};
use application::use_cases::ListDevices;
use infrastructure::stub::DiskServiceStub;
use std::sync::Arc;

/// Estado da UI alimentado pelo caso de uso (sobre o stub nesta fase).
pub(crate) struct LiveUiState {
    devices: Vec<DeviceView>,
}

impl LiveUiState {
    /// Monta o estado executando a listagem uma vez.
    ///
    /// # Errors
    /// Propaga falha do caso de uso.
    pub(crate) fn build() -> anyhow::Result<Self> {
        let uc = ListDevices::new(Arc::new(DiskServiceStub::new()));
        let devices = uc.execute()?;
        Ok(Self { devices })
    }
}

impl UiState for LiveUiState {
    fn devices(&self) -> Vec<DeviceView> {
        self.devices.clone()
    }
}

/// Abre a janela do Nur. Bloqueia até o usuário fechar.
///
/// # Errors
/// Retorna erro se o eframe falhar ao iniciar.
pub(crate) fn open(state: Arc<dyn UiState>) -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Nur",
        options,
        Box::new(|_cc| Ok(Box::new(ui::NurApp::new(state)))),
    )
    .map_err(|e| anyhow::anyhow!("falha ao iniciar a janela: {e}"))
}
```

- [ ] **Step 2: Update main.rs**

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

    let state = match window::LiveUiState::build() {
        Ok(state) => Arc::new(state),
        Err(e) => {
            eprintln!("erro ao montar estado: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    match window::open(state) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
```

- [ ] **Step 3: Compile app**

```bash
cargo build -p app
```
Expected: success.

---

### Task 6: Xtask — translate identifiers

**Files:**
- Modify: `tools/xtask/src/main.rs`
- Modify: `tools/xtask/src/line_limit/rule.rs`
- Modify: `tools/xtask/src/line_limit/rule/tests.rs`

**Interfaces:**
- `LineLimitRule::check` signature unchanged; internal locals renamed

- [ ] **Step 1: Update tools/xtask/src/main.rs**

```rust
//! Ferramenta de build do Nur (lints customizados).

mod line_limit {
    pub mod rule;
}

use line_limit::rule::LineLimitRule;
use std::path::Path;

fn main() -> std::process::ExitCode {
    let command = std::env::args().nth(1).unwrap_or_default();
    match command.as_str() {
        "line-limit" => run_line_limit(),
        _ => {
            eprintln!("uso: cargo xtask line-limit");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run_line_limit() -> std::process::ExitCode {
    let root = Path::new("crates");
    match LineLimitRule::check(root) {
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
```

- [ ] **Step 2: Update tools/xtask/src/line_limit/rule.rs**

```rust
//! Regra de limite de linhas por arquivo `.rs`.

use std::path::Path;

/// Verifica que nenhum arquivo `.rs` ultrapassa o limite de linhas.
pub struct LineLimitRule;

impl LineLimitRule {
    /// Limite (exclusivo): arquivos com `LIMIT` ou mais linhas violam a regra.
    pub const LIMIT: usize = 200;

    /// Varre `root` recursivamente e retorna os caminhos que excedem o limite.
    pub fn check(root: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut violations = Vec::new();
        Self::scan(root, &mut violations)?;
        Ok(violations)
    }

    fn scan(dir: &Path, acc: &mut Vec<String>) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                Self::scan(&path, acc)?;
            } else if path.extension().is_some_and(|e| e == "rs") {
                let lines = std::fs::read_to_string(&path)?.lines().count();
                if lines >= Self::LIMIT {
                    acc.push(format!("{} ({lines} linhas)", path.display()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 3: Update tools/xtask/src/line_limit/rule/tests.rs**

```rust
use super::*;
use std::io::Write;

#[test]
fn points_out_file_above_limit() {
    let dir = std::env::temp_dir().join("nur_xtask_test_large");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("large.rs");
    let mut f = std::fs::File::create(&file).unwrap();
    for _ in 0..LineLimitRule::LIMIT {
        writeln!(f, "// linha").unwrap();
    }
    let violations = LineLimitRule::check(&dir).unwrap();
    assert!(violations.iter().any(|v| v.contains("large.rs")));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn accepts_file_within_limit() {
    let dir = std::env::temp_dir().join("nur_xtask_test_small");
    std::fs::create_dir_all(&dir).unwrap();
    let mut f = std::fs::File::create(dir.join("small.rs")).unwrap();
    writeln!(f, "// só uma linha").unwrap();
    let violations = LineLimitRule::check(&dir).unwrap();
    assert!(violations.is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}
```

- [ ] **Step 4: Compile xtask**

```bash
cargo build --manifest-path tools/xtask/Cargo.toml
```
Expected: success.

---

### Task 7: Quality gates + commit

- [ ] **Step 1: Format**

```bash
cargo fmt --all
```

- [ ] **Step 2: Test workspace**

```bash
cargo test --workspace
```
Expected: all tests pass.

- [ ] **Step 3: Clippy**

```bash
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: zero warnings.

- [ ] **Step 4: Doc**

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
```
Expected: zero warnings.

- [ ] **Step 5: Test xtask**

```bash
cargo test --manifest-path tools/xtask/Cargo.toml
```
Expected: all tests pass.

- [ ] **Step 6: Xtask line-limit**

```bash
cargo xtask line-limit
```
Expected: `line-limit: OK`.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor: english identifiers + field encapsulation"
```
