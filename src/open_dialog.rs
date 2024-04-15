use eframe::egui::{self, Align2, Grid};
use emath::vec2;

#[derive(PartialEq, Clone)]
pub enum SignalType {
    Float32,
    Complex64,
}

pub struct OpenDialog {
    path: String,
    sample_rate: u32,
    signal_type: SignalType,
}

impl Default for OpenDialog {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            sample_rate: 1,
            signal_type: SignalType::Float32,
        }
    }
}

impl OpenDialog {
    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> bool {
        let mut ok = false;
        egui::Window::new("Open File")
            .open(open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Grid::new("open-file-options")
                        .num_columns(2)
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("File");
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    vec2(200.0, 18.0),
                                    egui::TextEdit::singleline(&mut self.path),
                                );
                                if ui.button("Browse").clicked() {
                                    let path = rfd::FileDialog::new().pick_file();
                                    if path.is_some() {
                                        self.path = path.unwrap().to_str().unwrap().to_owned();
                                    }
                                }
                            });
                            ui.end_row();
                            ui.label("Sample Rate");
                            ui.add_sized(
                                vec2(200.0, 18.0),
                                egui::DragValue::new(&mut self.sample_rate)
                                    .speed(1.0)
                                    .suffix(" Hz"),
                            );
                            ui.end_row();
                            ui.label("Signal Type");
                            egui::ComboBox::from_label("")
                                .selected_text(match self.signal_type {
                                    SignalType::Float32 => "float32",
                                    SignalType::Complex64 => "complex64",
                                })
                                .width(200.)
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap = Some(false);
                                    ui.selectable_value(
                                        &mut self.signal_type,
                                        SignalType::Float32,
                                        "float32",
                                    );
                                    ui.selectable_value(
                                        &mut self.signal_type,
                                        SignalType::Complex64,
                                        "complex64",
                                    );
                                });
                            ui.end_row();
                        });
                });
                ui.add_space(30.);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("OK").clicked() {
                            ok = true;
                        }
                    })
                })
            });
        ok
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn signal_type(&self) -> SignalType {
        self.signal_type.clone()
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }
}
