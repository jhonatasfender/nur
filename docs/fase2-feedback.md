# Fase 2 — Feedback do usuário (lista de TODOs)

Itens reportados durante a implementação da UI, para não atrapalhar o fluxo atual.

| # | Item | Status |
|---|------|--------|
| 1 | Layout deve seguir o protótipo **à risca** | ✅ feito |
| 2 | Baixar/embutir uma **fonte bonita** (Inter) | ✅ feito |
| 3 | Remover a **barra de título preta** (todas as plataformas) | ✅ feito (`with_decorations(false)` + header arrastável) |
| 4 | Janela com **bordas arredondadas** (estava "quadradona") | ✅ feito (janela transparente + card arredondado) |
| 5 | **Dark/light não funcionava** | ✅ feito (lock de tema via `set_visuals_of`+`set_theme`; e drag do header roubava o clique do toggle) |
| 6 | Ajustar **altura do select** | ✅ feito (`interact_size.y` + `button_padding`) |
| 7 | Ajustar **altura e padding do input e do select** | ✅ feito (spacing global + `.margin` nos `TextEdit`) |
| 8 | **Refatorar**: criar pasta de **componentes reutilizáveis** (button, input, select, etc.) | ✅ feito (`crates/ui/src/components/`: `FieldLabel`, `LabeledSelect`, `LabeledInput`, `PrimaryButton`, `SecondaryButton`, `DangerButton`) |

> Validação visual de cada item é feita por screenshot headless (xvfb) — ver `docs/screenshots/`.
