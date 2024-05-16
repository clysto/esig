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
    let icon = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon)
        .expect("Failed to open icon path")
        .to_rgba8();
    let (icon_width, icon_height) = image.dimensions();
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(eframe::egui::IconData {
                rgba: image.into_raw(),
                width: icon_width,
                height: icon_height,
            })
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
