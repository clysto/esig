mod app;
mod fft;
mod open_dialog;
mod signal_plot;
mod psd_dialog;
mod series;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "esig",
        native_options,
        Box::new(|_cc| Box::new(app::App::new())),
    )
}
