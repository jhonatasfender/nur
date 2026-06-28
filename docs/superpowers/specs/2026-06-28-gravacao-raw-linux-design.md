# Fase 4 — Gravação raw da ISO (Linux) — Design

**Data:** 2026-06-28
**Status:** Aprovado
**Escopo:** Implementar a **gravação real** de uma ISO bootável num pendrive no Linux, modo *Boot*, por **cópia raw byte-a-byte** (isohybrid). Inclui seletor de ISO nativo, detecção do tipo de ISO (bloqueando o que não for gravável por raw), abertura privilegiada do device via udisks2/polkit, progresso real, cancelamento e verificação pós-escrita. **Primeiro código destrutivo do projeto.**
**Relacionado:** ADR 0004 (udisks2/polkit), ADR 0005 (gravador inteligente), ADR 0006 (raw write primeiro + spike), `docs/pesquisa/03`. Continua a Fase 3 (enumeração real por sysfs).

---

## 1. Objetivo

A UI (já completa, hoje com progresso **simulado**) passa a **gravar de verdade** uma ISO Linux isohybrid no pendrive selecionado. O usuário escolhe a ISO num diálogo nativo, o app detecta se ela é gravável por raw, confirma com o modal "digite APAGAR" e grava com barra de progresso real, podendo cancelar; ao final, relê e verifica que o conteúdo bate com a ISO.

**Não** entram neste incremento: o modo *Formatar* (particionar + mkfs), o caminho Windows (extração + split do WIM) e os adapters Windows/macOS — todos ficam para incrementos próprios.

## 2. Decisões (travadas no brainstorming)

| Tema | Decisão |
|------|---------|
| Fatiamento | **Um único PR** com seleção + detecção + gravação raw + progresso + cancelamento. |
| Elevação de privilégio | **udisks2 `Block.OpenDevice`** → fd autorizado pelo polkit (prompt por-ação). GUI nunca roda como root (ADR 0004). |
| Verificação | **Sempre**: relê o device e compara com a ISO (caligula faz por padrão). Usa a fase "Verificando" já existente. |
| Operação | **Só o modo Boot** (gravar ISO). O modo Formatar continua como UI sem ação real. |
| Detecção | **Simples**: é isohybrid? Se sim → grava raw; senão → bloqueia com aviso honesto. Sem parsear UDF, sem shell-out. |
| Ponte de progresso | **Espelhar a ponte da Fase 3**: estado num `Arc<RwLock<WriteState>>`, a UI lê pela porta `UiState` + `request_repaint()`. |

## 3. Arquitetura e fluxo

```
[UI] --pick_iso()-->        [AppCommands] --spawn--> rfd → IsoInspector.classify → grava IsoSelection
[UI] <--selected_iso()----  (RwLock)                                              (Isohybrid | Unsupported)
[UI] --start(device)-->     [AppCommands] --spawn--> CreateBootable
[UI] <--write_state()-----  (RwLock)                    ├─ Preparing  (abre device via udisks2/polkit)
[UI] --cancel()-->          (AtomicBool)                ├─ Writing{done,total}   (chunks 4 MiB + cancel)
                                                        ├─ Verifying{done,total} (relê e compara)
                                                        └─ Done | Failed | Cancelled
```

A UI permanece **presenter**: lê estado por `UiState`, dispara ações por uma nova porta `UiCommands`. Toda a parte destrutiva e o IO ficam no `app`/`infrastructure`. A simulação atual (`tick`/`Phase`/`progress` local em `crates/ui/src/app.rs`) é **removida**; a barra passa a refletir `write_state` real.

## 4. Componentes por camada

### domain
- `IsoKind { Isohybrid, Unsupported }` — value object puro (resultado da classificação). Sem campos `pub` (já é enum sem dados).

