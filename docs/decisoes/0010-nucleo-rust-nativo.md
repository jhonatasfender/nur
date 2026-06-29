# ADR 0010 — Núcleo Rust-nativo (sem ferramentas externas) e re-escopo do formatador

**Status:** Aceita · **Data:** 2026-06-29 · **Revisa:** ADR 0004 (parte de `Block.Format`/mkfs) e ADR 0005 (ambição "qualquer ISO já").

## Contexto

Princípio de produto reforçado pelo usuário: **quem usa o app não deve precisar instalar nada**. No Linux, as operações destrutivas dependiam de ferramentas externas: `mkfs.vfat/ntfs/exfat/ext4` (via udisks `Block.Format`) e, para Windows, `wimlib` (split do `install.wim`) + leitura de **UDF**.

Levantamento do que existe em **Rust puro maduro**:
- **Tem:** particionamento GPT/MBR (`gpt`, `mbrman`), criação de **FAT32** (`fatfs`), leitura ISO9660, gravação raw.
- **Não tem (só leitura ou inexistente):** criação de `ext4`/`NTFS`/`exFAT`, leitura de **UDF**, split de **WIM**.

## Decisão

Adotar um **núcleo Rust-nativo** e **escopar o produto ao que fecha em Rust**, sem binários externos:

1. **Gravação raw de ISO (Linux):** permanece Rust-nativo (já é). Privilégio via udisks2/polkit (fd) — udisks2 **não** é "instalar algo": é serviço universal de desktop, só o mecanismo de elevação.
2. **Formatar:** reescrito em **Rust nativo** — `OpenDevice` (fd privilegiado) → `gpt`/`mbrman` (tabela) → `fatfs` (FAT32). **Sem `mkfs`, sem `Block.Format`.** A UI passa a oferecer **somente FAT32** (GPT/MBR mantidos). NTFS/exFAT/ext4 saem de escopo (não há criação em Rust).
3. **Windows:** **deferido**, e **reduzido a duas lacunas**. A formatação do Windows-USB (UEFI) é **GPT + FAT32** — a mesma fundação do item 2, reaproveitada. O que falta é só **ler a ISO em UDF** e **split do `install.wim`**, ambos sem caminho Rust maduro. Decisão sobre essas lacunas (escrever em Rust × embutir binário só nelas × manter deferido) fica para quando o incremento Windows for retomado. O spike (`docs/spikes/2026-06-29-windows-uefi-recipe.md`) segue como referência da receita.

## Justificativa

- **Zero instalação no caso principal** (pendrive bootável de ISO Linux) e na formatação FAT32.
- **Mais testável:** formatar FAT32 e escrever GPT viram unit tests em buffer (`Cursor<Vec<u8>>`), sem device — diferente do `Block.Format` (casca opaca).
- **Honestidade de escopo:** não prometer NTFS/exFAT/Windows enquanto não há caminho Rust; readicionar quando (e se) houver decisão consciente para as lacunas.

## Consequências

- `Udisks2Formatter` (via `Block.Format`/mkfs) é substituído por um formatador Rust-nativo (`gpt`/`mbrman` + `fatfs`).
- UI do *Formatar*: sem seletor de filesystem (sempre FAT32). `FilesystemKind` e `FormatError::ToolMissing` deixam de existir.
- Novas deps: `gpt`, `fatfs`, `mbrman` (Rust puro).
- udisks2/polkit permanecem só como mecanismo de privilégio (abrir o device sem rodar como root).
- ADR 0005 deixa de ser "funciona para qualquer ISO já"; passa a "núcleo Rust-nativo (Linux raw + FAT32); Windows = fundação FAT32 + lacunas UDF/WIM a decidir".
