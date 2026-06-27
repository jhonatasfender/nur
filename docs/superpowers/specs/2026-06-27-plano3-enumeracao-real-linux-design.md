# Plano 3 — Incremento 1: Enumeração real de pendrives (Linux) — Design

**Data:** 2026-06-27
**Status:** Aprovado
**Escopo:** Substituir o `DiskServiceStub` por enumeração **real** de pendrives no Linux (udisks2), com atualização ao vivo. **Read-only, zero risco.** Gravação/formatação destrutiva fica para o próximo incremento.
**Relacionado:** ADR 0004 (udisks2/polkit), ADR 0006 (ordem: enumerar antes de gravar), `docs/pesquisa/02`.

---

## 1. Objetivo

A UI (já completa, sobre stubs) passa a mostrar os **pendrives reais** conectados, com a lista se atualizando sozinha quando um dispositivo é plugado/removido. Nada é gravado nem formatado neste incremento — é o primeiro passo que troca o backend falso por um real e **valida o encanamento mais arriscado**: udisks2 + a ponte **tokio→egui**.

## 2. Decisões

| Tema | Decisão |
|------|---------|
| Mecanismo | **udisks2 via D-Bus (`zbus`)** — o mesmo que usaremos para gravar (polkit). |
| Porta | `DiskService` vira **assíncrona** (`async-trait`). |
| Atualização ao vivo | **Polling** a cada ~1,5s numa task tokio (mais simples que assinar sinais D-Bus; valida a ponte igualmente). |
| Estado compartilhado | `Arc<RwLock<DeviceListState>>` (enum `Loading \| Ready(Vec<DeviceView>) \| Error(String)`) preenchido pela task, lido pela UI. |
| Segurança | Filtrar **removível/USB** e **excluir o disco de sistema**. |
| Fallback | `DiskServiceStub` permanece para testes/preview. |

## 3. Arquitetura e componentes

```
ui (UiState::device_list) ──lê──►  Arc<RwLock<DeviceListState>>
                                       ▲ escreve
app (composition root) ── spawn ──►  task tokio (poll a cada 1,5s)
                                       │ chama
application::ListDevices (async) ───►  DiskService (async, porta)
                                       ▲ implementa
infrastructure::Udisks2DiskService (zbus + udisks2)
```

> `DeviceListState` (em `application`) = `Loading | Ready(Vec<DeviceView>) | Error(String)`.
> A porta `UiState` ganha `fn device_list(&self) -> DeviceListState` (mantém `devices()` como conveniência derivada, ou é substituído por `device_list()`). A UI renderiza: `Loading` → "Detectando…", `Ready([])` → "Nenhum pendrive detectado", `Ready([..])` → combo, `Error(m)` → mensagem `m`.

1. **`application` — porta async**
   - `DiskService` passa a `#[async_trait] pub trait DiskService: Send + Sync { async fn list_devices(&self) -> Result<Vec<Device>, DiskError>; }`.
   - `ListDevices::execute(&self) -> Result<Vec<DeviceView>, DiskError>` vira `async`.
   - Nova dependência: `async-trait`.
   - `DiskError` ganha variante para falha de backend (ex.: `Backend(String)`) além de `Unavailable`.

2. **`infrastructure` — `Udisks2DiskService`**
   - Novo módulo (ex.: `src/linux/udisks2_disk_service.rs`), feature/cfg `target_os = "linux"`.
   - Usa `udisks2` + `zbus 5`: obtém os objetos `Block`/`Drive` via o `ObjectManager` do udisks2; para cada bloco, lê propriedades (modelo do drive, tamanho, se é removível, bus de conexão `usb`, e se é o device "raiz" ou tem filhos de sistema).
   - **Filtros:** mantém apenas dispositivos **removíveis e/ou conectados por USB**, e **exclui o disco que contém o sistema de arquivos raiz** (ex.: ignorando o drive cujo bloco está montado em `/` ou via `HintSystem`).
   - Mapeia para `domain::Device::new(DevicePath, model, ByteSize, removable)`.
   - O `DiskServiceStub` existente passa a implementar a porta **async** também (trivial: `async fn` que devolve os 2 fixos).

3. **`app` — ponte tokio→egui**
   - `LiveUiState` passa a guardar `Arc<RwLock<DeviceListState>>` e a implementar `UiState::device_list()` lendo (clonando) esse estado.
   - No composition root: cria o `Udisks2DiskService`, o caso de uso `ListDevices`, o estado compartilhado (inicia em `Loading`), e **spawna uma task tokio** que num loop: dorme 1,5s → `execute().await` → escreve `Ready(...)` ou `Error(...)` no `RwLock` (só quando muda) e chama `egui_ctx.request_repaint()`.
   - O `egui_ctx` é obtido no `eframe::run_native` (do `CreationContext`), então a montagem do app passa por dentro do creator closure.
   - Em caso de erro do backend, grava um estado de erro (ver §4) e segue tentando.

4. **Erros e UX**
   - udisks2 indisponível (daemon ausente) → a UI mostra **"udisks2 indisponível"** no lugar da lista, sem quebrar.
   - Lista vazia → o seletor mostra **"Nenhum pendrive detectado"**.
   - Isso é exatamente o que o enum `DeviceListState` (`Loading | Ready(Vec<DeviceView>) | Error(String)`) carrega. A UI (`device_selector`) renderiza conforme cada variante.

5. **Privilégios:** enumeração é read-only → **sem polkit**.

## 4. Tratamento de erros

- `Udisks2DiskService` mapeia erros de zbus/D-Bus para `DiskError::Backend(msg)` ou `DiskError::Unavailable` (quando o serviço não responde).
- A task de polling nunca derruba o app: captura o erro, atualiza o estado para `Error(msg)`, espera e tenta de novo.
- Sem-pânico mantido (sem `unwrap`/`expect` em produção).

## 5. Testes

- **Unit:** a função de **mapeamento** de propriedades udisks2 → `Device` (dado um conjunto de propriedades simuladas, produz o `Device` correto; filtros de removível/sistema). Isolar o mapeamento numa função pura testável, separada das chamadas zbus.
- **Stub:** continua cobrindo os testes de UI/casos de uso (agora async).
- **Manual/integração:** rodar `nur` numa máquina com udisks2 e um pendrive real, validar por screenshot (a UI lista o dispositivo de verdade; plugar/remover atualiza).

## 6. Critérios de aceite

- [ ] `nur` lista os pendrives **reais** removíveis/USB (não os stubs), excluindo o disco de sistema.
- [ ] Plugar/remover um pendrive atualiza a lista em ≤ ~2s (polling), sem reiniciar.
- [ ] udisks2 ausente → mensagem "udisks2 indisponível"; nenhum pendrive → "Nenhum pendrive detectado".
- [ ] Nada é gravado nem formatado; operação 100% read-only.
- [ ] Qualidade mantida: clippy `-D warnings`, testes, `unsafe_code=forbid` (zbus é Rust seguro), zero campos `pub`, ≤199 linhas/arquivo, código em inglês.

## 7. Fora de escopo

- Gravação raw / formatação (próximo incremento, com o spike do ADR 0006).
- `IsoInspector` (detecção isohybrid vs Windows).
- Hot-plug por **sinais** D-Bus (fica como refinamento; polling basta agora).
- Adapters de Windows/macOS.
- Seletor de arquivo ISO real (`rfd`).