### application — ports
- `IsoInspector` (`#[async_trait]`): `async fn classify(&self, iso: &Path) -> Result<IsoKind, IsoError>`.
- `BootableWriter` (`#[async_trait]`): `async fn write(&self, req: &WriteRequest, sink: Arc<dyn ProgressSink>, cancel: &CancelFlag) -> Result<(), WriteError>`.
- `ProgressSink`: `fn report(&self, progress: WriteProgress)` — abstrai para onde o progresso vai (RwLock no app; vetor num teste).
- `CancelFlag`: wrapper sobre `Arc<AtomicBool>` com `is_cancelled()`/`cancel()`.
- `WriteRequest { iso_path: PathBuf, device: DevicePath }` (campos privados + getters).
- `WriteProgress { phase: WritePhase, done: u64, total: u64 }` onde `WritePhase { Preparing, Writing, Verifying }`.
- `WriteState { Idle | Preparing | Writing { done, total } | Verifying { done, total } | Done | Failed(String) | Cancelled }` — o que a UI lê.
- `IsoSelection { name: String, size: ByteSize, kind: IsoKind }` (+ `path` guardado; campos privados + getters).
- Estende `UiState`: `fn write_state(&self) -> WriteState` e `fn selected_iso(&self) -> Option<IsoSelection>`.
- `UiCommands` (nova porta, chamada pela UI): `fn pick_iso(&self)`, `fn start(&self, device: DevicePath)`, `fn cancel(&self)`.

### application — use_cases
- `CreateBootable`: recebe `Arc<dyn IsoInspector>` + `Arc<dyn BootableWriter>`. Orquestra: `classify` → se `Unsupported`, retorna `WriteError`/estado `Failed` com mensagem clara; se `Isohybrid`, chama `write` (que internamente faz preparar → gravar → verificar), repassando `ProgressSink`/`CancelFlag`.

### infrastructure — linux
- `IsoFileInspector` impl `IsoInspector`: lê os primeiros 512 B (assinatura `0x55AA` em 510–511 **e** ≥1 entrada de partição não-vazia em `0x1BE`–`0x1FD`) e o setor 16 / offset `0x8001` (`CD001`); classifica `Isohybrid` vs `Unsupported`. Rust puro; roda a leitura em `spawn_blocking`.
- `Udisks2BlockWriter` impl `BootableWriter`:
  - `Block.OpenDevice("rw", {flags: O_EXCL|O_SYNC|O_CLOEXEC})` via **zbus blocking** dentro de `tokio::task::spawn_blocking` → recebe `OwnedFd` → `std::fs::File::from(owned_fd)` (`From<OwnedFd>`, **sem `unsafe`**).
  - Valida `iso.len() <= device.size()` **antes** de abrir; senão `WriteError::DeviceTooSmall`.
  - Copia em chunks de 4 MiB lendo a ISO, reportando `WriteProgress` e checando `cancel` entre chunks; `fsync` ao final.
  - **Verificação**: reabre o device para leitura, relê `len(iso)` bytes e compara contra a ISO em streaming; divergência → `WriteError::VerificationMismatch`.
- `RfdIsoPicker` (na infra; usa `rfd::AsyncFileDialog`) para o seletor nativo, filtrando `*.iso`.

### app — composition root
- `LiveUiState` ganha `Arc<RwLock<WriteState>>` e `Arc<RwLock<Option<IsoSelection>>>` (além do `device_list` da Fase 3) e implementa os novos métodos de `UiState`.
- `AppCommands` impl `UiCommands`: guarda o `tokio::runtime::Handle`, o `egui::Context`, as portas e os `RwLock`/`AtomicBool`. `pick_iso` spawna task (rfd → `classify` → grava `IsoSelection`); `start` spawna task que roda `CreateBootable` escrevendo `WriteState`; `cancel` seta o `AtomicBool`. Cada atualização chama `ctx.request_repaint()`.

### ui
- `NurApp` recebe `Arc<dyn UiCommands>` (além do `Arc<dyn UiState>`).
- Botão "Selecionar ISO" → `commands.pick_iso()`; mostra `selected_iso` (nome/tamanho) e, se `kind == Unsupported`, exibe aviso ("ISO não-isohybrid — gravação raw indisponível; ISOs Windows e afins precisam do modo extração, ainda não disponível") e **desabilita** o botão "Criar".
- Botão "Criar bootável" (somente após o modal "digite APAGAR") → `commands.start(device_path)`.
- Durante a operação, a barra e os rótulos refletem `write_state`; botão **Cancelar** → `commands.cancel()`.
- Remove `tick`, `Phase`, `progress` e `mode`-simulação relacionados (mantém `Mode` para alternar Boot/Format na UI).

