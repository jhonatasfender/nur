# Fase 5 — Abrir o pendrive no gerenciador de arquivos — Design

**Data:** 2026-06-28
**Status:** Aprovado
**Escopo:** Um atalho discreto que, ao selecionar um pendrive, deixa o usuário **abrir o conteúdo no gerenciador de arquivos do SO** (Files/Nautilus/Dolphin no Linux; Explorer no Windows depois) para conferir o que há antes de apagar. **100% read-only.** É um handoff puro ao SO — não construímos navegador de arquivos dentro do app.
**Relacionado:** ADR 0003 (cross-platform faseado), ADR 0004 (udisks2/polkit). Continua a Fase 4 (gravação).

---

## 1. Objetivo

Ao selecionar um pendrive, aparece abaixo do dispositivo um link discreto — **"📂 Abrir para conferir o conteúdo"** — perto do aviso vermelho "Todos os dados deste dispositivo serão apagados". Clicar pede ao SO para montar a partição (se ainda não estiver) e abrir a pasta no gerenciador nativo, dando ao usuário a chance de navegar e ter certeza antes de gravar/apagar.

## 2. Decisões (do brainstorming)

| Tema | Decisão |
|------|---------|
| Natureza | Handoff ao gerenciador do SO; nada de browser embutido. |
| Não montado | **Pedir ao SO para montar e abrir** — via udisks2 `Filesystem.Mount` (mesmo mecanismo do desktop ao plugar; GUI não roda como root). |
| Posição | Link discreto no `device_selector`, só quando há pendrive selecionado. |
| Erros | Best-effort: falha vira uma linha discreta; nunca quebra; nunca apaga. |
| Plataforma | Linux agora; a porta deixa Windows/macOS para depois (ADR 0003). |

## 3. Arquitetura e componentes

```
[UI] --open_device(device)--> [AppCommands] --spawn--> DeviceBrowser.open(device)
[UI] <--browse_notice()------ (RwLock<Option<String>>)     ├─ acha partição montada (/proc/mounts) OU
                                                           ├─ monta a 1ª partição com FS (udisks2 Filesystem.Mount)
                                                           └─ xdg-open <mountpoint>
```

### application — ports
- `DeviceBrowser` (`#[async_trait]`): `async fn open(&self, device: &DevicePath) -> Result<(), BrowseError>`.
- `UiCommands` ganha `fn open_device(&self, device: DevicePath)`.
- `UiState` ganha (default) `fn browse_notice(&self) -> Option<String> { None }` — mensagem discreta de erro do "abrir" (best-effort), lida pela UI.

### application — errors
- `BrowseError { NoFilesystem, Mount(String), Launch(String) }` (mensagens em pt-BR).

### infrastructure — linux
- `Udisks2DeviceBrowser` impl `DeviceBrowser`:
  1. **Já montado?** lê `/proc/mounts`; se alguma partição de `/dev/sdX` (o próprio device ou `sdX1`, `sdX2`…) estiver montada, usa esse ponto de montagem.
  2. **Senão, montar:** enumera as partições em `/sys/block/<name>/<name>*`; para a 1ª, chama udisks2 `org.freedesktop.UDisks2.Filesystem.Mount(a{sv})` no objeto `/org/freedesktop/UDisks2/block_devices/<part>` (zbus **blocking** em `spawn_blocking`), que devolve o caminho de montagem `s`.
  3. **Abrir:** `std::process::Command::new("xdg-open").arg(mount_point)` (sem `unsafe`).
  - Sem partição/filesystem montável → `BrowseError::NoFilesystem`.
  - A função de **parsing de `/proc/mounts`** (dado o conteúdo e o `name` do device, retorna o 1º mount point de uma partição correspondente) é **pura e testável**.

### app — composition root
- `AppCommands::open_device` spawna a task tokio que roda `DeviceBrowser::open`; em erro, grava `browse_notice` (`Arc<RwLock<Option<String>>>`, compartilhado com `LiveUiState`) e `request_repaint()`. Em sucesso, limpa o notice.

### ui
- No `device_selector` (`crates/ui/src/app/draw.rs`), com pendrive selecionado, renderiza o link discreto que chama `self.commands.open_device(DevicePath::new(path))`; abaixo, se `browse_notice()` for `Some(msg)`, mostra a mensagem discreta.

## 4. Tratamento de erros

- `NoFilesystem` → "este pendrive não tem uma partição legível para abrir".
- `Mount(msg)`/`Launch(msg)` → "não foi possível abrir o pendrive".
- A task nunca derruba o app; o link continua clicável para nova tentativa. Operação read-only (nenhuma escrita).

## 5. Testes

- **Puro:** o parser de `/proc/mounts` — dado um conteúdo sintético e o nome do device, encontra o mount point correto da partição (e devolve `None` quando nenhuma corresponde). Casos: device em si montado (`/dev/sdb`), partição montada (`/dev/sdb1`), nenhuma correspondência, prefixo enganoso (`/dev/sdbb1` **não** corresponde a `sdb`).
- **Casca fina:** `Udisks2DeviceBrowser` (udisks + `xdg-open`) sem teste unitário — validação manual.
- **Manual:** plugar um pendrive com FAT/ext4, selecionar no app, clicar no link → o gerenciador abre a pasta; testar também com pendrive cru (sem FS) → mensagem discreta.

## 6. Critérios de aceite

- [ ] Com pendrive selecionado, aparece o link discreto "Abrir para conferir o conteúdo".
- [ ] Clicar abre a pasta do pendrive no gerenciador nativo (montando antes, se preciso).
- [ ] Pendrive sem filesystem legível → mensagem discreta, sem quebrar.
- [ ] Nada é montado como root; nenhuma escrita ocorre.
- [ ] Qualidade: clippy `-D warnings`, testes, `unsafe_code=forbid`, zero campos `pub`, ≤199 linhas/arquivo, fmt, doc, deny, machete.

## 7. Fora de escopo

- Navegador de arquivos embutido / pré-visualização dentro do app.
- Escolher qual partição abrir quando há várias (abre a 1ª montável).
- Desmontar/ejetar pelo app.
- Adapters Windows/macOS do `DeviceBrowser` (a porta fica pronta; o `xdg-open` vira `explorer`/`open` depois).
