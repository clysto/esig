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
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("ESig"),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "esig",
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}
