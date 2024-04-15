mod app;
mod fft;
mod open_dialog;
mod sig;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "esig",
        native_options,
        Box::new(|_cc| Box::new(app::App::new())),
    )
}
