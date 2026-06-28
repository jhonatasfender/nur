# Abrir o pendrive no gerenciador de arquivos — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Um atalho discreto que, com um pendrive selecionado, abre seu conteúdo no gerenciador de arquivos do SO (montando antes, se preciso).

**Architecture:** Hexagonal. Nova porta `DeviceBrowser` (application), implementada na infra por `Udisks2DeviceBrowser` (acha/monta a partição via udisks2 e chama `xdg-open`). A UI dispara por `UiCommands::open_device`; o `app` spawna a task e publica erro discreto num `Arc<RwLock<Option<String>>>` que a UI lê via `UiState::browse_notice`. A lógica de parsing de `/proc/mounts` é pura e testável.

**Tech Stack:** Rust 2024, egui/eframe 0.35, tokio, async-trait, `zbus 5` (blocking, para `Filesystem.Mount`), `xdg-open` via `std::process::Command`.

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
- Read-only: nada é gravado nem desmontado; a GUI nunca roda como root.

---

### Task 1: erro `BrowseError`

**Files:**
- Modify: `crates/application/src/errors.rs`
- Modify: `crates/application/src/errors/tests.rs`

**Interfaces:**
- Produces: `application::errors::BrowseError` — `pub enum BrowseError { NoFilesystem, Mount(String), Launch(String) }`.

- [ ] **Step 1: Write the failing test**

Append to `crates/application/src/errors/tests.rs`:
```rust
#[test]
fn browse_error_messages_are_in_ptbr() {
    use super::BrowseError;
    assert_eq!(
        BrowseError::NoFilesystem.to_string(),
        "este pendrive não tem uma partição legível para abrir"
    );
    assert_eq!(
        BrowseError::Mount("x".to_owned()).to_string(),
        "não foi possível montar o pendrive: x"
    );
    assert_eq!(
        BrowseError::Launch("y".to_owned()).to_string(),
        "não foi possível abrir o gerenciador: y"
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p application errors`
Expected: FAIL (`BrowseError` not found).

- [ ] **Step 3: Implement**

Append to `crates/application/src/errors.rs` (before the trailing `#[cfg(test)] mod tests;`):
```rust
/// Falhas ao abrir o pendrive no gerenciador de arquivos.
#[derive(Debug, thiserror::Error)]
pub enum BrowseError {
    /// O dispositivo não tem uma partição com filesystem montável.
    #[error("este pendrive não tem uma partição legível para abrir")]
    NoFilesystem,
    /// Falha ao montar a partição.
    #[error("não foi possível montar o pendrive: {0}")]
    Mount(String),
    /// Falha ao lançar o gerenciador de arquivos.
    #[error("não foi possível abrir o gerenciador: {0}")]
    Launch(String),
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p application errors`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/application/src/errors.rs crates/application/src/errors/tests.rs
git commit -m "feat(application): BrowseError"
```

---

### Task 2: porta `DeviceBrowser` + `UiCommands::open_device` + `UiState::browse_notice`

**Files:**
- Create: `crates/application/src/ports/device_browser.rs`
- Modify: `crates/application/src/ports/ui_commands.rs`
- Modify: `crates/application/src/ports/ui_state.rs`
- Modify: `crates/application/src/ports/ui_state/tests.rs`
- Modify: `crates/application/src/ports/mod.rs`

**Interfaces:**
- Consumes: `domain::DevicePath`, `application::errors::BrowseError`.
- Produces:
  - `DeviceBrowser` (`#[async_trait]`): `async fn open(&self, device: &DevicePath) -> Result<(), BrowseError>`.
  - `UiCommands` gains `fn open_device(&self, device: DevicePath)`.
  - `UiState` gains default `fn browse_notice(&self) -> Option<String> { None }`.

- [ ] **Step 1: Write the failing test**

