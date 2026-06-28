# Plano 3 — Incremento 1: Enumeração real (Linux/udisks2) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Substituir o `DiskServiceStub` por enumeração real de pendrives removíveis/USB via udisks2 (zbus), atualizando a lista ao vivo (polling ~1,5s) por uma ponte tokio→egui. Read-only, sem gravar nada.

**Architecture:** Porta `DiskService` passa a assíncrona. `Udisks2DiskService` (zbus + udisks2) lista blocos, filtra removível/USB (exclui o disco de sistema) e mapeia para `domain::Device` via uma função pura testável. No `app`, uma task tokio faz polling do caso de uso `ListDevices` e escreve num `Arc<RwLock<DeviceListState>>` que a UI lê via `UiState::device_list()`, chamando `request_repaint()`.

**Tech Stack:** Rust 2024 · egui/eframe 0.35 · tokio · `udisks2 0.3` + `zbus 5` · `async-trait`.

## Global Constraints

- Edição 2024; `unsafe_code = "forbid"` (zbus/udisks2 são Rust seguro — sem `unsafe`).
- Sem-pânico em produção (`unwrap`/`expect`/`panic` só em testes).
- `missing_docs`/`unreachable_pub` = erro; todo item público com `///`, todo arquivo com `//!`.
- **Zero campos `pub`** (validado por `cargo xtask pub-fields`); máx **199 linhas** por `.rs` (`cargo xtask line-limit`).
- **Código em inglês**; comentários, logs e UI em **pt-BR**.
- Testes em arquivo irmão (`foo.rs` → `foo/tests.rs`).
- A UI nunca depende de `infrastructure`; só `app` conhece todos os crates.

---

## File Structure

```
Cargo.toml                                   # +async-trait no [workspace.dependencies]
crates/application/Cargo.toml                # +async-trait, +tokio (dev)
crates/application/src/erros.rs (errors.rs)  # DiskError::Backend
crates/application/src/ports/disk_service.rs # trait async
crates/application/src/ports/ui_state.rs     # DeviceListState + device_list()
crates/application/src/use_cases/list_devices.rs  # execute() async (+ tests async)
crates/infrastructure/Cargo.toml             # +udisks2, +zbus, +async-trait, +tokio(dev)
crates/infrastructure/src/stub/disk_service_stub.rs  # impl async
crates/infrastructure/src/linux/mod.rs       # módulo Linux
crates/infrastructure/src/linux/udisks2_disk_service.rs  # adapter + map (+ tests)
crates/infrastructure/src/lib.rs             # pub mod linux
crates/ui/src/app/draw.rs                    # device_selector usa device_list()
crates/ui/src/app/tests.rs                   # UiStateFake usa device_list()
crates/app/src/window.rs                     # LiveUiState com estado compartilhado + task de polling
crates/app/src/main.rs                       # passa egui_ctx para a ponte
crates/app/Cargo.toml                        # (já tem tokio/eframe)
```

---

### Task 1: Camada de disco assíncrona

**Files:**
- Modify: `Cargo.toml` (workspace deps), `crates/application/Cargo.toml`, `crates/infrastructure/Cargo.toml`
- Modify: `crates/application/src/errors.rs`, `crates/application/src/ports/disk_service.rs`, `crates/application/src/use_cases/list_devices.rs`
- Modify: `crates/infrastructure/src/stub/disk_service_stub.rs`
- Modify: `crates/app/src/main.rs` (interim block_on)
- Test: `crates/application/src/use_cases/list_devices/tests.rs`, `crates/infrastructure/src/stub/disk_service_stub/tests.rs`

**Interfaces:**
- Produces: `#[async_trait] trait DiskService { async fn list_devices(&self) -> Result<Vec<Device>, DiskError> }`; `ListDevices::execute(&self).await -> Result<Vec<DeviceView>, DiskError>`; `DiskError::Backend(String)`.

- [ ] **Step 1: Adicionar `async-trait` ao workspace**

Em `Cargo.toml` `[workspace.dependencies]`, após `serde`:
```toml
async-trait = "0.1"
```

- [ ] **Step 2: Deps de application e infrastructure**

