# ADR 0005 — Gravador inteligente com auto-detecção (Ventoy descartado)

**Status:** Aceita · **Data:** 2026-06-26

## Contexto
O usuário quer que o app "funcione para qualquer ISO" (Linux e Windows) com uma experiência simples. A pesquisa (`../pesquisa/03`) mostrou que **não existe método raw único** que sirva para os dois: ISO Linux isohybrid quer **cópia byte-a-byte (dd)**; ISO Windows quer **particionar + copiar arquivos** (FAT32 + split do WIM). São mutuamente exclusivos na escrita.

## Decisão
Adotar **Opção A — Gravador inteligente com auto-detecção**:
- Um único botão "Criar bootável".
- A porta `BootableWriter` usa um **`IsoInspector`** que lê os primeiros setores (assinatura `0x55AA` + tabela de partição em 0x1BE; `CD001` em 0x8001) e a árvore de arquivos (`bootmgr`, `sources/install.wim`) para escolher a estratégia:
  - **`RawWriteStrategy`** — ISO isohybrid (maioria Linux): raw write no device inteiro.
  - **`WindowsExtractStrategy`** — ISO Windows: GPT/MBR + FAT32 + copiar arquivos + **split `install.wim`→`.swm`** (`wimlib`/DISM, ≤4000 MiB) + deletar o wim original.
- O usuário não escolhe a técnica; o app decide.

## Alternativa descartada — Opção B (estilo Ventoy)
Move a ramificação para o tempo-de-boot (bootloader multi-ISO). Descartada porque:
- Muda o conceito do produto (multi-boot, não "1 ISO → 1 pendrive") e invalida a UI aprovada.
- **GPLv3+**; árvore com **binários pré-compilados não auditados** (controvérsia 2021–2025); usa `dm-patch` que **taint o kernel**; modelo de Secure Boot enrola **MOK própria** = bypass durável no NVRAM.
- Dependência arriscada para um produto que pretendemos garantir/assinar.

## Justificativa
- Mantém a UI e o conceito aprovados.
- Caminho Windows recomendado (FAT32 + split do WIM) é **Secure-Boot-limpo** e **sem deps GPL em runtime** (`../pesquisa/03` §2/§5).
- É o mesmo design que os mantenedores do Popsicle (Rust) concluíram ser necessário.

## Consequências
- Dois caminhos de código sob o mesmo botão.
- O caminho Windows é **fortemente baseado em shell-out** (sem crates Rust maduros p/ UDF, NTFS-write, WIM split): `7z`/loop-mount, `mkfs.ntfs`, `wimlib`/DISM. Ver ADR 0006 para a ordem de implementação.
