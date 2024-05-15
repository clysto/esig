use eframe::egui;
use egui_plot::{Line, PlotPoints};
use emath::vec2;

pub struct PsdDialog {
    freqs: Vec<f64>,
    psd: Vec<f64>,
}

impl Default for PsdDialog {
    fn default() -> Self {
        Self {
            freqs: Vec::new(),
            psd: Vec::new(),
        }
    }
}

impl PsdDialog {
    pub fn show(&self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new("PSD")
            .open(open)
            .resizable(true)
            .min_size([400.0, 250.0])
            .default_size([400.0, 250.0])
            .show(ctx, |ui| {
                egui_plot::Plot::new("psd")
                    .set_margin_fraction(vec2(0., 0.1))
                    .x_axis_label("Frequency")
                    .y_axis_label("Power Spectral Density (dB)")
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
    }

    pub fn set_data(&mut self, freqs: Vec<f64>, psd: Vec<f64>) {
        self.freqs = freqs;
        self.psd = psd;
    }
}