`crates/application/Cargo.toml` — em `[dependencies]` adicionar `async-trait = { workspace = true }`; criar `[dev-dependencies]`:
```toml
[dev-dependencies]
tokio = { workspace = true }
```
`crates/infrastructure/Cargo.toml` — em `[dependencies]` adicionar `async-trait = { workspace = true }`; criar `[dev-dependencies]` com `tokio = { workspace = true }`.

- [ ] **Step 3: `DiskError::Backend` em `crates/application/src/errors.rs`**

```rust
//! Erros da camada de aplicação.

/// Falhas ao interagir com o serviço de disco.
#[derive(Debug, thiserror::Error)]
pub enum DiskError {
    /// O backend de disco está indisponível ou não respondeu.
    #[error("serviço de disco indisponível: {0}")]
    Unavailable(String),
    /// Falha do backend ao consultar/operar dispositivos.
    #[error("falha no backend de disco: {0}")]
    Backend(String),
}
```

- [ ] **Step 4: Trait `DiskService` assíncrona**

`crates/application/src/ports/disk_service.rs`:
```rust
//! Porta de acesso ao disco (implementada na infraestrutura).

use crate::errors::DiskError;
use domain::Device;

/// Serviço de disco: enumera dispositivos (e, futuramente, grava/formata).
#[async_trait::async_trait]
pub trait DiskService: Send + Sync {
    /// Lista os dispositivos removíveis/USB disponíveis.
    ///
    /// # Errors
    /// Retorna [`DiskError`] se o backend falhar.
    async fn list_devices(&self) -> Result<Vec<Device>, DiskError>;
}
```

- [ ] **Step 5: Atualizar o teste do caso de uso (`.../list_devices/tests.rs`) para async**

```rust
use super::*;
use domain::{ByteSize, DevicePath, Device};
use std::sync::Arc;

struct DiskServiceFake;

#[async_trait::async_trait]
impl DiskService for DiskServiceFake {
    async fn list_devices(&self) -> Result<Vec<Device>, DiskError> {
        Ok(vec![Device::new(
            DevicePath::new("/dev/sdb".to_owned()),
            "SanDisk Ultra".to_owned(),
            ByteSize::from_bytes(32_000_000_000),
            true,
        )])
    }
}

#[tokio::test]
async fn maps_devices_to_views() {
    let uc = ListDevices::new(Arc::new(DiskServiceFake));
    let views = uc.execute().await.unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].path(), "/dev/sdb");
    assert!(views[0].description().contains("SanDisk Ultra"));
}
```

- [ ] **Step 6: Rodar e ver falhar**

Run: `cargo test -p application`
Expected: FAIL de compilação (`execute` ainda síncrono; trait ainda síncrona).

- [ ] **Step 7: `ListDevices::execute` async**

`crates/application/src/use_cases/list_devices.rs` — tornar `execute` async:
```rust
    /// Executa a listagem e mapeia para [`DeviceView`].
    ///
    /// # Errors
    /// Propaga [`DiskError`] do backend.
    pub async fn execute(&self) -> Result<Vec<DeviceView>, DiskError> {
        let devices = self.service.list_devices().await?;
        Ok(devices
            .into_iter()
            .map(|d| DeviceView::new(d.path().as_str().to_owned(), d.description()))
            .collect())
    }
```
(Mantenha o restante do arquivo; só `execute` muda de assinatura/corpo.)

- [ ] **Step 8: `DiskServiceStub` async**

`crates/infrastructure/src/stub/disk_service_stub.rs` — anotar a impl:
```rust
#[async_trait::async_trait]
impl DiskService for DiskServiceStub {
    async fn list_devices(&self) -> Result<Vec<Device>, DiskError> {
        Ok(vec![
            Device::new(DevicePath::new("/dev/sdb".to_owned()), "SanDisk Ultra".to_owned(), ByteSize::from_bytes(32_000_000_000), true),
            Device::new(DevicePath::new("/dev/sdc".to_owned()), "Kingston DataTraveler".to_owned(), ByteSize::from_bytes(16_000_000_000), true),
        ])
    }
}
```
E o teste irmão `.../disk_service_stub/tests.rs` vira `#[tokio::test] async fn ...` chamando `.list_devices().await.unwrap()`.

