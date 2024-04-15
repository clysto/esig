use crate::fft::compute_psd;
use crate::open_dialog::{OpenDialog, SignalType};
use crate::sig::MultiResolutionSignal;
use eframe::egui::{self, Button, Key, KeyboardShortcut, Modifiers, OpenUrl, Vec2b, Widget};
use egui_plot::{Legend, Line, PlotBounds, PlotPoints};
use emath::vec2;
use rustfft::num_complex::Complex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::slice;

pub struct App {
    data: Vec<f32>,
    signals: HashMap<String, MultiResolutionSignal>,
    last_bounds: Option<PlotBounds>,
    open_dialog: OpenDialog,
    open_dialog_visible: bool,
    max_points: usize,
    sample_rate: u32,
    psd_visiable: bool,
    freqs: Vec<f64>,
    psd: Vec<f64>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: vec![],
            signals: HashMap::new(),
            last_bounds: None,
            open_dialog: OpenDialog::default(),
            open_dialog_visible: false,
            max_points: 2048,
            sample_rate: 1,
            psd_visiable: false,
            freqs: vec![],
            psd: vec![],
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    fn open_file(&mut self, path: &String, signal_type: SignalType) {
        let result = File::open(path);
        if let Ok(mut file) = result {
            self.data.clear();
            let mut buffer = Vec::new();
            let _ = file.read_to_end(&mut buffer);
            let len = buffer.len() / 4;
            let ptr = buffer.as_mut_ptr() as *mut f32;
            unsafe {
                let slice = slice::from_raw_parts_mut(ptr, len);
                self.signals.clear();
                if signal_type == SignalType::Float32 {
                    self.signals.insert(
                        "inphase".to_string(),
                        MultiResolutionSignal::new(slice, self.max_points),
                    );
                } else {
                    let re = slice.iter().step_by(2).copied().collect::<Vec<f32>>();
                    let im = slice
                        .iter()
                        .skip(1)
                        .step_by(2)
                        .copied()
                        .collect::<Vec<f32>>();
                    self.signals.insert(
                        "inphase".to_string(),
                        MultiResolutionSignal::new(&re, self.max_points),
                    );
                    self.signals.insert(
                        "quadrature".to_string(),
                        MultiResolutionSignal::new(&im, self.max_points),
                    );
                }
            }
        }
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
                        let tmp = self.last_bounds.unwrap().range_x();
                        if !self.signals.is_empty() {
                            self.psd_visiable = true;
                            let re = self
                                .signals
                                .get("inphase")
                                .unwrap()
                                .get(*tmp.start() as usize, *tmp.end() as usize);
                            let im;
                            if self.signals.get("quadrature").is_none() {
                                im = vec![0.; re.len()];
                            } else {
                                im = self
                                    .signals
                                    .get("quadrature")
                                    .unwrap()
                                    .get(*tmp.start() as usize, *tmp.end() as usize);
                            }
                            let mut input = Vec::new();
                            for (r, i) in re.iter().zip(im.iter()) {
                                input.push(Complex::new(*r as f64, *i as f64));
                            }
                            let (freqs, psd) =
                                compute_psd(&input, 1024, 0, self.sample_rate as f64);
                            self.freqs = freqs;
                            self.psd = psd;
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
            let mut z_pressed = false;
            let mut space_pressed = false;
            if ctx.input(|i| i.key_down(Key::Z)) {
                z_pressed = true;
            }
            if ctx.input(|i| i.key_down(Key::Space)) {
                space_pressed = true;
            }
            egui_plot::Plot::new("signal")
                .legend(Legend::default())
                .auto_bounds(Vec2b::new(false, false))
                .allow_double_click_reset(false)
                .allow_zoom(Vec2b::new(!z_pressed, z_pressed))
                .allow_drag(space_pressed)
                .allow_boxed_zoom(!space_pressed)
                .boxed_zoom_pointer_button(egui::PointerButton::Primary)
                .x_axis_label("Samples")
                .show(ui, |plot_ui| {
                    if self.last_bounds.is_none() {
                        self.last_bounds = Some(PlotBounds::from_min_max(
                            [0., -1.],
                            [self.max_points as f64, 1.],
                        ));
                        plot_ui.set_plot_bounds(self.last_bounds.unwrap());
                    }
                    let bounds = plot_ui.plot_bounds();
                    // If the x-axis range changes, hide the PSD plot
                    if bounds.range_x() != self.last_bounds.unwrap().range_x() {
                        self.psd_visiable = false;
                    }
                    self.last_bounds = Some(bounds);
                    let mut x1 = *bounds.range_x().start();
                    let x2 = *bounds.range_x().end();
                    if !self.signals.is_empty() && x2 >= 0. {
                        if x1 < 0. {
                            x1 = 0.;
                        }
                        let index_start = x1.floor() as usize;
                        let index_end = x2.ceil() as usize;
                        for (name, signal) in self.signals.iter() {
                            let points = signal.points(index_start, index_end);
                            let line = Line::new(points).name(name);
                            plot_ui.line(line);
                        }
                    }
                });

            if self.open_dialog.show(ctx, &mut self.open_dialog_visible) {
                self.open_dialog_visible = false;
                self.sample_rate = self.open_dialog.sample_rate();
                self.open_file(&self.open_dialog.path(), self.open_dialog.signal_type());
            }

            egui::Window::new("PSD")
                .open(&mut self.psd_visiable)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    egui_plot::Plot::new("psd")
                        .set_margin_fraction(vec2(0., 0.1))
                        .x_axis_label("Frequency")
                        .y_axis_label("PSD (dB)")
                        .show(ui, |plot_ui| {
                            let line = Line::new(PlotPoints::new(
                                self.freqs
                                    .iter()
                                    .zip(self.psd.iter())
                                    .map(|(&x, &y)| [x, y])
                                    .collect(),
                            ))
                            .fill(-1000.)
                            .name("PSD");
                            plot_ui.line(line);
                        });
                });
        });
    }
}