## 5. Tratamento de erros e segurança

- `IsoError { Io(String) }` (só leitura/classificação da ISO); `WriteError { Unauthorized, DeviceBusy, DeviceTooSmall, Io(String), VerificationMismatch, Cancelled }`.
- Cancelar deixa o pendrive **parcialmente gravado** → estado `Cancelled` e a UI avisa "pendrive incompleto, regrave".
- Polkit negado → `Unauthorized` → "autorização negada".
- Device em uso (montado/aberto por outro) → `DeviceBusy`.
- Device menor que a ISO → bloqueia **antes** de abrir o fd.
- GUI nunca roda como root; `unsafe_code = forbid` mantido (`File::from(OwnedFd)` é seguro); sem `unwrap`/`expect`/`panic` em produção; código em inglês, comentários/UI em pt-BR; ≤199 linhas/arquivo; zero campos `pub`.

## 6. Testes

- **Puro:** `IsoFileInspector` com setores sintéticos — isohybrid (`0x55AA` + partição + `CD001`) classifica `Isohybrid`; sem assinatura/partição classifica `Unsupported`. Transições de `WriteState`/`WriteProgress`.
- **Lógica de cópia/verify isolada:** a rotina "copiar com progresso + verificar" opera sobre `Read`+`Write`/`Seek` genéricos e é testada com arquivos temporários/`Cursor` (grava num arquivo comum, confere bytes, progresso monotônico, cancelamento no meio) — **separada** da obtenção do fd via udisks, deixando a parte arriscada (zbus) fina.
- **Use case:** `CreateBootable` com fakes de `IsoInspector`/`BootableWriter` — orquestração, bloqueio de `Unsupported`, propagação de cancelamento e de erro.
- **Manual/integração (fora do CI):** gravar uma ISO Linux real num pendrive numa máquina com udisks2 + polkit e **bootar em QEMU** para confirmar a imagem; validar a UI por screenshot em cada fase.

## 7. Critérios de aceite

- [ ] Selecionar uma ISO abre o diálogo nativo e mostra nome/tamanho.
- [ ] ISO isohybrid → botão "Criar" habilitado; ISO não-isohybrid → aviso + botão desabilitado.
- [ ] Gravar pede confirmação polkit, escreve a ISO no device e mostra **progresso real** (não simulado).
- [ ] Cancelar interrompe a escrita e leva ao estado `Cancelled` com aviso de pendrive incompleto.
- [ ] Após gravar, a verificação relê e confirma o conteúdo; divergência vira erro visível.
- [ ] O pendrive gravado **boota em QEMU** (validação manual).
- [ ] Device menor que a ISO, polkit negado e device ocupado produzem mensagens claras, sem quebrar o app.
- [ ] Qualidade mantida: clippy `-D warnings`, testes, `unsafe_code=forbid`, zero campos `pub`, ≤199 linhas/arquivo, fmt, doc, deny, machete.

## 8. Riscos e mitigações

- **zbus + polkit lento/travando** (a dor da Fase 3): aqui é **1 única chamada** (`OpenDevice`), isolada em `spawn_blocking` com zbus **blocking**. Se ainda assim travar, **fallback documentado**: helper privilegiado via `pkexec` (modelo Popsicle/caligula). Decisão registrada caso seja necessária.
- **Verificação enganada por cache**: reabrir o device para a leitura de verificação (não reutilizar o fd de escrita com `seek(0)`), garantindo bytes do meio físico.
- **rfd no loop egui**: usar `AsyncFileDialog` (não a versão síncrona que congela a janela enquanto o diálogo está aberto).

## 9. Fora de escopo

- Modo *Formatar* real (particionar + mkfs FAT32/NTFS/exFAT).
- Caminho Windows (extração + split do `install.wim`) e o spike do ADR 0006.
- Detecção positiva de Windows (parser UDF / `bootmgr`/`install.wim`).
- Adapters Windows/macOS.
- Gravação paralela em múltiplos devices (estilo Popsicle).