- [ ] **Step 9: Manter o `app` compilando (interim block_on)**

`crates/app/src/window.rs` — `LiveUiState::build` recebe um `&tokio::runtime::Runtime` e usa `block_on`:
```rust
    pub(crate) fn build(runtime: &tokio::runtime::Runtime) -> anyhow::Result<Self> {
        let uc = ListDevices::new(Arc::new(DiskServiceStub::new()));
        let devices = runtime.block_on(uc.execute())?;
        Ok(Self { devices })
    }
```
`crates/app/src/main.rs` — chamar `LiveUiState::build(&runtime)` **antes** de `runtime.enter()` (block_on não pode rodar dentro do runtime). Ajustar a ordem: criar runtime → `build(&runtime)` → `enter()` → abrir janela.

- [ ] **Step 10: Rodar testes e build**

Run: `cargo test -p application -p infrastructure && cargo build --bin nur`
Expected: PASS / compila.

- [ ] **Step 11: Commit**

```bash
git add Cargo.toml crates
git commit -m "refactor: DiskService assíncrono (async-trait) + DiskError::Backend"
```

---

### Task 2: `DeviceListState` + `UiState::device_list` + render

**Files:**
- Modify: `crates/application/src/ports/ui_state.rs`
- Modify: `crates/ui/src/app/draw.rs` (device_selector), `crates/ui/src/app/tests.rs`
- Modify: `crates/app/src/window.rs` (LiveUiState implementa device_list)
- Test: `crates/application/src/ports/ui_state/tests.rs`

**Interfaces:**
- Produces: `enum DeviceListState { Loading, Ready(Vec<DeviceView>), Error(String) }` (Clone); `trait UiState { fn device_list(&self) -> DeviceListState }`.
- Consumes: `DeviceView`.

- [ ] **Step 1: Escrever teste do enum (`.../ui_state/tests.rs`)**

```rust
use super::*;

#[test]
fn ready_carries_devices() {
    let s = DeviceListState::Ready(vec![DeviceView::new("/dev/sdb".to_owned(), "x".to_owned())]);
    match s {
        DeviceListState::Ready(v) => assert_eq!(v.len(), 1),
        _ => panic!("esperava Ready"),
    }
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p application`
Expected: FAIL (`DeviceListState` inexistente).

- [ ] **Step 3: `DeviceListState` + `UiState::device_list` em `ui_state.rs`**

```rust
//! Porta de estado da UI: o que a tela lê para se desenhar.

/// Projeção de um dispositivo para exibição na UI (sem tipos de domínio).
#[derive(Debug, Clone)]
pub struct DeviceView {
    path: String,
    description: String,
}

impl DeviceView {
    /// Cria a projeção.
    #[must_use]
    pub fn new(path: String, description: String) -> Self {
        Self { path, description }
    }
    /// Caminho do dispositivo (ex.: `/dev/sdb`).
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    /// Descrição legível.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Estado da lista de dispositivos exibido pela UI.
#[derive(Debug, Clone)]
pub enum DeviceListState {
    /// Detecção em andamento.
    Loading,
    /// Lista pronta (pode estar vazia).
    Ready(Vec<DeviceView>),
    /// Falha ao detectar (mensagem para o usuário).
    Error(String),
}

/// Estado lido pela UI a cada frame.
pub trait UiState: Send + Sync {
    /// Estado atual da lista de dispositivos.
    fn device_list(&self) -> DeviceListState;
}

#[cfg(test)]
mod tests;
```
(Se `DeviceView` já estava neste arquivo, substitua a versão antiga por esta — campos privados + getters já existiam.)

- [ ] **Step 3b: Exportar `DeviceListState` em `ports/mod.rs`**

`crates/application/src/ports/mod.rs`:
```rust
pub use ui_state::{DeviceListState, DeviceView, UiState};
```

- [ ] **Step 4: `LiveUiState` implementa `device_list` (interim, sobre o stub)**

`crates/app/src/window.rs` — guardar `DeviceListState` e implementar:
```rust
impl UiState for LiveUiState {
    fn device_list(&self) -> DeviceListState {
        DeviceListState::Ready(self.devices.clone())
    }
}
```

