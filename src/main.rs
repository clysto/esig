#![windows_subsystem = "windows"]

mod app;
mod export_dialog;
mod fft;
mod menubar;
mod open_dialog;
mod psd_dialog;
mod series;
mod signal_plot;
mod utils;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "esig",
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}
