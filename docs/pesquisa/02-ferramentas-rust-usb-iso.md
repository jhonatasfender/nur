# Pesquisa 02 — Ferramentas Rust para USB/ISO (estado da arte)

> Pesquisa web jun/2026. Crates, técnica de gravação, egui atual, privilégios e armadilhas. Fontes primárias (GitHub, crates.io, docs.rs, storaged.org).

## 1. Projetos de referência

### Popsicle (System76, Rust + GTK) — https://github.com/pop-os/popsicle
- Workspace com `cli` e `gtk`; lógica de flash no crate raiz.
- Deps: `usb-disk-probe 0.2.0` (enumera discos USB via sysfs), `mnt 0.3.1` (lê `/proc/mounts`), `srmw 0.1.1` (single-reader-multiple-writer → grava várias USBs em paralelo), `async-std 1.12`, `futures 0.3`, `libc`.
- **Não usa udisks/dbus no core**: abre o device e faz **raw write** (dd paralelo). Verifica SHA256/MD5.
- **Privilégio**: app inteiro elevado via `pkexec` (abordagem grosseira).

### caligula (Rust TUI) — https://github.com/ifd3f/caligula
- `tokio 1.48`, `nix 0.31`, `libc 0.2.177`, `which 6`, `ratatui 0.26.3`, `crossterm 0.27`, `indicatif`, `inquire`.
- Descompressão: `flate2`/`xz2`/`bzip2`/`ruzstd`/`lz4_flex` (gz/xz/bz2/zst).
- Hash: `sha2`/`sha1`/`md-5` — valida antes e **revalida lendo o disco** depois.
- Raw write via `nix`/`libc`. **Sem udisks/dbus**.
- **Privilégio (padrão moderno)**: spawna **processo-filho privilegiado** via `sudo`/`doas`/`su` (`--root ask|always|never`). Só a parte que toca o device roda como root, não a UI.

### Outros
- **balenaEtcher** (Electron): raw write + **validação read-back**. Lição: usuários esperam verificação.
- **Fedora Media Writer** (Qt): grava "estilo dd" **via UDisks2** (não chama `dd`), revalida lendo de volta, oferece **"Restore"** (devolve o pendrive a partição única após o raw write sobrescrever a tabela). **Modelo arquitetural mais próximo do nosso no Linux.**
- **Ventoy**: instala bootloader + você copia ISOs como arquivos (multi-ISO). Não é raw write. Útil só como feature "multi-ISO" futura.
- **Rufus** (Windows, C): extrai arquivos e monta partição/bootloader; para ISOs Windows (`install.wim` > 4 GB) → split p/ FAT32 ou NTFS+UEFI:NTFS; "DD mode" p/ ISOs Linux.

### Conclusão (raw write basta?)
- **ISOs Linux isohybrid: SIM** — copiar bytes da ISO para o **device inteiro** (`/dev/sdX`, não a partição). A isohybrid traz tabela de partição DOS + ESP/EFI. Fontes: syslinux Isohybrid wiki; turnkeylinux iso2usb.
- Cuidados: gravar no disco e não na partição; `fsync` + reler tabela (`BLKRRPART`) / ejetar.
- **Exceção: ISOs Windows** exigem extração (caminho separado). Ver pesquisa 03.

## 2. Crates Rust atuais

### Enumeração de drives removíveis (cross-platform)
- **`rs-drivelist 0.9.4`** (2024-08) — port do drivelist da Balena; **Linux + Windows**; `drive_list() -> Vec<DeviceDescriptor>` com bus type, tamanho, mountpoints, flag removível. **Melhor opção cross-platform pronta.**
- `usb-disk-probe 0.2.0` (só Linux/sysfs, USB). `block-utils` (pop-os, Linux). `sysinfo` (metadados limitados). `disk-types` (pop-os).

### udisks2 / D-Bus
- **`udisks2 0.3.1`** — **recomendado**: bindings via `zbus-xmlgen`, depende de **`zbus ^5.2`**, **runtime-agnostic**. Cobre Block/Drive/Filesystem/Manager/Partition/Encrypted/NVMe. https://docs.rs/udisks2
- `dbus-udisks2` (pop-os) — legado, usa libdbus C, só leitura. Evitar.
- **Prefira `zbus`** (puro Rust, async, sem libdbus) a `dbus` (dbus-rs).

### Particionamento / FS
- **GPT: `gptman 3.1.1`** (puro Rust) / alternativa `gpt`. **MBR: `mbrman 0.6.1`** (puro Rust).
- **FAT32: `fatfs` (rust-fatfs ~0.3.x)** — `format_volume()` + `FormatVolumeOptions`. **Só quick format** (zerar antes p/ full). Opera sobre `ReadWriteSeek`. **Parado desde início de 2023.**
- **exFAT/NTFS/ext4 nativos: imaturos.** `hadris-fat` exFAT WIP; `ntfs` (ColinFinck) **read-only**; sem ext4 mkfs maduro. **→ shell-out `mkfs.fat`/`mkfs.exfat`/`mkfs.ntfs`/`mkfs.ext4`** ou udisks2 `Block.Format`.

