//! Seções de imagem ISO e opções de formato.

use super::{Mode, NurApp};
use crate::components::{Checkbox, FieldLabel, LabeledInput, LabeledSelect};
use crate::theme::Palette;
use domain::IsoKind;

const PARTITIONS: [&str; 2] = ["GPT", "MBR"];
const TARGETS: [&str; 3] = ["UEFI", "BIOS", "BIOS ou UEFI"];
const FILESYSTEMS: [&str; 4] = ["FAT32", "NTFS", "exFAT", "ext4"];

impl NurApp {
    pub(super) fn iso_section(&mut self, ui: &mut egui::Ui, palette: Palette) {
        if self.mode != Mode::Boot {
            return;
        }
        FieldLabel::show(ui, palette, "IMAGEM ISO");
        let selection = self.state.selected_iso();
        let main = match &selection {
            Some(sel) => format!("{} \u{00B7} {}", sel.name(), sel.size().humanize()),
            None => "Clique para selecionar um arquivo".to_owned(),
        };
        let area = egui::Frame::NONE
            .fill(palette.control())
            .stroke(egui::Stroke::new(1.0, palette.border()))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(12, 16))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(main).color(palette.text()).size(13.0));
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new(".iso \u{00B7} .img")
                            .color(palette.muted())
                            .size(11.0),
                    );
                });
            });
        let zone = ui.interact(
            area.response.rect,
            egui::Id::new("iso_zone"),
            egui::Sense::click(),
        );
        if zone.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if zone.clicked() {
            self.commands.pick_iso();
        }
        if selection.is_some_and(|s| s.kind() == IsoKind::Unsupported) {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(
                    "\u{26A0} ISO não-isohybrid \u{2014} gravação raw indisponível; \
                     ISOs Windows precisam do modo extração, ainda não disponível.",
                )
                .color(palette.destructive())
                .size(12.0),
            );
        }
        ui.add_space(18.0);
    }

    pub(super) fn options_section(&mut self, ui: &mut egui::Ui, palette: Palette) {
        FieldLabel::show(ui, palette, "OPÇÕES DE FORMATO");
        ui.columns(2, |cols| {
            LabeledSelect::show(
                &mut cols[0],
                palette,
                "partition",
                "Esquema de partição",
                &PARTITIONS,
                &mut self.partition,
            );
            LabeledSelect::show(
                &mut cols[1],
                palette,
                "target",
                "Sistema alvo",
                &TARGETS,
                &mut self.target,
            );
        });
        ui.add_space(12.0);
        ui.columns(2, |cols| {
            LabeledSelect::show(
                &mut cols[0],
                palette,
                "fs",
                "Sistema de arquivos",
                &FILESYSTEMS,
                &mut self.filesystem,
            );
            LabeledInput::show(&mut cols[1], palette, "Rótulo do volume", &mut self.label);
        });
        ui.add_space(12.0);
        Checkbox::show(ui, palette, "Formatação rápida", &mut self.quick_format);
    }
}
