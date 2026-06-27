# ADR 0006 — Raw write primeiro + spike Windows cedo

**Status:** Aceita · **Data:** 2026-06-26 · **A confirmar com o usuário** (proposta na conversa; aguardando OK final antes do plano)

## Contexto
Definida a Opção A (ADR 0005), surge a pergunta: implementar primeiro o caminho **fácil** (raw write/Linux) ou o **difícil** (extração Windows)? O usuário levantou começar pelo difícil ("encarar o medo primeiro").

## Decisão (proposta)
**Não inverter a ordem de construção.** Construir **raw write primeiro**, mas **retirar o risco do Windows cedo** com um spike descartável:

- **Spike (cedo, paralelo):** validar manualmente a receita Windows — FAT32 + split do `install.wim` via `wimlib` + bootar instalador Windows em VM. Confirma que a receita funciona **antes** de implementar pra valer.
- **Fase 1:** fundação + adapter Linux (udisks2) + `RawWriteStrategy` + `IsoInspector` já detectando Windows (avisa "ainda não suportado" em vez de gravar errado) + UI completa rodando no Linux (validável bootando ISO Linux em QEMU).
- **Fase 2:** `WindowsExtractStrategy` atrás do mesmo botão.
- **Fase 3:** adapters Windows/macOS (ADR 0003).

## Justificativa
- O **risco arquitetural real** mora na **fundação compartilhada** (udisks/polkit fd, ponte tokio→egui, fronteiras hexagonais, segurança), que o **raw write já exercita de ponta a ponta** e valida rápido (QEMU em dias).
- O caminho Windows é **trabalhoso, não arquiteturalmente incerto** (orquestração de shell-out); e exige **loop de feedback lento** (bootar Windows real com UEFI+Secure Boot).
- Começar pelo Windows obrigaria a construir fundação + particionamento + mkfs + extração + WIM + bootloader **tudo de uma vez**, com muitos pontos de falha simultâneos.
- O spike honra o instinto de "encarar o risco cedo" sem inverter a ordem de construção, e cobre o que o raw write **não** toca (criar tabela de partição, mkfs, montar, copiar).

## Consequências
- Primeira entrega utilizável (Linux) sai cedo.
- Risco do Windows conhecido antes do investimento grande.
- **Pendência:** confirmar com o usuário se aceita esta ordem ou se prefere o caminho Windows completo como primeira entrega.
