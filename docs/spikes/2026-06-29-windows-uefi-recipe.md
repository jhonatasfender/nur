# Spike — Receita Windows bootável (UEFI) num pendrive

**Data:** 2026-06-29 · **Tipo:** spike descartável (validação manual, fora do app) · **Relacionado:** ADR 0005, ADR 0006, `pesquisa/03`.

## Por que este spike

O ADR 0006 manda **retirar o risco do Windows antes** de implementar no app. Antes de escrever o `WindowsExtractStrategy`, vamos provar **na mão** que a receita recomendada (`pesquisa/03` §2) gera um pendrive que **boota o instalador do Windows em UEFI**:

> GPT + 1 partição **FAT32** → copiar os arquivos da ISO → **split do `install.wim` em `.swm`** (FAT32 não aceita arquivo > 4 GiB) → bootar em UEFI.

Se funcionar, implementamos com confiança. Se quebrar, descobrimos o ajuste **agora**, sem custo de código.

> ⚠️ **Destrutivo.** Os passos **apagam o pendrive** escolhido. Confira o `/dev/sdX` com calma — pegar o disco errado destrói dados. Nada aqui entra no app; é validação de uma vez só.

## Pré-requisitos

- **Linux** com: `sgdisk` (pacote `gptfdisk`), `mkfs.vfat` (`dosfstools`), `wimlib-imagex` (`wimtools`/`wimlib`), `rsync`, e `7z` (`p7zip`) **ou** suporte a loop-mount de UDF (módulo `udf`).
- Uma **ISO do Windows 10/11** (`Win.iso`).
- Um **pendrive ≥ 8 GB** (será apagado).
- Para bootar: **QEMU + OVMF** (`ovmf`/`edk2-ovmf`) **ou** um PC real com boot UEFI.

Checar ferramentas:
```bash
for t in sgdisk mkfs.vfat wimlib-imagex rsync 7z qemu-system-x86_64; do
  command -v "$t" >/dev/null && echo "ok: $t" || echo "FALTA: $t"
done
```

## Passo 0 — Identificar o pendrive (com cuidado)

```bash
lsblk -o NAME,SIZE,TYPE,TRAN,RM,MOUNTPOINT,MODEL
```
Identifique a linha do pendrive (coluna `TRAN=usb`, `RM=1`). **Confirme o tamanho e o modelo.** Então fixe a variável (ajuste `sdX`):
```bash
DEV=/dev/sdX        # ex.: /dev/sdb  — NÃO use o disco do sistema!
ISO=$HOME/Downloads/Win.iso
# Salvaguarda: aborta se não for removível/USB.
test "$(lsblk -ndo RM "$DEV")" = "1" && lsblk -ndo TRAN "$DEV" | grep -q usb \
  && echo "ok, $DEV parece um pendrive removível USB" \
  || { echo "ABORTAR: $DEV não parece um pendrive removível USB"; }
```
Desmonte qualquer partição já montada do pendrive:
```bash
sudo umount "${DEV}"* 2>/dev/null || true
```

## Passo 1 — Particionar GPT + FAT32 (ESP)

```bash
sudo sgdisk --zap-all "$DEV"                              # zera tabelas antigas
sudo sgdisk -n 1:0:0 -t 1:EF00 -c 1:WINUSB "$DEV"        # 1 partição cobrindo o disco, tipo ESP
sudo partprobe "$DEV"; sleep 1
sudo mkfs.vfat -F 32 -n WINUSB "${DEV}1"                  # FAT32
lsblk -f "$DEV"                                           # confere: vfat em ${DEV}1
```

## Passo 2 — Montar ISO e pendrive

```bash
sudo mkdir -p /mnt/winiso /mnt/winusb
sudo mount -o loop,ro "$ISO" /mnt/winiso   # UDF é auto-detectado; se falhar, ver "Plano B" no fim
sudo mount "${DEV}1" /mnt/winusb
ls /mnt/winiso                              # deve listar bootmgr, sources/, efi/, etc.
```

## Passo 3 — Copiar tudo, EXCETO o install.wim

```bash
sudo rsync -ah --info=progress2 --exclude 'sources/install.wim' /mnt/winiso/ /mnt/winusb/
```

## Passo 4 — Split do install.wim → install.swm (≤ 4000 MiB)

```bash
sudo wimlib-imagex split /mnt/winiso/sources/install.wim /mnt/winusb/sources/install.swm 4000
ls -lh /mnt/winusb/sources/install*.swm    # install.swm, install2.swm, ...
```
> O Windows Setup aceita `.swm` no lugar de `.wim` e rejunta sozinho. **Não** copie o `install.wim` original (ele estoura o limite de 4 GiB do FAT32).

## Passo 5 — Conferir o boot UEFI e finalizar

```bash
ls /mnt/winusb/efi/boot/bootx64.efi        # FAT32 é case-insensitive; deve existir
sync
sudo umount /mnt/winusb /mnt/winiso
```
Se `bootx64.efi` **não** existir (raro em ISOs modernas), copie de `efi/microsoft/boot/bootmgfw.efi` para `efi/boot/bootx64.efi` antes do `umount` — anote se precisou.

## Passo 6 — Bootar e validar

**Opção A — QEMU + OVMF (aponta pro pendrive; precisa de root p/ acessar o device):**
```bash
# Ajuste o caminho do OVMF conforme a distro:
OVMF=/usr/share/OVMF/OVMF_CODE.fd     # ou /usr/share/edk2-ovmf/x64/OVMF_CODE.fd
sudo qemu-system-x86_64 -enable-kvm -m 4096 \
  -drive if=pflash,format=raw,readonly=on,file="$OVMF" \
  -drive file="$DEV",format=raw,if=virtio \
  -boot menu=on
```
**Opção B — PC real:** boote o pendrive pelo menu UEFI (geralmente F12/F10/ESC), escolhendo a entrada **UEFI: WINUSB**.

**Sucesso =** o **instalador do Windows inicia** (tela "Windows Setup" / seleção de idioma) e avança até a seleção de disco sem erro de mídia.

## Resultado (preencher após rodar)

- [ ] Ferramentas presentes / faltou alguma? `____`
- [ ] Particionou e formatou FAT32 sem erro? `____`
- [ ] `install.wim` tinha > 4 GiB? Quantos `.swm` o split gerou? `____`
- [ ] `efi/boot/bootx64.efi` já existia (ou precisou copiar)? `____`
- [ ] Bootou em UEFI (QEMU ou PC real) e o Windows Setup iniciou? `____`
- [ ] Algo inesperado / ajuste necessário na receita? `____`

> Com este resultado, partimos pro brainstorming do incremento Windows **no app** (detecção UDF + extração + partição + WIM split + bootabilidade), agora com a receita confirmada.

## Plano B — extração sem loop-mount de UDF

Se `mount -o loop,ro` falhar (sem módulo `udf`), use o 7-Zip:
```bash
# Copia tudo menos o install.wim:
sudo 7z x "$ISO" -o/mnt/winusb -xr'!sources/install.wim' -y
# Extrai só o install.wim para um temporário e faz o split de lá:
7z x "$ISO" sources/install.wim -o/tmp/winwim -y
sudo wimlib-imagex split /tmp/winwim/sources/install.wim /mnt/winusb/sources/install.swm 4000
```

## Fora de escopo do spike

- BIOS legado (MBR + partição ativa) — só UEFI aqui.
- NTFS + UEFI:NTFS (alternativa p/ manter o `install.wim` inteiro) — a receita escolhida é o split em FAT32.
- Qualquer código no app — isto é validação manual descartável.
