# Design da Interface — Nur (نور) · Formatador de Pendrive & Criador de Boot

**Data:** 2026-06-26
**Status:** Aprovado (interface) + arquitetura técnica definida
**Escopo:** Layout/UX da tela principal **e** decisões de stack/arquitetura/lints. A implementação dos adapters de disco por SO é faseada (ver "Arquitetura técnica").
**Projeto de referência:** `/home/jonatas/projects/github/solana` — herdamos arquitetura, lints e abordagem de UI dele.
**Documentos relacionados:** `docs/README.md` (índice + estado atual) · `docs/pesquisa/` (relatórios completos) · `docs/decisoes/` (ADRs).

---

## 1. Visão geral

Aplicativo desktop para **formatar pendrives** e **criar pendrives bootáveis a partir de uma imagem ISO**. A operação é destrutiva, então a interface prioriza **clareza, praticidade e prevenção de perda acidental de dados**. Idioma: **pt-BR**.

Referência de produto: estilo Rufus (painel único), mas com visual mais limpo e moderno.

## 2. Decisões tomadas

| Tema | Decisão |
|------|---------|
| **Modelo de fluxo** | Painel único — tudo numa tela só (sem wizard). |
| **Layout base** | Variação compacta, 1 coluna, janela estreita (`max-w-md`). |
| **Tema claro/escuro** | Obrigatório. Toggle no header, **persistido** (localStorage no protótipo; storage do eframe + serde no app), respeita a preferência do sistema na primeira abertura. |
| **Paleta** | Neutra (preto/branco/cinzas) como base; vermelho só para alerta destrutivo; verde só para sucesso. |
| **Proteção contra acidente** | Modal de confirmação que exige **digitar `APAGAR`** antes de executar. |
| **Animações** | Aprovadas conforme protótipo; respeitam `prefers-reduced-motion`. |

## 3. Estrutura da tela (de cima para baixo)

1. **Header** — ícone do app, título "Formatador de Pendrive", subtítulo, e **toggle de tema** (sol/lua).
2. **Dispositivo** — `select` com pendrives detectados (modelo, tamanho, `/dev/...`, rótulo). Ao selecionar, exibe **aviso vermelho** de que os dados serão apagados.
3. **Modo** — seletor segmentado com **pílula deslizante**: "Criar bootável" / "Apenas formatar". No modo "Apenas formatar", o bloco de ISO some.
4. **Imagem ISO** — área de **arrastar-e-soltar** (drag & drop) com fallback de clique para selecionar arquivo. Aceita `.iso` / `.img`. Só visível no modo bootável.
5. **Opções de formato** (grid 2 colunas):
   - Esquema de partição: **GPT / MBR**
   - Sistema alvo: **UEFI / BIOS / BIOS ou UEFI**
   - Sistema de arquivos: **FAT32 / NTFS / exFAT / ext4**
   - Rótulo do volume (texto)
   - Checkbox **Formatação rápida**
6. **Status** — rótulo + percentual + **barra de progresso** com fases.
7. **Footer** — botão secundário (Fechar) + botão primário **Iniciar** (desabilitado até haver dispositivo selecionado).

## 4. Fluxo de interação e estados

1. **Inicial:** "Iniciar" desabilitado; status "Pronto / Selecione um dispositivo".
2. **Dispositivo selecionado:** aviso destrutivo aparece; "Iniciar" habilita e **pulsa** para chamar atenção.
3. **Modo bootável:** exige ISO; modo formatar dispensa.
4. **Iniciar →** abre **modal de confirmação** (fundo com blur). Botão de confirmar só habilita ao digitar `APAGAR`.
5. **Execução (fases da barra):**
   - **Preparando** — barra indeterminada.
   - **Gravando / Formatando** — barra determinada com shimmer e percentual.
   - **Verificando** — etapa curta pós-gravação.
   - **Concluído** — barra verde, check animado, brilho de sucesso na janela.

## 5. Animações (aprovadas)

