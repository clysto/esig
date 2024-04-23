use crate::export_dialog::ExportDialog;
use crate::fft::compute_psd;
use crate::menubar::{MenuBar, MenuItem};
use crate::open_dialog::OpenDialog;
use crate::psd_dialog::PsdDialog;
use crate::signal_plot::{Signal, SignalPlot};
use eframe::egui::{self, Key, Modifiers};
use rustfft::num_complex::Complex;
use std::fs::File;
use std::io::Write;
use std::slice;

pub struct App {
    open_dialog: OpenDialog,
    menubar: MenuBar,
    psd_dialog: PsdDialog,
    export_dialog: ExportDialog,
    export_dialog_visible: bool,
    open_dialog_visible: bool,
    psd_dialog_visible: bool,
    sample_rate: u32,
    psd_visiable: bool,
    signal_plot: SignalPlot,
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
        };
        ret.setup();
        ret
    }
}

impl App {
    const OPEN: u32 = 0;
    const EXPORT: u32 = 1;
    const QUIT: u32 = 2;
    const RESET: u32 = 3;
    const RETURN: u32 = 4;
    const PSD: u32 = 5;
    const ABOUT: u32 = 6;

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
        Self::default()
    }

    pub fn setup(&mut self) {
        self.menubar.add(MenuItem::new(
            "File",
            &[
                MenuItem::single_with_shortcut(Self::OPEN, "Open", Modifiers::COMMAND, Key::O),
                MenuItem::single_with_shortcut(Self::EXPORT, "Export", Modifiers::COMMAND, Key::S),
                MenuItem::separator(),
                MenuItem::single_with_shortcut(Self::QUIT, "Quit", Modifiers::COMMAND, Key::Q),
            ],
        ));
        self.menubar.add(MenuItem::new(
            "View",
            &[
                MenuItem::single(Self::RESET, "Reset"),
                MenuItem::single(Self::RETURN, "Return"),
                MenuItem::single_with_shortcut(Self::PSD, "PSD", Modifiers::COMMAND, Key::P),
            ],
        ));
        self.menubar.add(MenuItem::new(
            "Help",
            &[MenuItem::single(Self::ABOUT, "About")],
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
                let slice =
                    slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4);
                file.write_all(slice).unwrap();
            }
        }
    }

    pub fn show_menubar(&mut self, ui: &mut egui::Ui) {
        self.menubar.show(ui);
        if let Some(action) = self.menubar.comsume_action(ui) {
            match action {
                &Self::OPEN => {
                    self.open_dialog_visible = true;
                }
                &Self::EXPORT => {
                    if self.signal_plot.have_signal() {
                        self.export_dialog_visible = true;
                    }
                }
                &Self::QUIT => {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                &Self::RESET => {
                    self.signal_plot.reset_view();
                }
                &Self::RETURN => {
                    self.signal_plot.return_last_view();
                }
                &Self::PSD => {
                    self.psd();
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

        egui::CentralPanel::default().show(ctx, |ui| {
            self.signal_plot.show(ui);

            let sig = self.open_dialog.show(ctx, &mut self.open_dialog_visible);
            if sig.is_some() {
                self.open_dialog_visible = false;
                self.signal_plot.set_signal(sig.unwrap());
                self.signal_plot.reset_view();
                self.sample_rate = self.open_dialog.sample_rate();
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
