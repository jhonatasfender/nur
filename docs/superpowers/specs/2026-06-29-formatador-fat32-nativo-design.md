# Incremento A — Formatador FAT32 nativo (Rust puro) — Design

**Data:** 2026-06-29
**Status:** Aprovado
**Escopo:** Reescrever o modo *Formatar* em **Rust nativo** — particionar (GPT/MBR) e criar **FAT32** sem nenhuma ferramenta externa (`mkfs`/`Block.Format`). A UI do *Formatar* passa a oferecer **somente FAT32**. Privilégio continua via udisks2/polkit (apenas o fd; udisks2 é universal, não é "instalar algo").
**Relacionado:** ADR 0010 (núcleo Rust-nativo), revisa ADR 0004/0005. Substitui o `Udisks2Formatter` da Fase 6.

---

## 1. Objetivo

Hoje o modo *Formatar* delega ao udisks `Block.Format` (que chama `mkfs.*` do sistema) e oferece FAT32/NTFS/exFAT/ext4. Como só **FAT32** tem criação madura em Rust (`fatfs`) e o produto deve ser **zero-instalação**, reescrevemos a formatação 100% em Rust: abrir o device com privilégio (udisks `OpenDevice` → fd) e, sobre esse fd, escrever a **tabela de partição** (`gpt`/`mbrman`) e o **FAT32** (`fatfs`). A UI deixa de oferecer os outros filesystems.

**Não** entra: NTFS/exFAT/ext4 (sem criação em Rust), caminho Windows (UDF/WIM — ADR 0010), verificação pós-formatação.

## 2. Decisões (do brainstorming + ADR 0010)

| Tema | Decisão |
|------|---------|
| Filesystem | **Só FAT32** (`fatfs`). UI sem seletor de filesystem. |
| Partição | GPT (`gpt`) ou MBR (`mbrman`), 1 partição cobrindo o disco, alinhada a 1 MiB. Seletor GPT/MBR mantido. |
| Privilégio | udisks2 `Block.OpenDevice` → fd (mesma técnica da Fase 4). Sem `Block.Format`. |
| Escrita | GPT/MBR no início do device; FAT32 **no offset da partição** via um wrapper de IO que recorta `[início, fim]` da partição sobre o fd. |
| Quick vs completa | Rápida = só tabela + FAT32. Completa = **zera a região da partição** antes (lento). |
| Rótulo | `domain::VolumeLabel` (≤11 chars), aplicado pelo `fatfs`. |
| Limpeza | Remove `FilesystemKind`, o campo `filesystem` de `FormatOptions`, `FormatError::ToolMissing` e o `Udisks2Formatter`. |

## 3. Arquitetura e fluxo

```
[UI Formatar confirma] --format(device, options)--> [AppCommands] --spawn--> FormatDevice
[UI] <--write_state()--------------- (RwLock<WriteState>)   └─ NativeFatFormatter.format:
                                                               1. OpenDevice (udisks/polkit) → fd → File
                                                               2. zera a região (se !quick)
                                                               3. escreve GPT/MBR (1 partição, 1 MiB align)
                                                               4. fatfs::format_volume no offset da partição (+ label)
                                                               5. fsync
```

### domain
- **Remove** `FilesystemKind` (e o re-export). Mantém `PartitionScheme { Gpt, Mbr }` e `VolumeLabel`.

### application
- `FormatOptions { scheme: PartitionScheme, label: VolumeLabel, quick: bool }` — **sem** `filesystem`.
- `FormatError { Unauthorized, DeviceBusy, Io(String), Backend(String) }` — **sem** `ToolMissing`.
- Porta `DeviceFormatter::format(&self, device: &DevicePath, options: &FormatOptions) -> Result<(), FormatError>` (inalterada na forma).
- Caso de uso `FormatDevice` inalterado (reporta `Preparing` e delega).