### Integridade
- **`sha2`** (SHA-256/512), `sha1`, `md-5` (RustCrypto). Validar antes + read-back depois.

### Windows
- **`windows` crate** (oficial): `CreateFile` em `\\.\PhysicalDriveN` (device) e `\\.\X:` (volume); `FSCTL_LOCK_VOLUME` + `FSCTL_DISMOUNT_VOLUME` antes de gravar; `IOCTL_STORAGE_EJECT_MEDIA` p/ ejetar. Módulo `windows::Win32::System::Ioctl`. Cuidado: alguns drivers USB retornam `ERROR_INVALID_FUNCTION` no lock/dismount — tratar como não-fatal. Enumeração: `SetupDiGetClassDevs`/`IOCTL_STORAGE_QUERY_PROPERTY` ou `rs-drivelist`.

### macOS
- **DiskArbitration**: `DADiskUnmount`/`DADiskMount` (desmontar disco inteiro antes do raw write). Gravar em `/dev/rdiskN` (raw, mais rápido). CLI: `diskutil unmountDisk`. Bindings via `core-foundation`/`objc2` ou shell-out `diskutil`.

## 3. egui / eframe — estado atual (2026)

- **egui/eframe 0.35.0, lançada 2026-06-25.** Edição Rust **2024**. https://github.com/emilk/egui/releases
- **MSRV**: política "≥ 2 releases atrás do Rust mais novo"; faixa recente ~1.88→1.92. Confirmar no `Cargo.toml` da release exata.
- **Wayland/X11 (winit)**: no Linux **habilitar explicitamente** as features `wayland` e/ou `x11` no eframe (sem elas não roda em CI Linux). `wayland` também conserta clipboard.
- **Armadilha**: renderer **wgpu + Wayland** tem bug de janela sem resposta a mouse/teclado (issue #3924). **Usar renderer `glow` no Wayland**. Selecionar via `NativeOptions::renderer`.
- **Tema claro/escuro + persistência**:
  - `ctx.set_visuals(Visuals::dark()/light())` / `set_theme`/`ThemePreference` ("follow system") em versões recentes. Widgets `global_dark_light_mode_switch`/`buttons`.
  - Persistência: feature **`persistence`** do eframe; salvar struct de estado (campo `theme`/`dark_mode`) via **serde** em `cc.storage` — eframe (de)serializa entre execuções.

## 4. Privilégios / elevação

- **Linux (recomendado)**: **não rodar a GUI como root.** udisks2 via polkit: **`Block.OpenDevice()`** com `O_EXCL | O_SYNC | O_CLOEXEC` retorna **fd gravável autorizado** (prompt por-ação). `OpenForRestore`/`OpenForBackup` **deprecados desde udisks 2.7.3** — usar `OpenDevice`. É o que o Fedora Media Writer faz. Polkit actions: storaged.org/doc/udisks2-api/latest/udisks-polkit-actions.html
  - Alternativa (caligula): processo-filho privilegiado via `sudo`/`pkexec`.
- **Windows**: manifesto `requestedExecutionLevel level="requireAdministrator"` (UAC) ou relançar elevado via `ShellExecuteEx "runas"`. Tudo-ou-nada por processo.
- **macOS**: helper privilegiado via `SMJobBless`/Service Management (`AuthorizationExecuteWithPrivileges` deprecado pela Apple). Crate `security-framework`.

## 5. Armadilhas ao escrever em device de bloco (viram requisitos)

1. **Desmontar TODAS as partições antes**; automounters podem remontar — abrir com `O_EXCL` (Linux) / `FSCTL_LOCK_VOLUME` (Windows).
2. **Escrever no device inteiro, não na partição** (`/dev/sdX`, não `/dev/sdX1`).
3. **Cache engana a verificação**: após gravar, `fsync`/`FlushFileBuffers`; ao revalidar, **burlar cache** (`O_DIRECT`, `posix_fadvise(DONTNEED)`, `/dev/rdiskN` no macOS).
4. **Reler a tabela de partição** (`BLKRRPART`) ou ejetar; udisks2 cuida ao fechar o fd.
5. **Disco errado = catástrofe**: filtrar só removível/USB, mostrar modelo+tamanho+caminho, **excluir disco de sistema**, exigir confirmação explícita.
6. **Imagem comprimida**: descomprimir on-the-fly (gz/xz/zst).
7. **Integridade da fonte**: validar SHA-256 antes; oferecer read-back depois.

## Stack acionável (resumo)
- **Linux**: `udisks2 0.3.x` + `zbus 5` → `Block.OpenDevice` (polkit) → raw write com `fsync`; mkfs via udisks `Block.Format` ou shell-out `mkfs.*`. Enumeração `rs-drivelist`.
- **Windows**: crate `windows` (CreateFile/IOCTL lock+dismount+eject), UAC/`runas`.
- **macOS**: DiskArbitration + `/dev/rdiskN`, helper SMJobBless.
- **Compartilhado**: `gptman 3.1.x`/`mbrman 0.6.x`, `fatfs` (FAT32), `sha2`.
- **UI**: eframe 0.35 (edição 2024), features `wayland`+`x11`, renderer `glow`, feature `persistence` + serde p/ tema.
