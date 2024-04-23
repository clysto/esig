use crate::series::MultiResolutionSeries;
use crate::signal_plot::Signal;
use eframe::egui::Widget;
use eframe::egui::{self, Align2, Grid};
use rustfft::num_complex::Complex;
use std::fs::File;
use std::io::Read;
use std::slice;
use std::thread;

#[derive(PartialEq, Clone, Copy)]
pub enum SignalType {
    Float32,
    Complex64,
}

pub struct OpenDialog {
    path: String,
    sample_rate: u32,
    signal_type: SignalType,
    task: Option<thread::JoinHandle<Option<Signal>>>,
}

impl Default for OpenDialog {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            sample_rate: 2000000,
            signal_type: SignalType::Float32,
            task: None,
        }
    }
}

impl OpenDialog {
    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) -> Option<(Signal, String)> {
        if self.task.is_some() {
            *open = true;
        }
        egui::Window::new("Open File")
            .open(open)
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .default_width(300.)
            .max_width(300.)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Grid::new("open-file-options")
                        .num_columns(2)
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("File");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Browse").clicked() {
                                        let path = rfd::FileDialog::new().pick_file();
                                        if path.is_some() {
                                            self.path = path.unwrap().to_str().unwrap().to_owned();
                                        }
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
                            ui.label("Sample Rate");
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    egui::DragValue::new(&mut self.sample_rate)
                                        .custom_formatter(|f, _range| {
                                            if f < 1_000.0 {
                                                return format!("{:.0} Hz", f);
                                            }
                                            if f < 1_000_000.0 {
                                                return format!("{} kHz", f / 1_000.0);
                                            }
                                            if f < 1_000_000_000.0 {
                                                return format!("{} MHz", f / 1_000_000.0);
                                            }
                                            format!("{} GHz", f / 1_000_000_000.0)
                                        })
                                        .custom_parser(|str| {
                                            // str 1000hz 1000mhz 1000 MHz 10GHz 200 GhZ is valid
                                            let mut str = str.to_owned();
                                            str.make_ascii_lowercase();
                                            let num = str.trim_end_matches(|c: char| {
                                                c.is_alphabetic() && c != 'e' && c != '.'
                                            });
                                            let mut unit = str.trim_start_matches(|c: char| {
                                                c.is_numeric() || c == '.' || c == 'e'
                                            });
                                            if unit.is_empty() {
                                                unit = "hz";
                                            }
                                            let num = num.parse::<f64>();
                                            if num.is_err() {
                                                return None;
                                            }
                                            let num = num.unwrap();
                                            match unit {
                                                "hz" => return Some(num),
                                                "khz" => return Some(num * 1_000.0),
                                                "mhz" => return Some(num * 1_000_000.0),
                                                "ghz" => return Some(num * 1_000_000_000.0),
                                                _ => return None,
                                            }
                                        })
                                        .speed(1.0)
                                        .ui(ui);
                                },
                            );
                            ui.end_row();
                            ui.label("Signal Type");
                            egui::ComboBox::from_label("")
                                .selected_text(match self.signal_type {
                                    SignalType::Float32 => "float32",
                                    SignalType::Complex64 => "complex64",
                                })
                                .width(ui.available_width())
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
                            let path = self.path.clone();
                            let signal_type = self.signal_type.clone();
                            self.task = Some(thread::spawn(move || open_file(path, signal_type)));
                        }
                        if self.task.is_some() {
                            ui.spinner();
                        }
                    })
                })
            });
        if self.task.is_some() && self.task.as_ref().unwrap().is_finished() {
            let result = self.task.take().unwrap().join();
            if result.is_ok() {
                *open = false;
                self.task = None;
                return Some((result.unwrap().unwrap(), self.path.clone()));
            }
        }
        None
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

fn open_file(path: String, signal_type: SignalType) -> Option<Signal> {
    let result = File::open(path);
    if let Ok(mut file) = result {
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer);
        unsafe {
            if signal_type == SignalType::Float32 {
                let len = buffer.len() / 4;
                let ptr = buffer.as_ptr() as *const f32;
                let slice = slice::from_raw_parts(ptr, len);
                return Some(Signal::Real(MultiResolutionSeries::build(slice, 2048)));
            } else {
                let len = buffer.len() / 8;
                let ptr = buffer.as_ptr() as *const Complex<f32>;
                let slice = slice::from_raw_parts(ptr, len);
                return Some(Signal::Complex(MultiResolutionSeries::build(slice, 2048)));
            }
        }
    }
    None
}
