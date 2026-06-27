# ADR 0004 — Backend de disco Linux via udisks2 + polkit

**Status:** Aceita · **Data:** 2026-06-26

## Contexto
Formatar/gravar pendrive exige IO de baixo nível e privilégio. Queremos evitar rodar a GUI inteira como root. Ver `../pesquisa/02` §1 e §4.

## Decisão
No Linux, o adapter de disco usa **udisks2 via D-Bus** (crate `udisks2 0.3.x` + `zbus 5`):
- Enumeração de removíveis: `rs-drivelist` (filtrando removível/USB) e/ou udisks `Manager`.
- Abertura para gravação: **`Block.OpenDevice()`** com `O_EXCL | O_SYNC | O_CLOEXEC` → retorna **fd gravável autorizado pelo polkit** (prompt por-ação). (`OpenForRestore`/`OpenForBackup` estão deprecados desde udisks 2.7.3.)
- mkfs: udisks `Block.Format` ou shell-out `mkfs.*` (FAT32 também via `fatfs` nativo).
- Após gravar: `fsync` → reler tabela de partição (udisks faz ao fechar o fd) / ejetar.

A porta `PrivilegeElevator` no Linux é essencialmente **no-op** (o polkit do udisks resolve por-ação).

## Justificativa
- **Não rodar a GUI como root** (mais seguro; é o que o Fedora Media Writer faz).
- Tudo em Rust seguro (zbus) → respeita `unsafe_code=forbid` (ADR 0008).
- Evita reimplementar o que o udisks já faz (desmontar, autorizar, reler tabela).

## Consequências
- Depende do daemon udisks2 presente (padrão na maioria dos desktops Linux).
- Windows/macOS usam outros mecanismos de elevação (ver ADR 0003).
- Alternativa considerada e descartada: processo-filho privilegiado via `pkexec`/`sudo` (modelo Popsicle/caligula) — mais simples, porém menos granular que o polkit por-ação.