Add to `crates/application/src/ports/ui_state/tests.rs` (inside the existing module; extend the `Minimal` impl test):
```rust
#[test]
fn ui_state_default_browse_notice_is_none() {
    struct M;
    impl UiState for M {
        fn device_list(&self) -> DeviceListState {
            DeviceListState::Loading
        }
    }
    assert!(M.browse_notice().is_none());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p application ui_state`
Expected: FAIL (`browse_notice` not found).

- [ ] **Step 3: Implement**

`crates/application/src/ports/device_browser.rs`:
```rust
//! Porta para abrir o conteúdo de um dispositivo no gerenciador de arquivos.

use crate::errors::BrowseError;
use domain::DevicePath;

/// Abre o pendrive no gerenciador de arquivos do SO (montando se preciso).
#[async_trait::async_trait]
pub trait DeviceBrowser: Send + Sync {
    /// Abre o conteúdo do `device` no gerenciador nativo.
    ///
    /// # Errors
    /// Retorna [`BrowseError`] se não houver filesystem, ou se montar/abrir falhar.
    async fn open(&self, device: &DevicePath) -> Result<(), BrowseError>;
}
```

In `crates/application/src/ports/ui_commands.rs`, add the method to the trait:
```rust
    /// Abre o pendrive no gerenciador de arquivos do SO.
    fn open_device(&self, device: DevicePath);
```

In `crates/application/src/ports/ui_state.rs`, add the defaulted method to the `UiState` trait (after `selected_iso`):
```rust
    /// Mensagem discreta de falha ao abrir o pendrive (padrão: nenhuma).
    fn browse_notice(&self) -> Option<String> {
        None
    }
```

In `crates/application/src/ports/mod.rs`, register and export:
```rust
mod device_browser;
pub use device_browser::DeviceBrowser;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p application` then `cargo clippy -p application --all-targets -- -D warnings`
Expected: PASS / no warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/application/src/ports/
git commit -m "feat(application): porta DeviceBrowser + open_device + browse_notice"
```

---

### Task 3: parser puro de `/proc/mounts`

**Files:**
- Create: `crates/infrastructure/src/linux/mount_table.rs`
- Create: `crates/infrastructure/src/linux/mount_table/tests.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Produces: `infrastructure::linux::MountTable` with `fn mount_point_for(contents: &str, name: &str) -> Option<String>` — given `/proc/mounts` contents and a device base name (e.g. `sdb`), returns the first mount point whose source device is `/dev/<name>` or `/dev/<name>` followed by a partition digit (`sdb1`…). Must NOT match a different device whose name merely starts with `name` (e.g. `sdbb`).

**Detecção:** cada linha de `/proc/mounts` é `DEVICE MOUNTPOINT FSTYPE OPTS …` separada por espaços; o mount point pode conter espaços codificados como `\040` (octal). Uma source `/dev/X` corresponde a `name` se `X == name` ou `X == name + <dígitos>` (partição). Decodificar `\040` → espaço no mount point.

- [ ] **Step 1: Write the failing test**

`crates/infrastructure/src/linux/mount_table/tests.rs`:
```rust
use super::MountTable;

const SAMPLE: &str = "\
/dev/sda2 / ext4 rw,relatime 0 0
/dev/sdb1 /run/media/user/USB\\040STICK vfat rw 0 0
tmpfs /tmp tmpfs rw 0 0
";

#[test]
fn finds_partition_mount_point_decoding_spaces() {
    assert_eq!(
        MountTable::mount_point_for(SAMPLE, "sdb"),
        Some("/run/media/user/USB STICK".to_owned())
    );
}

#[test]
fn matches_whole_device_when_no_partition() {
    let table = "/dev/sdc /mnt/raw ext4 rw 0 0\n";
    assert_eq!(
        MountTable::mount_point_for(table, "sdc"),
        Some("/mnt/raw".to_owned())
    );
}

#[test]
fn does_not_match_prefix_collision() {
    let table = "/dev/sdbb1 /mnt/other vfat rw 0 0\n";
    assert_eq!(MountTable::mount_point_for(table, "sdb"), None);
}

#[test]
fn returns_none_when_absent() {
    assert_eq!(MountTable::mount_point_for(SAMPLE, "sdz"), None);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p infrastructure mount_table`
