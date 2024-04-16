use crate::fft::compute_psd;
use crate::open_dialog::OpenDialog;
use crate::psd_dialog::PsdDialog;
use crate::signal_plot::{Signal, SignalPlot};
use eframe::egui::{self, Button, Key, KeyboardShortcut, Modifiers, OpenUrl, Widget};
use rustfft::num_complex::Complex;

pub struct App {
    open_dialog: OpenDialog,
    psd_dialog: PsdDialog,
    open_dialog_visible: bool,
    psd_dialog_visible: bool,
    sample_rate: u32,
    psd_visiable: bool,
    signal_plot: SignalPlot,
}

impl Default for App {
    fn default() -> Self {
        Self {
            open_dialog: OpenDialog::default(),
            psd_dialog: PsdDialog::default(),
            open_dialog_visible: false,
            psd_dialog_visible: false,
            sample_rate: 1,
            psd_visiable: false,
            signal_plot: SignalPlot::new(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if ui.ctx().input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::O))
            }) {
                self.open_dialog_visible = true;
            }
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let shortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
                    if Button::new("Open")
                        .shortcut_text(ui.ctx().format_shortcut(&shortcut))
                        .ui(ui)
                        .clicked()
                    {
                        ui.close_menu();
                        self.open_dialog_visible = true;
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Save Current").clicked() {}
                    if ui.button("PSD").clicked() {
                        ui.close_menu();
                        self.psd_visiable = true;
                        if self.signal_plot.have_signal() {
                            let mut input = vec![];
                            match self.signal_plot.signal() {
                                Signal::Real(sig) => {
                                    for s in sig.get(self.signal_plot.range(), 1).iter() {
                                        input.push(Complex::new(*s as f64, 0.0));
                                    }
                                }
                                Signal::Complex(sig) => {
                                    for s in sig.get(self.signal_plot.range(), 1).iter() {
                                        input.push(Complex::new(s.re as f64, s.im as f64));
                                    }
                                }
                            }
                            let (freqs, psd) =
                                compute_psd(&input, 1024, 0, self.sample_rate as f64);
                            self.psd_dialog.set_data(freqs, psd);
                            self.psd_dialog_visible = true;
                        }
                    }
                });
                ui.menu_button("About", |ui| {
                    if ui.button("Toggle Dark/Light").clicked() {
                        ui.close_menu();
                        let visuals = if ui.visuals().dark_mode {
                            egui::Visuals::light()
                        } else {
                            egui::Visuals::dark()
                        };
                        ctx.set_visuals(visuals);
                    }
                    use egui::special_emojis::GITHUB;
                    if ui.button(format!("{GITHUB} esig on GitHub")).clicked() {
                        ui.close_menu();
                        ctx.open_url(OpenUrl {
                            url: "https://github.com/emilk/egui".to_owned(),
                            new_tab: true,
                        })
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.signal_plot.show(ui);

            let sig = self.open_dialog.show(ctx, &mut self.open_dialog_visible);
            if sig.is_some() {
                self.open_dialog_visible = false;
                self.signal_plot.set_signal(sig.unwrap());
                self.sample_rate = self.open_dialog.sample_rate();
            }

            self.psd_dialog.show(ctx, &mut self.psd_dialog_visible);
        });
    }
}