- [ ] **Step 5: `device_selector` renderiza por estado**

`crates/ui/src/app/draw.rs` — substituir o corpo de `device_selector` para ler `self.state.device_list()`:
```rust
    fn device_selector(&mut self, ui: &mut egui::Ui, palette: Palette) {
        FieldLabel::show(ui, palette, "DISPOSITIVO");
        let devices = match self.state.device_list() {
            application::ports::DeviceListState::Loading => {
                ui.label(egui::RichText::new("Detectando…").color(palette.muted()).size(13.0));
                return;
            }
            application::ports::DeviceListState::Error(msg) => {
                ui.label(egui::RichText::new(msg).color(palette.destructive()).size(13.0));
                return;
            }
            application::ports::DeviceListState::Ready(d) => d,
        };
        let selected_text = self.selected.and_then(|i| devices.get(i)).map_or_else(
            || {
                if devices.is_empty() {
                    "Nenhum pendrive detectado".to_owned()
                } else {
                    "— Selecione o pendrive —".to_owned()
                }
            },
            |d| d.description().to_owned(),
        );
        egui::ComboBox::from_id_salt("device")
            .selected_text(selected_text)
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (i, device) in devices.iter().enumerate() {
                    ui.selectable_value(&mut self.selected, Some(i), device.description());
                }
            });
        if self.selected.is_some() {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("\u{26A0} Todos os dados deste dispositivo serão apagados.")
                    .color(palette.destructive())
                    .size(12.0),
            );
        }
    }
```

- [ ] **Step 6: Atualizar `UiStateFake` nos testes de UI (`crates/ui/src/app/tests.rs`)**

```rust
impl UiState for UiStateFake {
    fn device_list(&self) -> application::ports::DeviceListState {
        application::ports::DeviceListState::Ready(vec![DeviceView::new(
            "/dev/sdb".to_owned(),
            "Teste — 32.0 GB (/dev/sdb)".to_owned(),
        )])
    }
}
```
(Importe `application::ports::DeviceView` no topo do arquivo de teste.)

- [ ] **Step 7: Rodar testes e build**

Run: `cargo test -p application -p ui && cargo build --bin nur`
Expected: PASS / compila. (A UI ainda mostra os 2 stubs.)

- [ ] **Step 8: Commit**

```bash
git add crates
git commit -m "feat: DeviceListState + UiState::device_list (loading/ready/error na UI)"
```

---

### Task 3: `Udisks2DiskService` (adapter Linux) + mapeamento puro

**Files:**
- Create: `crates/infrastructure/src/linux/mod.rs`, `crates/infrastructure/src/linux/udisks2_disk_service.rs`
- Modify: `crates/infrastructure/src/lib.rs`, `crates/infrastructure/Cargo.toml`
- Test: `crates/infrastructure/src/linux/udisks2_disk_service/tests.rs`

**Interfaces:**
- Consumes: `DiskService`, `DiskError`, `Device`.
- Produces: `Udisks2DiskService::new() -> Self`; impl async `DiskService`. Função pura `Udisks2DiskService::map_drive(model: &str, size: u64, removable: bool, connection_bus: &str, path: &str) -> Option<Device>` (Some se removível ou USB).

- [ ] **Step 1: Deps udisks2/zbus**

`crates/infrastructure/Cargo.toml` `[dependencies]`:
```toml
udisks2 = "0.3"
zbus = "5"
```

- [ ] **Step 2: Teste do mapeamento (`.../udisks2_disk_service/tests.rs`)**

```rust
use super::*;

#[test]
fn maps_usb_removable_device() {
    let d = Udisks2DiskService::map_drive("SanDisk Ultra", 32_000_000_000, true, "usb", "/dev/sdb")
        .expect("deve mapear pendrive USB");
    assert_eq!(d.path().as_str(), "/dev/sdb");
    assert_eq!(d.model(), "SanDisk Ultra");
    assert!(d.removable());
}

#[test]
fn skips_internal_system_disk() {
    // disco interno: não removível e não-USB → excluído.
    assert!(Udisks2DiskService::map_drive("Samsung SSD", 512_000_000_000, false, "sata", "/dev/sda").is_none());
}
```