Expected: FAIL (type missing).

- [ ] **Step 3: Implement**

`crates/infrastructure/src/linux/mount_table.rs`:
```rust
//! Parser puro de `/proc/mounts` para achar o ponto de montagem de um device.

/// Localiza pontos de montagem de partições no `/proc/mounts`.
pub struct MountTable;

impl MountTable {
    /// Primeiro mount point cujo device é `/dev/<name>` ou `/dev/<name><dígitos>`.
    #[must_use]
    pub fn mount_point_for(contents: &str, name: &str) -> Option<String> {
        contents.lines().find_map(|line| {
            let mut fields = line.split(' ');
            let source = fields.next()?;
            let mount = fields.next()?;
            let dev = source.strip_prefix("/dev/")?;
            if Self::matches(dev, name) {
                Some(Self::decode(mount))
            } else {
                None
            }
        })
    }

    // `dev` corresponde a `name` (o próprio device ou uma partição `name<dígitos>`).
    fn matches(dev: &str, name: &str) -> bool {
        match dev.strip_prefix(name) {
            Some("") => true,
            Some(rest) => rest.bytes().all(|b| b.is_ascii_digit()),
            None => false,
        }
    }

    // Decodifica escapes octais do `/proc/mounts` (ex.: `\040` → espaço).
    fn decode(raw: &str) -> String {
        raw.replace("\\040", " ")
            .replace("\\011", "\t")
            .replace("\\134", "\\")
    }
}

#[cfg(test)]
mod tests;
```

