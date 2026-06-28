# Fase 6 — Modo Formatar real (Linux) — Design

**Data:** 2026-06-28
**Status:** Aprovado
**Escopo:** Ligar o modo *Formatar* (hoje UI inerte) a uma formatação **real** no Linux: criar tabela de partição (GPT/MBR), uma partição cobrindo o disco e formatar com o filesystem + rótulo escolhidos, via udisks2. Operação destrutiva, mas curta.
**Relacionado:** ADR 0004 (udisks2/polkit), ADR 0005/0006. Continua a Fase 4 (gravação) e Fase 5 (abrir no gerenciador).

---

## 1. Objetivo

No modo *Formatar*, com pendrive selecionado e opções escolhidas (Esquema de partição, Sistema de arquivos, Rótulo, Formatação rápida), o botão "Iniciar" abre o modal "digite APAGAR"; confirmando, o app cria a tabela de partição, **uma** partição cobrindo o disco e a formata com o filesystem + rótulo escolhidos. A barra mostra "Formatando…" e depois "Concluído". O seletor "Sistema alvo" **some** no modo Formatar (é conceito de boot, sem efeito ao formatar).

**Não** entram: tornar o pendrive bootável no Formatar, múltiplas partições, criptografia, verificação pós-formatação.

## 2. Decisões (do brainstorming)

| Tema | Decisão |
|------|---------|
| Resultado | **Tabela de partição (GPT/MBR) + 1 partição** cobrindo o disco + mkfs do FS + rótulo. |
| Sistema alvo | **Ocultar** no modo Formatar (conceito de boot, sem efeito ao formatar). |
| Mecanismo | **udisks2** (ADR 0004): `Block.Format` (tabela) → `PartitionTable.CreatePartition` → format da partição. |
| Formatação rápida | Ligada → `mkfs` direto; desligada → `erase: "zero"` (lento) antes. |
| Progresso | Reusa `WriteState` (Preparing→"Formatando…"→Done/Failed); **indeterminado** (mkfs não reporta progresso granular). |
| Cancelamento | **Não** (operação curta — YAGNI). |
| Confirmação | Mesmo modal "digite APAGAR" já existente. |

## 3. Arquitetura e fluxo

```
[UI modal confirma, mode==Format] --format(device, options)--> [AppCommands] --spawn--> FormatDevice
[UI] <--write_state()------------- (RwLock<WriteState>)             ├─ Preparing ("Formatando…")
                                                                    ├─ Block.Format(device, gpt|dos)
                                                                    ├─ PartitionTable.CreatePartition (cobre o disco)
                                                                    ├─ format da partição (vfat|ntfs|exfat|ext4 + label)
                                                                    └─ Done | Failed(msg)
```

### domain
- `PartitionScheme { Gpt, Mbr }` — value object.
- `FilesystemKind { Fat32, Ntfs, ExFat, Ext4 }` — value object.
- Reusa `VolumeLabel` (já existe, com validação) para o rótulo.

### application — ports & errors
- `FormatOptions { scheme: PartitionScheme, filesystem: FilesystemKind, label: VolumeLabel, quick: bool }` (campos privados + getters + ctor `new`).
- `FormatError { Unauthorized, DeviceBusy, ToolMissing(String), Backend(String) }` (mensagens pt-BR).
- `DeviceFormatter` (`#[async_trait]`): `async fn format(&self, device: &DevicePath, options: &FormatOptions, sink: Arc<dyn ProgressSink>) -> Result<(), FormatError>`.
- `UiCommands` ganha `fn format(&self, device: DevicePath, options: FormatOptions)`.

### application — use_cases
- `FormatDevice`: recebe `Arc<dyn DeviceFormatter>`; `async fn execute(&self, device: DevicePath, options: FormatOptions, sink: Arc<dyn ProgressSink>) -> Result<(), FormatError>` — reporta `Preparing` e delega ao formatter.

### infrastructure — linux
- `Udisks2Formatter` impl `DeviceFormatter`:
  - **Mapeamento puro/testável** (`fn udisks_table(scheme) -> &str` → `gpt`/`dos`; `fn udisks_fs(fs) -> &str` → `vfat`/`ntfs`/`exfat`/`ext4`).
  - Fluxo (zbus blocking em `spawn_blocking`): `Block.Format(<table>, {})` no device → relê para achar a partição criada → `PartitionTable.CreatePartition(0, 0, "", "", {})` (offset 0, size 0 = resto) → no objeto da partição, `Block.Format(<fs>, {"label": label, ["erase": "zero"]})`.
  - `quick == false` → inclui `erase: "zero"`.
  - Erros do D-Bus → `Unauthorized`/`DeviceBusy`/`Backend`; mkfs ausente (mensagem do udisks tipo "not found"/"Failed to execute") → `ToolMissing`.

### app
- `AppCommands::format` spawna a task que roda `FormatDevice::execute`, publicando `WriteState` no lock compartilhado (igual à gravação); `Done`/`Failed`.

### ui
- `modal.rs`: no confirm, `if mode == Format { commands.format(device, options) } else { commands.start(device) }`, montando `FormatOptions` dos índices/label da UI.
- `options.rs`: `iso_section` já é só-Boot; o "Sistema alvo" passa a aparecer **apenas no modo Boot**; no Format mostra Esquema de partição, Sistema de arquivos, Rótulo, Formatação rápida.
- `status.rs`: textos sensíveis ao modo — Boot: "Gravando imagem…"/"Pendrive bootável pronto!"; Format: "Formatando…"/"Formatação concluída!".

## 4. Tratamento de erros

- `ToolMissing(fs)` → "instale as ferramentas para formatar em {fs}".
- `Unauthorized` → "autorização negada"; `DeviceBusy` → "dispositivo ocupado"; `Backend(msg)` → mensagem do udisks.
- A task nunca derruba o app; erro vira `WriteState::Failed(msg)` na barra.
- GUI nunca como root; rótulo validado por `VolumeLabel`.

## 5. Testes

- **Puro:** `Udisks2Formatter::udisks_table`/`udisks_fs` (cada variante → string correta); construção de `FormatOptions` e validação de rótulo por `VolumeLabel` (rótulo inválido recusado).
- **Use case:** `FormatDevice` com fake de `DeviceFormatter` — orquestração e propagação de erro.
- **Manual/loopback (sem hardware):** criar um arquivo de imagem, associá-lo a `/dev/loopN` (`losetup`), formatar pelo app e conferir a tabela/partição/FS com `lsblk`/`blkid`. Também testar FS sem `mkfs` instalado → `ToolMissing`.

## 6. Critérios de aceite

- [ ] No modo Formatar, "Sistema alvo" não aparece; as demais opções, sim.
- [ ] Confirmar formata o pendrive: tabela escolhida + 1 partição + filesystem + rótulo.
- [ ] FS sem ferramenta instalada → mensagem clara; polkit negado/ocupado → mensagens claras.
- [ ] Barra mostra "Formatando…" e depois "Formatação concluída!".
- [ ] Nada roda como root; o modo Boot (gravação) continua intacto.
- [ ] Qualidade: clippy `-D warnings`, testes, `unsafe_code=forbid`, zero campos `pub`, ≤199 linhas/arquivo, fmt, doc, deny, machete.

## 7. Fora de escopo

- Tornar o pendrive bootável no Formatar (marcar partição ativa, criar ESP).
- Múltiplas partições, criptografia (LUKS), verificação pós-formatação.
- Cancelamento (operação curta).
- Adapters Windows/macOS do `DeviceFormatter` (a porta fica pronta).