- Entrada da janela (fade + subida) e dos campos em cascata escalonada.
- Pílula do seletor de modo deslizando com efeito de mola.
- Aviso de dispositivo abrindo com altura animada.
- Botão "Iniciar" pulsando quando habilitado.
- Drop da ISO: ícone vira check verde "desenhado" com pop.
- Modal: entrada tipo mola + blur gradual do fundo.
- Barra: indeterminada → shimmer → verde com check.
- Toggle de tema: ícone gira no hover.
- **Acessibilidade:** tudo desativado sob `prefers-reduced-motion: reduce`.

## 6. Protótipos de referência

- `superdesign/design_iterations/desktop_app_for_form_1.html` — base compacta escolhida (claro/escuro).
- `superdesign/design_iterations/desktop_app_for_form_1_1.html` — **versão final aprovada**, com todas as animações.
- `superdesign/design_iterations/desktop_app_for_form_2.html` — variação espaçosa (2 colunas), descartada como base.

## 7. Arquitetura técnica

Herdada do projeto de referência `solana`, com desvios conscientes (seção 9).

**Stack:**
- **Linguagem:** Rust **edição 2024**, stable recente (egui 0.35 exige edição 2024; MSRV ~1.88+).
- **UI:** `egui` + `eframe` **0.35** nativo. Features `wayland` + `x11`; **renderer `glow`** (evita bug de input do wgpu no Wayland, issue emilk/egui#3924). Persistência via feature `persistence` + `serde` (salva tema entre execuções).
- **Async:** `tokio` em background; ponte para a UI imediata via `request_repaint_after` e `runtime.handle()` (mesmo padrão do `solana`, ideal para o progresso de gravação).
- **Erros:** `thiserror` nas libs/núcleo, `anyhow` só no composition root.

**Workspace hexagonal (crates):**
- `domain` — núcleo puro: modelos de `Dispositivo`, `Imagem/ISO`, `OpçãoDeFormato`, value objects (tamanho, rótulo). Sem IO.
- `application` — casos de uso (`ListarDispositivos`, `Formatar`, `GravarImagem`, `VerificarIntegridade`) e **portas (traits)**, ex.: `DiskService`, `DeviceWatcher`, `ImageReader`, `PrivilegeElevator`, **`BootableWriter`** (com `IsoInspector` + estratégias `RawWriteStrategy`/`WindowsExtractStrategy`).
- `infrastructure` — adapters concretos de IO por SO (ver abaixo).
- `app` — composition root + binário(s); injeta adapters reais ou stubs.
- `ui` — presenter egui; fala só com `domain`+`application` via `Arc<dyn Trait>` + builder (stub para preview/screenshots, real em produção). **Nunca** depende de `infrastructure`.
- `tools/xtask` — fora do workspace de produção (limite de linhas etc.).

**Backend de disco — porta `DiskService` com adapters por SO (cross-platform via abstração):**

| SO | Adapter | Observação |
|----|---------|-----------|
| **Linux** | udisks2 via D-Bus (`zbus`) | Rust seguro, autorização via polkit (sem rodar como root). **Implementado primeiro.** |
| **Windows** | APIs Win32 (`windows` crate) / DiskPart | Pode exigir `unsafe`/FFI isolado ou shell-out. Fase posterior. |
| **macOS** | DiskArbitration / `diskutil` | Shell-out seguro possível. Fase posterior. |

**Estratégia de gravação — Gravador inteligente (auto-detecção):** um único botão "Criar bootável"; o `IsoInspector` lê os primeiros setores + a árvore de arquivos da ISO e seleciona a estratégia: **raw write** (isohybrid/Linux) ou **extração Windows** (FAT32 + split do `install.wim`). Não existe método raw único para os dois — ver ADR 0005 e seção 8. Descartado o modelo "estilo Ventoy" (ADR 0005).

**Faseamento de implementação (ver ADR 0006):**
0. **Spike descartável (cedo, em paralelo):** validar manualmente a receita Windows (FAT32 + split do `install.wim` via `wimlib` + bootar instalador Windows em VM) — retira o risco antes de implementar.
1. **Fase 1:** arquitetura cross-platform-ready + **adapter Linux (udisks2)** + **raw write** + `IsoInspector` já detectando Windows (avisa "ainda não suportado" em vez de gravar errado) + UI completa rodando de verdade no Linux (validável bootando ISO Linux em QEMU).
2. **Fase 2:** estratégia `WindowsExtractStrategy` atrás do mesmo botão.
3. **Fase 3:** adapter Windows (APIs Win32) e adapter macOS (DiskArbitration).

**Técnica de gravação:** raw write da ISO isohybrid no device inteiro; caminho Windows por extração. Detalhes, marcadores de detecção e armadilhas na seção 8.

## 8. Pesquisa de ferramentas (consolidada)

Pesquisa web de jun/2026 sobre projetos atuais e crates mantidos.

**Referências estudadas:**
- **Fedora Media Writer** (Qt) — **modelo arquitetural mais próximo no Linux**: grava "estilo dd" abrindo o device **via UDisks2** (não chama `dd`), revalida lendo de volta, e oferece "Restore" para devolver o pendrive a uma partição única.
- **Popsicle** (System76, Rust/GTK) — raw write paralelo; usa `usb-disk-probe`, `mnt`, `srmw`; eleva o app todo via `pkexec` (abordagem grosseira, evitar).
- **caligula** (Rust TUI) — raw write via `nix`/`libc`; **só o processo que toca o device roda elevado** (`sudo`/`doas`); descomprime gz/xz/zst on-the-fly; valida SHA antes e read-back depois.
- **Rufus** — para **ISOs do Windows** faz extração para FAT32/NTFS (não raw write). balenaEtcher — raw write + validação read-back automática.

**Técnica de gravação (decisão):**
- **ISOs Linux modernas (isohybrid):** basta **raw write da imagem no device inteiro** (`/dev/sdX`, não a partição) — a isohybrid já traz tabela de partição + ESP/EFI. Esse é o caminho principal.
- **ISOs do Windows:** exigem extração estilo Rufus (`install.wim` > 4 GB → split/NTFS). **Tratado como caminho separado e fase posterior** (decidir se entra no escopo).
- Após gravar: `fsync` → reler tabela de partição (`BLKRRPART` / ejetar). udisks faz isso ao fechar o fd.

**Crates recomendados:**

| Necessidade | Crate | Notas |
|-------------|-------|-------|
| Enumerar removíveis (cross-platform) | `rs-drivelist 0.9.x` | Linux + Windows; bus type, tamanho, flag removível |
| udisks2 (Linux) | `udisks2 0.3.x` + `zbus 5` | runtime-agnostic; cobre Block/Drive/Filesystem/Partition |
| Tabela GPT / MBR | `gptman 3.1.x` / `mbrman 0.6.x` | puro Rust |
| FAT32 nativo | `fatfs` (rust-fatfs 0.3.x) | `format_volume()`; só quick format (zerar antes p/ full) |
| exFAT / NTFS / ext4 | **shell-out `mkfs.*`** ou udisks `Block.Format` | libs nativas imaturas (ntfs é read-only; exfat WIP) |
| Checksum / read-back | `sha2`, `sha1`, `md-5` | RustCrypto |
| Descompressão on-the-fly | `flate2`/`xz2`/`ruzstd` | gz/xz/zst (feature futura) |

**Elevação de privilégios (porta `PrivilegeElevator`):**
- **Linux (recomendado):** **não rodar a GUI como root.** udisks2 `Block.OpenDevice()` com `O_EXCL | O_SYNC | O_CLOEXEC` retorna um **fd gravável autorizado pelo polkit** (prompt por-ação). (`OpenForRestore`/`OpenForBackup` estão deprecados desde udisks 2.7.3 — usar `OpenDevice`.) → no Linux o "elevator" é essencialmente no-op (o udisks resolve).
- **Windows:** manifesto UAC `requireAdministrator` ou relançar elevado via `ShellExecuteEx "runas"` (tudo-ou-nada por processo).
- **macOS:** helper privilegiado via `SMJobBless`/Service Management (`AuthorizationExecuteWithPrivileges` está deprecado).

**Armadilhas a tratar na implementação (viram requisitos):**
1. **Desmontar todas as partições antes** de gravar; abrir com `O_EXCL`/`FSCTL_LOCK_VOLUME` para impedir remontagem por automounter.
2. **Gravar no device inteiro, nunca na partição.**
3. **`fsync`/`FlushFileBuffers`** após gravar; na revalidação **burlar o cache de página** (`O_DIRECT`/`posix_fadvise`/`/dev/rdiskN`) — senão lê-se o cache e "valida" dado não persistido.
4. **Reler a tabela de partição** (`BLKRRPART`) após o raw write.
5. **Proteção contra disco errado:** filtrar só removível/USB, **excluir o disco de sistema**, mostrar modelo+tamanho+caminho, exigir confirmação explícita (já temos o "digite APAGAR" na UI).

**Stack acionável (resumo):** Linux = `udisks2`+`zbus` (`Block.OpenDevice` polkit) + raw write com `fsync` + mkfs via udisks/`mkfs.*`; enumeração `rs-drivelist`; tabela `gptman`/`mbrman`; FAT32 `fatfs`; checksum `sha2`. UI = eframe 0.35 (edição 2024), features `wayland`+`x11`, renderer `glow`, persistência+serde.

## 9. Desvios conscientes em relação ao `solana`

- **Tema claro/escuro** (o `solana` é dark-only): estender o `ThemeKit` para **duas paletas + toggle**, persistido, respeitando `prefers-color-scheme`.
- **Cross-platform** (o `solana` é Linux/X11-only): arquitetura preparada para 3 SOs; egui modernizado com Wayland.
- **`unsafe_code`**: manter `forbid` no workspace; onde FFI for inevitável (Windows/macOS), isolar `unsafe` no crate de infra daquele SO ou usar shell-out seguro.
- **MSRV/egui**: modernizar em vez de mirar rustc 1.84.1 / egui 0.29.

## 10. Padrões de qualidade herdados (aplicar verbatim)

- `[workspace.lints]`: `clippy::all=deny`, `pedantic=warn`; `unwrap_used`/`expect_used`/`panic=deny`; `unsafe_code=forbid`; `missing_docs=deny`, `unreachable_pub=deny` e demais lints do compilador como erro.
- `clippy.toml` liberando `unwrap`/`expect`/`panic` **só em testes**.
- `deny.toml` (advisories/licenças/sources) e CI com jobs paralelos: fmt, clippy (`-D warnings`), test, deny, doc, machete, spell, line-limit.
- **Estilo OOP estrito**: sem função livre exceto `main`; campos privados + getters; Value Objects no lugar de primitivos.
- **Limite de ~199 linhas por arquivo `.rs`** (xtask).
- **Testes em arquivo irmão** (`foo.rs` → `foo/tests.rs`).
- Comentários em **pt-BR** explicando o "porquê".

## 11. Itens em aberto / candidatos a feature

- Detecção automática de pendrives (hot-plug) e atualização da lista em tempo real (`DeviceWatcher`).
- Verificação de integridade da ISO (checksum/SHA256) **antes** e **validação read-back depois** da gravação (padrão Etcher/caligula).
- Descompressão on-the-fly de imagens `.gz`/`.xz`/`.zst` (não exigir descompactar antes).
- **Suporte a ISOs do Windows** (extração estilo Rufus) — **decidido: entra via Opção A faseada** (Fase 2, ver ADR 0005/0006). Caminho de código separado do raw write, fortemente baseado em shell-out (`7z`/loop-mount p/ UDF, `mkfs.ntfs`, `wimlib`/DISM p/ split do WIM).
- Cancelar operação em andamento; ejetar com segurança ao concluir.
- "Restore" do pendrive para partição única após raw write (como Fedora Media Writer).

## 12. Critérios de aceite da interface

- [ ] Painel único renderiza em claro e escuro, com persistência da escolha.
- [ ] Selecionar dispositivo habilita "Iniciar" e mostra o aviso destrutivo.
- [ ] Alternar modo mostra/esconde o bloco de ISO.
- [ ] "Iniciar" abre modal que só confirma após digitar `APAGAR`.
- [ ] Barra de progresso percorre as fases e termina em estado de sucesso.
- [ ] Animações respeitam `prefers-reduced-motion`.