Add to `crates/infrastructure/src/linux/mod.rs`:
```rust
mod mount_table;
pub use mount_table::MountTable;
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p infrastructure mount_table`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/infrastructure/src/linux/mount_table.rs crates/infrastructure/src/linux/mount_table/tests.rs crates/infrastructure/src/linux/mod.rs
git commit -m "feat(infrastructure): MountTable (parser de /proc/mounts)"
```

---

### Task 4: `Udisks2DeviceBrowser` (montar via udisks2 + `xdg-open`)

**Files:**
- Create: `crates/infrastructure/src/linux/udisks2_device_browser.rs`
- Modify: `crates/infrastructure/src/linux/mod.rs`

**Interfaces:**
- Consumes: `DeviceBrowser`, `BrowseError`, `DevicePath`, `MountTable`.
- Produces: `infrastructure::linux::Udisks2DeviceBrowser` with `new() -> Self` + `Default`; impl `DeviceBrowser`.

**Notas (casca fina — validação manual, sem teste unitário):**
- `name` = `device.as_str().trim_start_matches("/dev/")`.
- **Já montado?** lê `/proc/mounts` (`std::fs::read_to_string`) e usa `MountTable::mount_point_for`. Se `Some(path)`, pula para o `xdg-open`.
- **Senão, montar:** enumere as partições em `/sys/block/<name>/` cujo nome começa com `<name>` (entradas `sdb1`, `sdb2`…). Para cada uma, tente `Filesystem.Mount` via zbus blocking no objeto `/org/freedesktop/UDisks2/block_devices/<part>`, interface `org.freedesktop.UDisks2.Filesystem`, método `Mount`, args `(HashMap::<&str, zbus::zvariant::Value>::new(),)` (options vazio). O reply traz o mount path `s`: `reply.body().deserialize::<String>()`. Primeiro sucesso → use esse path. Se nenhuma partição existir/montar → `BrowseError::NoFilesystem` (ou `Mount(msg)` se o erro do D-Bus não for "sem filesystem").
- **Abrir:** `std::process::Command::new("xdg-open").arg(&mount_point).spawn()`; erro → `BrowseError::Launch(e.to_string())`. Não aguardar o processo (`spawn`, não `status`).
- Toda a parte bloqueante (sysfs + zbus + spawn) roda em `tokio::task::spawn_blocking`; a `async fn open` apenas aguarda o join e mapeia `JoinError` → `BrowseError::Launch`.
- Mantenha o arquivo ≤199 linhas (helpers privados `mounted_path`, `mount_first_partition`, `launch` como associated fns).

- [ ] **Step 1: Implement**

Escreva `crates/infrastructure/src/linux/udisks2_device_browser.rs` conforme as notas. Registre em `crates/infrastructure/src/linux/mod.rs`:
```rust
mod udisks2_device_browser;
pub use udisks2_device_browser::Udisks2DeviceBrowser;
```

- [ ] **Step 2: Build + lints**

Run: `cargo build -p infrastructure`
Run: `cargo clippy -p infrastructure --all-targets -- -D warnings`
Run: `cargo xtask check`
Expected: builds; no warnings; arquivo ≤199 linhas.

- [ ] **Step 3: Manual validation (registre no PR, fora do CI)**

Plugar um pendrive (FAT/ext4), e numa app de teste ou após o wiring (Task 5): selecionar e clicar no link → o gerenciador abre a pasta. Testar pendrive cru (sem FS) → `NoFilesystem`. **Sem teste automatizado nesta task** (precisa udisks + sessão gráfica); a lógica testável está no `MountTable` (Task 3).

- [ ] **Step 4: Commit**

```bash
git add crates/infrastructure/src/linux/udisks2_device_browser.rs crates/infrastructure/src/linux/mod.rs
git commit -m "feat(infrastructure): Udisks2DeviceBrowser (monta + xdg-open)"
```

---

### Task 5: wiring (`app`) + link na UI

**Files:**
- Modify: `crates/app/src/commands.rs`
- Modify: `crates/app/src/window.rs`
- Modify: `crates/ui/src/app/draw.rs`
- Modify: `crates/ui/src/app/tests.rs`

**Interfaces:**
- Consumes: `Udisks2DeviceBrowser`, `DeviceBrowser`, `UiCommands::open_device`, `UiState::browse_notice`, `DevicePath`.
- Produces:
  - `LiveUiState` gains `notice: Arc<RwLock<Option<String>>>` and overrides `browse_notice()`.
  - `AppCommands` gains `notice: Arc<RwLock<Option<String>>>` and implements `open_device` (spawns task running `Udisks2DeviceBrowser::open`; on error stores the message in `notice`, on success clears it; `request_repaint`).
  - The device selector renders a discreet link + the notice line.

- [ ] **Step 1: `LiveUiState` carries the notice**

In `crates/app/src/window.rs`:
- Add field `notice: Arc<RwLock<Option<String>>>` to `LiveUiState` and a constructor param; override:
```rust
    fn browse_notice(&self) -> Option<String> {
        self.notice.read().ok().and_then(|g| g.clone())
    }
```
- In the creator closure, create `let notice = Arc::new(RwLock::new(None));`, pass `Arc::clone(&notice)` to both `LiveUiState::new(...)` and `AppCommands::new(...)`.

- [ ] **Step 2: `AppCommands::open_device`**

In `crates/app/src/commands.rs`:
- Add field `notice: Arc<RwLock<Option<String>>>` and the constructor param.
- Add imports: `application::ports::DeviceBrowser`, `infrastructure::linux::Udisks2DeviceBrowser`.
- Implement:
```rust
    fn open_device(&self, device: DevicePath) {
        let ctx = self.ctx.clone();
        let notice = Arc::clone(&self.notice);
        self.runtime.spawn(async move {
            let result = Udisks2DeviceBrowser::new().open(&device).await;
            if let Ok(mut guard) = notice.write() {
                *guard = result.err().map(|e| e.to_string());
            }
            ctx.request_repaint();
        });
    }