- [ ] **Step 3: Rodar e ver falhar**

Run: `cargo test -p infrastructure`
Expected: FAIL (`Udisks2DiskService` inexistente).

- [ ] **Step 4: Implementar `udisks2_disk_service.rs`**

```rust
//! Adapter de disco para Linux via udisks2 (zbus). Read-only (enumeração).

use application::errors::DiskError;
use application::ports::DiskService;
use domain::{ByteSize, Device, DevicePath};

/// Lista dispositivos removíveis/USB consultando o daemon udisks2.
pub struct Udisks2DiskService;

impl Udisks2DiskService {
    /// Cria o adapter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Mapeia um drive do udisks2 para um Device de domínio, aplicando o filtro
    // (apenas removível ou conectado por USB — exclui o disco de sistema).
    fn map_drive(
        model: &str,
        size: u64,
        removable: bool,
        connection_bus: &str,
        path: &str,
    ) -> Option<Device> {
        if !removable && connection_bus != "usb" {
            return None;
        }
        Some(Device::new(
            DevicePath::new(path.to_owned()),
            model.to_owned(),
            ByteSize::from_bytes(size),
            removable,
        ))
    }
}

impl Default for Udisks2DiskService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl DiskService for Udisks2DiskService {
    async fn list_devices(&self) -> Result<Vec<Device>, DiskError> {
        let client = udisks2::Client::new()
            .await
            .map_err(|e| DiskError::Unavailable(e.to_string()))?;
        let mut devices = Vec::new();
        let objects = client
            .object_manager()
            .get_managed_objects()
            .await
            .map_err(|e| DiskError::Backend(e.to_string()))?;
        for path in objects.keys() {
            let Ok(object) = client.object(path.clone()) else {
                continue;
            };
            let (Ok(block), Ok(drive)) = (object.block().await, object.drive().await) else {
                continue;
            };
            let dev_bytes = block.device().await.unwrap_or_default();
            let dev_path = String::from_utf8_lossy(&dev_bytes)
                .trim_end_matches('\u{0}')
                .to_string();
            let model = drive.model().await.unwrap_or_default();
            let size = drive.size().await.unwrap_or_default();
            let removable = drive.removable().await.unwrap_or_default();
            let bus = drive.connection_bus().await.unwrap_or_default();
            if let Some(device) = Self::map_drive(&model, size, removable, &bus, &dev_path) {
                devices.push(device);
            }
        }
        Ok(devices)
    }
}

#[cfg(test)]
mod tests;
```
> Nota: a API exata do crate `udisks2 0.3` pode diferir levemente (nomes de métodos dos proxies `block()`/`drive()`/`object()`). Ajuste conforme `cargo doc -p udisks2` se necessário, mantendo a semântica: iterar objetos, pegar block+drive, ler device/model/size/removable/connection_bus, e filtrar via `map_drive`.

- [ ] **Step 5: `linux/mod.rs` e `lib.rs`**

`crates/infrastructure/src/linux/mod.rs`:
```rust
//! Adapters de IO para Linux.

mod udisks2_disk_service;

pub use udisks2_disk_service::Udisks2DiskService;
```
`crates/infrastructure/src/lib.rs` — adicionar:
```rust
pub mod linux;
```

- [ ] **Step 6: Rodar testes (do mapeamento) e build**

Run: `cargo test -p infrastructure && cargo build -p infrastructure`
Expected: os 2 testes do `map_drive` passam; compila (baixa udisks2/zbus).

- [ ] **Step 7: Commit**

```bash
git add crates/infrastructure
git commit -m "feat(infrastructure): Udisks2DiskService (enumeração real via udisks2/zbus)"
```

---

### Task 4: Ponte tokio→egui (polling + estado compartilhado)

**Files:**
- Modify: `crates/app/src/window.rs`, `crates/app/src/main.rs`

**Interfaces:**
- Consumes: `Udisks2DiskService`, `ListDevices`, `DeviceListState`, `UiState`, `NurApp`.

- [ ] **Step 1: `LiveUiState` com estado compartilhado**

