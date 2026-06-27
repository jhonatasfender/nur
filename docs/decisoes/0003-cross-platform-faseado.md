# ADR 0003 — Cross-platform faseado (Linux primeiro)

**Status:** Aceita · **Data:** 2026-06-26

## Contexto
O usuário quer **cross-platform total** (Linux + Windows + macOS). As operações de disco são fortemente específicas por SO; o backend escolhido (udisks2/D-Bus, ADR 0004) **só existe no Linux**.

## Decisão
Arquitetura **cross-platform-ready desde o início** (porta `DiskService` + estrutura de crates), mas **implementação faseada**:
1. **Linux** — adapter udisks2 (primeiro, validável de verdade aqui).
2. **Windows** — adapter via APIs Win32 (`windows` crate: CreateFile/IOCTL lock+dismount+eject; elevação UAC/`runas`).
3. **macOS** — adapter via DiskArbitration / `diskutil`; helper SMJobBless.

A porta abstrai disco; cada SO ganha seu adapter em `infrastructure`.

## Justificativa
- Permite ter algo **rodando e testável rápido no Linux** (máquina do usuário; boot em QEMU) sem travar no IO dos outros SOs.
- udisks2 é Linux-only → Windows/macOS exigem adapters próprios de qualquer forma; faseá-los evita um primeiro milestone gigante.

## Consequências
- A porta `DiskService` deve ser desenhada genérica o suficiente para os 3 SOs.
- Adapters de Windows/macOS podem exigir FFI/`unsafe` → isolar (ver ADR 0008) ou shell-out seguro.
