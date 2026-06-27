# Pesquisa 03 — Gravação de ISO: Linux vs Windows (técnicas reais)

> Pesquisa web jun/2026. Como criar pendrive bootável que funcione para ISOs Linux **e** Windows, e o que os projetos reais fazem.

## Resumo executivo

**Não existe método único real de "1 ISO → 1 pendrive" que torne qualquer ISO (Windows E Linux) bootável sem detecção/ramificação na gravação.** Os formatos pedem operações opostas: ISOs Linux modernas (isohybrid) querem **cópia raw byte-a-byte (dd)**; ISOs Windows querem **particionar e copiar arquivos** (ESP FAT32 + NTFS/WIM split). São mutuamente exclusivos na escrita. O único que parece "um fluxo para qualquer ISO" é o **modelo Ventoy**, que move a ramificação do tempo-de-gravação para o tempo-de-boot. Praticamente todo projeto sério ramifica (Rufus detecta e troca de modo; mantenedores do Popsicle/Rust projetaram "detectar-e-trocar-de-modo").

## 1. Detecção do tipo de ISO

### Marcadores concretos (offsets)
| Marcador | Offset | Significado |
|---|---|---|
| `CD001` | 0x8001 (setor 16, PVD) | É ISO9660 |
| Boot Record + `EL TORITO SPECIFICATION` | 0x8800 (setor 17) | Bootável opticamente. **Não** torna dd-gravável |
| Tabela de partição MBR embutida | 0x1BE–0x1FD | Isohybrid: 1ª partição cobre a imagem |
| Assinatura MBR `0x55 0xAA` | 0x1FE–0x1FF | Com ≥1 partição não-vazia = **isohybrid, dd-safe** |
| MBR protetor `0xEE` / `EFI PART` | tipo em 0x1BE; magic em 0x200 | Isohybrid-GPT, dd-gravável |

**Regra prática:** leia 512 bytes; se `0x55AA` em 510–511 **E** ≥1 entrada de partição não-vazia em 0x1BE–0x1FD → isohybrid (dd-writable). Combine com `CD001` em 0x8001. Use o teste "0x55AA + ≥1 partição", **não** o LBA exato (varia entre builds).