`crates/app/src/window.rs` — substituir `LiveUiState`:
```rust
use application::ports::{DeviceListState, UiState};
use application::use_cases::ListDevices;
use infrastructure::linux::Udisks2DiskService;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Estado da UI alimentado por uma task de polling em background.
pub(crate) struct LiveUiState {
    shared: Arc<RwLock<DeviceListState>>,
}

impl LiveUiState {
    /// Cria o estado (inicia em `Loading`) e spawna a task que faz polling do
    /// udisks2 a cada 1,5s, atualizando o estado e repintando a UI.
    pub(crate) fn spawn(runtime: &tokio::runtime::Handle, ctx: egui::Context) -> Self {
        let shared = Arc::new(RwLock::new(DeviceListState::Loading));
        let writer = Arc::clone(&shared);
        runtime.spawn(async move {
            let uc = ListDevices::new(Arc::new(Udisks2DiskService::new()));
            loop {
                let next = match uc.execute().await {
                    Ok(views) => DeviceListState::Ready(views),
                    Err(e) => DeviceListState::Error(format!("udisks2 indisponível: {e}")),
                };
                if let Ok(mut guard) = writer.write() {
                    *guard = next;
                }
                ctx.request_repaint();
                tokio::time::sleep(Duration::from_secs_f32(1.5)).await;
            }
        });
        Self { shared }
    }
}

impl UiState for LiveUiState {
    fn device_list(&self) -> DeviceListState {
        self.shared
            .read()
            .map(|g| g.clone())
            .unwrap_or(DeviceListState::Loading)
    }
}
```

- [ ] **Step 2: `main.rs` passa o `egui_ctx` para a ponte**

`crates/app/src/main.rs` — montar o `LiveUiState` **dentro** do creator do eframe (onde há `cc.egui_ctx`), usando o handle do runtime:
```rust
    let handle = runtime.handle().clone();
    let _guard = runtime.enter();
    let options = /* NativeOptions como hoje */;
    let result = eframe::run_native(
        "Nur",
        options,
        Box::new(move |cc| {
            let state = window::LiveUiState::spawn(&handle, cc.egui_ctx.clone());
            Ok(Box::new(ui::NurApp::new(std::sync::Arc::new(state))))
        }),
    );
```
Mover a construção das `NativeOptions` para o `main` (ou um helper) e remover a antiga `window::open`/`UiStateAoVivo::build`. O `main` continua tratando erros com `match`/`ExitCode` (sem `unwrap`).

- [ ] **Step 3: Build e rodar (manual, com pendrive real)**

Run: `cargo build --bin nur`
Expected: compila. Validação manual: `cargo run --bin nur` numa máquina com udisks2 — a lista mostra pendrives reais; plugar/remover atualiza em ~2s; sem pendrive → "Nenhum pendrive detectado"; sem udisks2 → "udisks2 indisponível".

- [ ] **Step 4: Verificação headless por screenshot**

Run: `NUR_CAPTURE=/tmp/nur.png cargo run --bin nur` (em ambiente sem udisks2, deve capturar mostrando "udisks2 indisponível" ou "Detectando…", confirmando que não quebra).

- [ ] **Step 5: Commit**

```bash
git add crates/app
git commit -m "feat(app): ponte tokio→egui (polling udisks2 + estado compartilhado)"
```

---

## Definition of Done (Incremento 1)

- [ ] `nur` lista pendrives **reais** removíveis/USB via udisks2 (não os stubs), excluindo o disco de sistema.
- [ ] Plugar/remover atualiza a lista em ≤ ~2s sem reiniciar.
- [ ] udisks2 ausente → "udisks2 indisponível"; nenhum pendrive → "Nenhum pendrive detectado"; durante detecção → "Detectando…".
- [ ] Nada é gravado nem formatado (read-only).
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo fmt --all -- --check`, `cargo xtask check` — todos verdes.

## Fora deste plano (próximos incrementos)

- Gravação raw / formatação (com o spike do ADR 0006).
- `IsoInspector` (isohybrid vs Windows) e seletor de arquivo ISO real (`rfd`).
- Hot-plug por sinais D-Bus (em vez de polling).
- Adapters Windows/macOS.