```

- [ ] **Step 3: The UI link**

In `crates/ui/src/app/draw.rs`, inside `device_selector`, where `self.selected.is_some()` already shows the red warning, add below it a discreet clickable link and the notice. After the existing warning label block:
```rust
        if let Some(device) = self.selected.and_then(|i| devices.get(i)) {
            let path = device.path().to_owned();
            ui.add_space(4.0);
            let link = ui.add(
                egui::Label::new(
                    egui::RichText::new("\u{1F4C2} Abrir para conferir o conteúdo")
                        .color(palette.accent())
                        .size(12.0),
                )
                .sense(egui::Sense::click()),
            );
            if link.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if link.clicked() {
                self.commands.open_device(domain::DevicePath::new(path));
            }
            if let Some(msg) = self.state.browse_notice() {
                ui.add_space(2.0);
                ui.label(egui::RichText::new(msg).color(palette.destructive()).size(11.0));
            }
        }
```
(Note: `device_selector` borrows `self.selected`/`self.state`; compute `path` before calling `self.commands` to avoid borrow conflicts. `domain` must be a dependency of `ui` — it already is from Fase 4.)

- [ ] **Step 4: Build, test, lints, screenshot**

Run: `cargo build --bin nur`
Run: `cargo test --workspace`
Run: `cargo clippy --workspace --all-targets -- -D warnings`
Run: `cargo xtask check`
Capture the selector with a device selected and inspect that the link renders:
```bash
NUR_CAPTURE=/tmp/nur-browse.png NUR_DEMO=ready NUR_THEME=light \
LIBGL_ALWAYS_SOFTWARE=1 WINIT_UNIX_BACKEND=x11 \
timeout 60 xvfb-run -a -s "-screen 0 900x1000x24" ./target/debug/nur
```
Read `/tmp/nur-browse.png` and confirm the "Abrir para conferir o conteúdo" link appears under the device warning.

- [ ] **Step 5: Update UI test fakes if needed**

The `CommandsFake` in `crates/ui/src/app/tests.rs` must implement the new `open_device`:
```rust
    fn open_device(&self, _device: DevicePath) {}
```
Run: `cargo test -p ui`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/app/src/ crates/ui/src/
git commit -m "feat: link 'abrir pendrive no gerenciador' (wiring + UI)"
```

---

## Self-Review (preenchido pelo autor do plano)

**Spec coverage:**
- Link discreto no selector → Task 5. Handoff ao gerenciador (`xdg-open`) → Task 4. Montar se preciso (udisks2 `Filesystem.Mount`) → Task 4. Achar partição montada (`/proc/mounts`) → Task 3 (testável). Porta `DeviceBrowser` + `UiCommands::open_device` + `UiState::browse_notice` → Task 2. `BrowseError` → Task 1. Erro discreto na UI → Task 5. Read-only / sem root → Task 4 (apenas Mount/`xdg-open`). Linux-agora, porta pronta p/ outros SOs → Task 2/4. Critérios de qualidade → cada task.
- **Sem gaps.**

**Type consistency:** `DeviceBrowser::open(&DevicePath)` (Task 2) usado na Task 4/5; `UiCommands::open_device(DevicePath)` (Task 2) implementado na Task 5 e no fake; `UiState::browse_notice() -> Option<String>` (Task 2) sobrescrito na Task 5; `MountTable::mount_point_for(&str, &str) -> Option<String>` (Task 3) usado na Task 4; `BrowseError` (Task 1) usado em 2/4.

**Notas ao executor:**
- A casca `Udisks2DeviceBrowser` (Task 4) não tem teste unitário — a lógica testável é o `MountTable`. A conversão exata do reply de `Mount` (`String`) e a API zbus seguem o padrão já usado no `Udisks2BlockWriter` (Fase 4); ajuste à versão instalada se preciso.
- Cuidado com o borrow em `device_selector` (Task 5): extraia `path: String` do device antes de chamar `self.commands.open_device(...)`.