**Detectar Windows** procurando na árvore ISO9660/**UDF**: `bootmgr`, `bootmgr.efi`, `sources/install.wim`/`install.esd`/`install.swm` (+ flag se ≥ 4 GiB), `sources/boot.wim`, `EFI/BOOT/BOOTX64.EFI`. São os flags do Rufus `check_iso_props()` (`has_bootmgr`, `wininst_index`, `has_4GB_file`). Fonte: github.com/pbatard/rufus/blob/master/src/iso.c

### Como cada projeto decide
- **Rufus** — único que inspeciona e troca raw-dd ↔ extração. Escaneia UDF→ISO9660; em isohybrid oferece "ISO Image mode" vs "DD Image mode"; auto-splita `install.wim` ≥ 4 GiB.
- **Fedora Media Writer / balenaEtcher / GNOME Disks** — **raw only**.
- **WoeUSB** — **só extração** (Windows); usa `wimlib-imagex split` p/ FAT32.
- **Ventoy** — ignora o tipo; boota o arquivo em runtime.
- **Popsicle / caligula (Rust)** — **raw only**.

### Libs/crates para parsear
- Rust: `iso9660` (incompleto), `iso9660_simple`, `cdfs` (ISO9660, **sem UDF**), `mbr`/`mbr-nostd`, `gpt`, `gptman`, `gpt-disk-types` (Google, no_std). El Torito sem crate → ler 0x8800 manualmente (trivial).
- C: **libcdio** (Rufus linka), **libisofs/libisoburn** (`xorriso -report_system_area` = forma mais autoritativa), **libmagic** (`file` imprime `(DOS/MBR boot sector)` p/ isohybrid).

## 2. Caminho da ISO Windows (extração) — detalhe

### Esquema de partição
| Boot | Esquema | Por quê |
|---|---|---|
| UEFI puro | **GPT + ESP FAT32** (GUID `C12A7328-...`, `EF00`) | UEFI boota `\EFI\BOOT\BOOTX64.EFI` |
| BIOS legado | **MBR**, partição ativa, tipo `0x0C`/`0x0B` | BIOS roda `bootmgr` |
| Ambos | **MBR + FAT32** (ativa) | Combo máx. compatibilidade (Media Creation Tool) |

### FAT32 vs NTFS (UEFI)
- Spec UEFI exige **só driver FAT** no firmware (NTFS nativo raro) → daí o UEFI:NTFS.
- **Limite 4 GB do FAT32**: tamanho é 32-bit → máx. ~4 GiB. `install.wim` do Win10/11 passa disso (4,4–6+ GB) e **nem pode ser copiado** p/ FAT32 (EFBIG). Por isso ISOs Windows são **UDF**.

### 3 soluções p/ install.wim > 4 GB

**(a) Split do WIM em `.swm` (mantém FAT32, sem dep GPL em runtime — RECOMENDADA)**
```bash
wimsplit /mnt/iso/sources/install.wim /mnt/usb/sources/install.swm 4000   # MiB
# ou: wimlib-imagex split ...
```
```bat
Dism /Split-Image /ImageFile:C:\sources\install.wim /SWMFile:D:\sources\install.swm /FileSize:4000
```
Gera `install.swm`, `install2.swm`...; colocar em `\sources\` e **deletar o `install.wim`**. O Windows Setup aceita `.wim` OU `.swm` e rejunta sozinho. Fontes: wimlib.net/man1/wimsplit.html ; learn.microsoft.com/.../split-windowsimage

**(b) NTFS + UEFI:NTFS (truque do Rufus)**
- Bootloader UEFI genérico que boota de NTFS/exFAT sem o firmware ler NTFS. Mantém o `install.wim` inteiro.
- Autor: pbatard. Fonte: github.com/pbatard/uefi-ntfs (vendorizado em `rufus/res/uefi/uefi-ntfs.img`). **Licença GPLv2** (não v3).
- Chainload: 2 partições — (1) NTFS grande com arquivos Windows; (2) FAT minúscula no fim com `uefi-ntfs.img`. Firmware boota a FAT → UEFI:NTFS carrega driver NTFS → acha a NTFS → chainloada o Windows Boot Manager.
- Driver NTFS read-only derivado do **ntfs-3g (GPLv2)**, **assinado pela MS** (Secure-Boot-safe). Driver exFAT (EfiFs) é **GPLv3, não assinado**.
- **Embutível** (Rufus embute), mas obriga conformidade GPLv2. Rota (a) evita dep GPL em runtime.

**(c) exFAT + chainloader** — sem limite 4 GB, mas firmware não lê exFAT; driver exFAT é **GPLv3 e não Secure-Boot-assinado** → não boota com Secure Boot. Preferir NTFS.

### Tornar bootável (BIOS + UEFI)
- **UEFI:** precisa `\EFI\BOOT\BOOTX64.EFI` (= `bootmgfw.efi` de `\efi\microsoft\boot\`). ISOs modernas já trazem; se faltar, copiar.
- **BIOS:** precisa `bootmgr` + `\boot\BCD` na raiz (presentes na ISO); escrever boot sector que carrega `bootmgr` (ex.: `ms-sys -7 /dev/sdX`) e marcar partição **ativa**.

### Libs/crates
- **Ler UDF**: crates Rust são **só ISO9660** → não leem UDF (ISOs Windows). **Sem crate UDF maduro.** Fallback: **7-Zip** (`7z x`, lê ISO9660+UDF) ou loop-mount.
- **Escrever FAT32**: `fatfs` (parado 2023). **NTFS**: `ntfs` read-only → `mkntfs`/`ntfs-3g`. **Split WIM**: **wimlib** (C) — **sem binding Rust maduro** → shell-out `wimlib-imagex`/DISM.

## 3. Ventoy (universal)

### Mecanismo
GRUB2 + 2 partições (dados exFAT + `VTOYEFI` ~32 MB). No boot, escaneia/lista/boota a imagem. **Truque:** não faz loop-mount; computa as **faixas de setores físicos (chunks)** da ISO e passa o mapa ao OS, que reconstrói um **device de bloco virtual** apontando p/ os setores raw.
- **Linux:** device-mapper (`dm_mod`) via `vtoydm`; hooks no initramfs; binário **`dm-patch`** que **taint o kernel** e quebra entre versões.
- **Windows:** injeta `vtoyjump32/64.exe` no WinPE que **remonta a ISO como disco real** (`OpenVirtualDisk`/`AttachVirtualDisk` no Win8+, ou ImDisk no Win7) e patcheia BCD/WIM.
- **Windows e Linux NÃO são tratados igual.** Config via `ventoy.json`. Fonte: deepwiki.com/ventoy/Ventoy/4.2-windows-boot-process

### Licença / embutibilidade
- **GPLv3+**. ventoy.net/en/doc_license.html
- **Controvérsia dos BLOBs (2021–2025):** muitos **binários pré-compilados de procedência incerta**, build não-reproduzível, possível violação GPLv3 (issues #2795, #3224). Autor reconheceu (~mai/2025), resolução incompleta.
- CLI scriptável: `sh Ventoy2Disk.sh -I /dev/sdX` (`-i`/`-I`/`-u`/`-s`/`-g`). **Veredito:** viável automatizar, mas BLOBs não-auditados + `dm-patch` (kernel taint) = dependência questionável p/ produto assinado.

### Alternativas mais fáceis
- **GRUB `loopback` + `iso9660`** — ótimo p/ **Linux**, geralmente **não Windows** (distro precisa cooperar via `findiso=`/`iso-scan/filename=`). 
- **syslinux MEMDISK** — carrega na RAM; inviável p/ instaladores Windows e ISOs live grandes.
- **GLIM/Easy2Boot/SARDU** — config-pesado.
- **Por que Windows é difícil de loopback:** `bootmgr` exige **volume real legível**; o loop device do GRUB não existe no ambiente do bootmgr. Só o Ventoy resolve (remontagem in-OS via `vtoyjump`).

## 3b. Existe método único real (sem ramificação)?
**Não.** 
- **Sempre dd**: funciona p/ Linux; **falha no Windows** (não isohybrid; `install.wim` não cabe na ESP FAT32). "Win11 é dd-able" = **falso/não verificado**.
- **Sempre extrair**: lida com Windows, mas **quebra muitas distros** (checksums de boot, cadeia shim→grub assinada, persistência/casper).
- **Conclusão:** universal-sem-ramificação é impossível na escrita direta. Só o **Ventoy** "unifica", movendo a ramificação p/ o boot (não é "1 ISO → 1 USB raw").

## 4. O que os projetos Rust fazem
- **Popsicle** (Pop!_OS, MIT, v1.3.3 Jan/2024): **só raw**, paralelo. **Sem Windows** (issue #67, aberta desde 2019). Mantenedores: *"detectar que a iso é Windows... copiar arquivos para mount points NTFS, em vez de escrita em nível de bloco"*. Branch `windows-iso` não-merjado. **→ eles próprios concluíram: suportar ambos exige detecção + caminho separado.**
- **caligula** (GPL-3.0): **raw + verificação** + descompressão transparente. **Sem Windows** ("Eventually™").
- **Lacuna de crates** (razão de não existir "Rufus em Rust puro"):

| Crate | Função | Status |
|---|---|---|
| `fatfs` 0.3.6 | FAT R/W | parado 2023-01 |
| `ntfs` 0.4.0 | NTFS **read-only** | não cria NTFS |
| `iso9660` 0.1.1 | ISO9660 reader | incompleto |
| `gpt` 4.1.0 | GPT R/W | **ativo** (2025-03) |
| `gpt-disk-types` 0.16.1 | tipos GPT no_std | **ativo** (2025-04) |
| `mbr-nostd` | MBR parser | abandonado |
| `sys-mount` | mount block/ISO | Linux-only |

## 5. Secure Boot
**Fato-chave:** a assinatura é **Authenticode embutida no binário PE/COFF**, não metadado de FS → tanto dd quanto cópia-de-arquivo **preservam**. Modelo shim: distro envia `shim` assinado pela MS UEFI CA; firmware roda shim → verifica `grubx64.efi`/kernel via MOK. Layout: `\EFI\BOOT\BOOTX64.EFI`=shim; `grubx64.efi`; `mmx64.efi`.

| Método | Secure Boot | Por quê |
|---|---|---|
| Raw dd | ✅ intacto | Cópia byte-idêntica |
| Extração p/ FAT32 | ✅ geralmente | Assinaturas PE sobrevivem; **copiar os `.efi` verbatim** e manter `BOOTX64.EFI`=shim. Quebra se regenerar GRUB (`grub-install`) ou se grub.cfg busca root por UUID/label antigo |
| Windows + UEFI:NTFS | ✅ Rufus ≥ 3.17 | UEFI:NTFS assinado pela MS (ntfs-3g GPLv2); `bootmgfw.efi` MS-assinado. exFAT (GPLv3) **não** assinado |
| Ventoy | ⚠️ com ressalvas | Enrola **própria MOK** e roda qualquer EFI sem verificação (*"bypass secure boot"*). MOK persiste no NVRAM = bypass durável |

**Estado Ventoy 2024–26:** Windows Update 13/ago/2024 (SBAT) revogou shims antigos → Ventoy falhava; corrigido no **Ventoy 1.1.00 (22/jan/2025)** (shim 15.8, "UEFI CA 2023", exige nova chave no 1º boot ≥ 1.1.13). Crítica "backdoor": issue #2184.

## Recomendação para o app
1. **Não dá p/ evitar ramificação** se quiser stick raw que boote ambos. Estratégias: **(A) gravador inteligente com auto-detecção** (design do Popsicle) ou **(B) estilo Ventoy** (chainloader de boot-time).
2. **Caminho Windows recomendado:** GPT/MBR + **FAT32** + copiar conteúdo + **split `install.wim`→`.swm`** (`wimlib`, ≤4000 MiB) + deletar o wim. Sem dep GPL runtime, Secure-Boot-limpo, BIOS+UEFI. Alternativa (manter WIM inteiro): NTFS + UEFI:NTFS (GPLv2).
3. **Extrair ISO Windows (UDF):** 7-Zip ou loop-mount (crates Rust só ISO9660). WIM: shell-out.
4. **Stack Rust:** `gpt`/`gpt-disk-types` (ativos) + `fatfs` p/ FAT/Linux; NTFS/WIM/UDF via subprocesso.
