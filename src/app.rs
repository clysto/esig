use crate::export_dialog::ExportDialog;
use crate::fft::compute_psd;
use crate::menubar::{MenuBar, MenuItem};
use crate::open_dialog::OpenDialog;
use crate::psd_dialog::PsdDialog;
use crate::signal_plot::{Signal, SignalPlot};
use crate::utils::{human_readable_freq, human_readable_time};
use eframe::egui::{self, Key, Modifiers};
use rustfft::num_complex::Complex;
use std::fs::File;
use std::io::Write;
use std::slice;

#[derive(Clone)]
enum MenuAction {
    Open,
    Export,
    Quit,
    Reset,
    Return,
    Psd,
    About,
    Mag,
}

pub struct App {
    open_dialog: OpenDialog,
    menubar: MenuBar<MenuAction>,
    psd_dialog: PsdDialog,
    export_dialog: ExportDialog,
    export_dialog_visible: bool,
    open_dialog_visible: bool,
    psd_dialog_visible: bool,
    sample_rate: u32,
    psd_visiable: bool,
    signal_plot: SignalPlot,
    signal_path: String,
}

impl Default for App {
    fn default() -> Self {
        let mut ret = Self {
            open_dialog: OpenDialog::default(),
            menubar: MenuBar::new(),
            psd_dialog: PsdDialog::default(),
            export_dialog: ExportDialog::default(),
            export_dialog_visible: false,
            open_dialog_visible: false,
            psd_dialog_visible: false,
            sample_rate: 1,
            psd_visiable: false,
            signal_plot: SignalPlot::new(),
            signal_path: "".to_owned(),
        };
        ret.setup();
        ret
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "noto".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/NotoSansSC-Regular.ttf")),
        );
        // 中文支持
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("noto".to_owned());
        ctx.set_fonts(fonts);
        // dark mode
        ctx.set_visuals(egui::Visuals::dark());
        Self::default()
    }

    pub fn setup(&mut self) {
        self.menubar.add(MenuItem::new(
            "File",
            &[
                MenuItem::single_with_shortcut(
                    MenuAction::Open,
                    "Open",
                    Modifiers::COMMAND,
                    Key::O,
                ),
                MenuItem::single_with_shortcut(
                    MenuAction::Export,
                    "Export",
                    Modifiers::COMMAND,
                    Key::S,
                ),
                MenuItem::separator(),
                MenuItem::single_with_shortcut(
                    MenuAction::Quit,
                    "Quit",
                    Modifiers::COMMAND,
                    Key::Q,
                ),
            ],
        ));
        self.menubar.add(MenuItem::new(
            "View",
            &[
                MenuItem::single_with_shortcut(
                    MenuAction::Reset,
                    "Reset",
                    Modifiers::COMMAND,
                    Key::R,
                ),
                MenuItem::single_with_shortcut(
                    MenuAction::Return,
                    "Return",
                    Modifiers::COMMAND,
                    Key::ArrowUp,
                ),
                MenuItem::single_with_shortcut(
                    MenuAction::Mag,
                    "Toogle Magnitude",
                    Modifiers::COMMAND,
                    Key::M,
                ),
                MenuItem::separator(),
                MenuItem::single_with_shortcut(MenuAction::Psd, "PSD", Modifiers::COMMAND, Key::P),
            ],
        ));
        self.menubar.add(MenuItem::new(
            "Help",
            &[MenuItem::single(MenuAction::About, "About")],
        ));
    }

    pub fn psd(&mut self) {
        if self.signal_plot.have_signal() {
            self.psd_visiable = true;
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
            let (freqs, psd) = compute_psd(&input, 1024, 0, self.sample_rate as f64);
            self.psd_dialog.set_data(freqs, psd);
            self.psd_dialog_visible = true;
        }
    }

    pub fn export(&self, path: &str) {
        if self.signal_plot.have_signal() {
            let mut data = vec![];
            match self.signal_plot.signal() {
                Signal::Real(sig) => {
                    for s in sig.get(self.signal_plot.range(), 1).iter() {
                        data.push(*s);
                    }
                }
                Signal::Complex(sig) => {
                    for s in sig.get(self.signal_plot.range(), 1).iter() {
                        data.push(s.re);
                        data.push(s.im);
                    }
                }
            }
            unsafe {
                let mut file = File::create(path).unwrap();
                let slice = slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4);
                file.write_all(slice).unwrap();
            }
        }
    }

    pub fn show_menubar(&mut self, ui: &mut egui::Ui) {
        self.menubar.show(ui);
        if let Some(action) = self.menubar.comsume_action(ui) {
            match action {
                &MenuAction::Open => {
                    self.open_dialog_visible = true;
                }
                &MenuAction::Export => {
                    if self.signal_plot.have_signal() {
                        self.export_dialog_visible = true;
                    }
                }
                &MenuAction::Quit => {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                &MenuAction::Reset => {
                    self.signal_plot.reset_view();
                }
                &MenuAction::Return => {
                    self.signal_plot.return_last_view();
                }
                &MenuAction::Psd => {
                    self.psd();
                }
                &MenuAction::Mag => {
                    self.signal_plot.toggle_magnitude();
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_menubar(ui);
        });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.signal_path);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(human_readable_time(self.signal_plot.window_time()));
                    ui.label(format!("{} samples", self.signal_plot.window_samples()));
                    if let Some(freq) = self.signal_plot.measure_frequency() {
                        ui.label(human_readable_freq(freq));
                    }
                })
            });
        });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::BLACK)
                    .inner_margin(5.),
            )
            .show(ctx, |ui| {
                self.signal_plot.show(ui);

                if let Some((sig, sig_mag, path)) =
                    self.open_dialog.show(ctx, &mut self.open_dialog_visible)
                {
                    self.open_dialog_visible = false;
                    self.signal_path = path;
                    self.signal_plot.set_signal(sig, sig_mag);
                    self.signal_plot.reset_view();
                    self.sample_rate = self.open_dialog.sample_rate();
                    self.signal_plot.set_sample_rate(self.sample_rate);
                }

                let export_path = self
                    .export_dialog
                    .show(ctx, &mut self.export_dialog_visible);
                if export_path.is_some() {
                    self.export_dialog_visible = false;
                    self.export(export_path.unwrap().to_str().unwrap());
                }

                self.psd_dialog.show(ctx, &mut self.psd_dialog_visible);
            });
    }
}
