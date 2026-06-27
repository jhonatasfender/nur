# ADR 0009 — Nome do projeto: Nur (نور)

**Status:** Aceita · **Data:** 2026-06-26

## Contexto
O projeto precisava de um nome. O usuário pediu algo que **glorificasse o Deus Criador**, numa **língua antiga** (grego, egípcio ou árabe antigo), com **impacto**. Critérios técnicos: curto, fácil de digitar, bom como nome de binário, sem colisão óbvia com concorrentes (Rufus, Etcher, Ventoy, Popsicle, caligula).

## Decisão
**Nome: Nur** (árabe **نور**, transliterado *Nûr*/*Noor*), que significa **"luz"**.

- **Significado teológico:** *An-Nûr* (النور), "A Luz", é um dos nomes de Deus.
- **Relevância para o app:** dar boot é "acender"/trazer a luz à máquina — o nome conversa diretamente com a função.
- **Nome de exibição:** Nur.
- **Nome do binário:** `nur`.
- **Crates do workspace:** seguir a convenção do `solana` (crates por caminho, não publicados): `domain`, `application`, `infrastructure`, `app`, `ui`, `tools/xtask`. O binário produzido é `nur`.

## Alternativas consideradas
- Grego: **Eikōn** (Εἰκών, "imagem" — imago Dei + imagem ISO), **Phôs** (Φῶς, "luz" — "Luz da Luz" do Credo), **Dóxa** (Δόξα, "glória"), **Lógos** (Λόγος, "o Verbo").
- Árabe: **Al-Khâliq** (الخالق, "O Criador").
- Egípcio: **Ankh** (☥, "vida"). Descartados nomes de deuses pagãos egípcios (Atum, Rá) por conflito com a glorificação do Deus Criador único.

## Consequências / pendências
- O diretório atual ainda é `format-bootpendrive`; renomear o repositório para `nur` é opcional (fazer quando conveniente).
- **Verificar colisão em crates.io** caso um dia se publique alguma lib (existe um crate `nur` task-runner não relacionado); como é app desktop, o binário `nur` local não conflita. Para publicação, usar prefixo (`nur-*`) se necessário.
- Considerar uma identidade visual com o tema "luz" (já casa com o tema claro/escuro — ADR 0007).
