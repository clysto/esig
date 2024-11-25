use eframe::egui::Widget;
use eframe::egui::{self, Align2, Grid};
use egui_file_dialog::FileDialog;
use std::path::PathBuf;

pub struct ExportDialog {
    path: String,
    file_dialog: FileDialog,
}

impl Default for ExportDialog {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            file_dialog: FileDialog::new()
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0., 0.)),
        }
    }
}

impl ExportDialog {
    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> Option<PathBuf> {
        let mut ok = false;
        egui::Window::new("Export")
            .open(open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .default_width(300.)
            .max_width(300.)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Grid::new("export-dialog")
                        .num_columns(2)
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("Path");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Browse").clicked() {
                                        self.file_dialog.save_file();
                                    }
                                    ui.with_layout(
                                        egui::Layout::top_down_justified(egui::Align::LEFT),
                                        |ui| {
                                            egui::TextEdit::singleline(&mut self.path).ui(ui);
                                        },
                                    );
                                },
                            );
                            ui.end_row();
                        });
                });
                ui.add_space(30.);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Save").clicked() {
                            ok = true;
                        }
                    })
                });
                self.file_dialog.update(ctx);
                if let Some(path) = self.file_dialog.take_selected() {
                    self.path = path.to_str().unwrap().to_owned();
                }
            });
        if ok {
            return Some(self.path.clone().into());
        } else {
            return None;
        }
    }
}
