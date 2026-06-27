<h1 align="center">Nur · نور</h1>

<p align="center">
  <em>"Luz" — <strong>An-Nûr</strong>, um dos nomes de Deus.<br/>
  Dar boot é acender a máquina.</em>
</p>

---

**Nur** é um app desktop em **Rust** para **formatar pendrives** e **criar pendrives bootáveis a partir de uma imagem ISO** — com foco em ser **simples, prático e à prova de acidentes** (a operação é destrutiva).

## Características

- 🖥️ **UI nativa** em [egui/eframe](https://github.com/emilk/egui) — painel único, leve e moderno.
- 🌗 **Tema claro/escuro** com persistência.
- 🧠 **Gravador inteligente**: detecta o tipo da ISO e escolhe a técnica (raw write para Linux isohybrid; extração para Windows).
- 🛡️ **Segurança em primeiro lugar**: exclui o disco de sistema, exige confirmação explícita, desmonta antes de gravar.
- 🧩 **Arquitetura hexagonal** (workspace de crates), cross-platform (Linux primeiro; depois Windows e macOS).

## Status

🚧 **Em design/planejamento.** Ainda não há código — a documentação de design, pesquisa e decisões está completa.

## Documentação

- 📄 [`docs/README.md`](docs/README.md) — índice geral e estado atual.
- 📐 [`docs/superpowers/specs/`](docs/superpowers/specs/) — especificação de design (UI + arquitetura).
- 🔬 [`docs/pesquisa/`](docs/pesquisa/) — relatórios (projeto de referência, ferramentas Rust, gravação Linux vs Windows).
- 🧭 [`docs/decisoes/`](docs/decisoes/) — ADRs (decisões com o porquê).
- 🎨 [`superdesign/`](superdesign/) — protótipos de UI (HTML interativo).

## Tecnologias

Rust (edição 2024) · egui/eframe 0.35 · tokio · udisks2/zbus (Linux) · arquitetura hexagonal.

---

<p align="center"><sub>Idioma do produto e da documentação: português (pt-BR).</sub></p>