### infrastructure (linux)
- **Remove** `udisks2_formatter.rs`. Cria `native_fat_formatter.rs` implementando `DeviceFormatter`:
  - `open_device(name) -> File` (reusa o padrão do `Udisks2BlockWriter`: `OpenDevice("rw", {flags})` via zbus blocking → `OwnedFd` → `File`, sem `unsafe`).
  - `device_size(name) -> u64` (sysfs, já existe no projeto).
  - **Geometria pura/testável** (`fn layout(device_bytes) -> PartitionLayout { start, len }`): alinhamento 1 MiB, partição cobrindo o resto.
  - Escrita da tabela: GPT via `gpt` / MBR via `mbrman` (tipo FAT32 `0x0C`), 1 partição em `start..start+len`.
  - **`OffsetVolume`** (wrapper): `Read + Write + Seek` que mapeia posições sobre uma janela `[start, start+len)` do fd — entregue ao `fatfs::format_volume(..., FormatVolumeOptions::new().fat_type(Fat32).volume_label(label_bytes))`.
  - Quick=false → zera `[start, start+len)` (ou o início + áreas-chave) antes.
  - `fsync` ao final. Mapeia erros D-Bus → `Unauthorized`/`DeviceBusy`/`Backend`.
  - Tudo bloqueante roda em `spawn_blocking`.

### app
- `AppCommands::format` instancia `NativeFatFormatter` em vez de `Udisks2Formatter` (1 linha). Monta `FormatOptions` sem `filesystem`.

### ui
- `options.rs`: no modo Formatar, **remove o seletor "Sistema de arquivos"** (mostra rótulo fixo "FAT32" ou nada); mantém "Esquema de partição", "Rótulo do volume", "Formatação rápida". No modo Boot, layout atual permanece.
- `modal.rs`: `dispatch` no Format monta `FormatOptions::new(scheme, label, quick)` (sem `filesystem_kind()`); remove o helper `filesystem_kind`.

## 4. Tratamento de erros

- `Unauthorized` (polkit negou), `DeviceBusy` (montado/em uso), `Backend(msg)` (D-Bus), `Io(msg)` (escrita/`fatfs`).
- Device menor que o mínimo de FAT32 → `Io`/`Backend` com mensagem clara.
- Nunca roda como root; nunca quebra a UI (vira `WriteState::Failed(msg)`).

## 5. Testes (grande parte em Rust puro, sem device)

- **Geometria:** `layout(bytes)` — alinhamento a 1 MiB, partição = resto; casos de device pequeno.
- **FAT32 real em memória:** `fatfs::format_volume` sobre um `Cursor<Vec<u8>>` de ~64 MiB com um rótulo; **reabrir** com `fatfs::FileSystem` e asserir `fat_type() == Fat32` e `volume_label()` == rótulo. (Valida a criação de verdade, sem hardware.)
- **`OffsetVolume`:** escrita/leitura/seek confinadas à janela (com `Cursor`), inclusive que não vaza além de `len`.
- **GPT:** escrever num `Cursor` e reler com `gpt` → 1 partição no range esperado. (MBR análogo com `mbrman`.)
- **Casca fina** (`open_device` via udisks): sem teste unitário — validação manual por **loopback** (`losetup`), que dá pra rodar sem pendrive físico.

## 6. Critérios de aceite

- [ ] Modo Formatar não mostra mais "Sistema de arquivos"; mostra GPT/MBR, Rótulo, Formatação rápida.
- [ ] Formatar cria GPT/MBR + 1 partição FAT32 com o rótulo, **sem** `mkfs`/`Block.Format` e sem deps externas.
- [ ] Validação por loopback: `lsblk -f`/`blkid` mostram a partição FAT32 com o rótulo; `fatfs` reabre a imagem.
- [ ] polkit negado/dispositivo ocupado → mensagens claras; nada roda como root.
- [ ] `FilesystemKind`, campo `filesystem`, `FormatError::ToolMissing` e `Udisks2Formatter` removidos; `cargo machete`/clippy limpos.
- [ ] Qualidade: clippy `-D warnings`, testes, `unsafe_code=forbid`, zero campos `pub`, ≤199 linhas/arquivo, fmt, doc, deny, **`cargo xtask check`** (line-limit/pub-fields/bin-visibility).

## 7. Fora de escopo

- NTFS/exFAT/ext4 (sem criação em Rust) — readicionar só com decisão consciente (ADR 0010).
- Caminho Windows (UDF + WIM).
- Verificação pós-formatação; múltiplas partições; criptografia.
