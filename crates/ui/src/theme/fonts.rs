//! Fonte da UI: Inter embutida, para um visual mais bonito que a padrão.

/// Registra a fonte Inter no contexto egui.
pub(crate) struct Fonts;

impl Fonts {
    // Inter (variable, OFL-1.1) embutida no binário.
    const INTER: &[u8] = include_bytes!("../../assets/fonts/Inter.ttf");

    /// Instala a Inter como proporcional primária, preservando os fallbacks
    /// padrão do egui (ícones/emoji). Chamar uma única vez.
    pub(crate) fn install(ctx: &egui::Context) {
        let mut defs = egui::FontDefinitions::default();
        defs.font_data.insert(
            "Inter".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(Self::INTER)),
        );
        if let Some(family) = defs.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "Inter".to_owned());
        }
        ctx.set_fonts(defs);
    }
}
