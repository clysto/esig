use crate::series::MultiResolutionSeries;
use eframe::egui::{self, Key, Vec2b};
use egui_plot::{Legend, Line, PlotBounds, PlotPoints};
use rustfft::num_complex::Complex;

pub enum Signal {
    Real(MultiResolutionSeries<f32>),
    Complex(MultiResolutionSeries<Complex<f32>>),
}

pub struct SignalPlot {
    signal: Option<Signal>,
    range: std::ops::Range<usize>,
    first_render: bool,
    reset_view: bool,
    reset_to_last_view: bool,
    zoom_history: Vec<PlotBounds>,
}

fn auto_ratio(max_points: usize, max_ratio: usize, nsamples: usize) -> usize {
    let mut ratio = 1;
    while nsamples / ratio > max_points {
        if ratio << 1 > max_ratio {
            break;
        }
        ratio <<= 1;
    }
    ratio
}

impl SignalPlot {
    pub fn new() -> Self {
        Self {
            signal: None,
            range: 0..0,
            first_render: true,
            reset_view: false,
            reset_to_last_view: false,
            zoom_history: Vec::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let mut z_pressed = false;
        let mut space_pressed = false;
        if ui.ctx().input(|i| i.key_down(Key::Z)) {
            z_pressed = true;
        }
        if ui.ctx().input(|i| i.key_down(Key::Space)) {
            space_pressed = true;
        }
        let max_samples = (ui.available_width() * 2.5) as usize;
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
                if self.first_render {
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max([0., -0.99], [2048., 1.]));
                    self.first_render = false;
                }
                let mut bounds = plot_ui.plot_bounds();
                if self.reset_view {
                    let mut reset_bounds = bounds.clone();
                    reset_bounds.set_y(&PlotBounds::from_min_max([0., -0.99], [2048., 1.]));
                    plot_ui.set_plot_bounds(reset_bounds);
                    self.reset_view = false;
                } else if self.reset_to_last_view {
                    if let Some(bounds) = self.zoom_history.pop() {
                        plot_ui.set_plot_bounds(bounds);
                    }
                    self.reset_to_last_view = false;
                }
                if plot_ui
                    .response()
                    .drag_started_by(egui::PointerButton::Primary)
                {
                    self.zoom_history.push(bounds.clone());
                }
                bounds = plot_ui.plot_bounds();
                let x1 = *bounds.range_x().start();
                let x2 = *bounds.range_x().end();
                let index_start = x1.floor().max(0.) as usize;
                let index_end = x2.ceil() as usize + 1;
                if index_end <= index_start || self.signal.is_none() {
                    return;
                }
                let signal = self.signal.as_ref().unwrap();
                self.range = index_start..index_end;
                match signal {
                    Signal::Real(signal) => {
                        let ratio =
                            auto_ratio(max_samples, signal.max_ratio(), index_end - index_start);
                        let data = signal.get(index_start..index_end, ratio);
                        let re = PlotPoints::new(
                            data.iter()
                                .enumerate()
                                .map(|(i, &y)| [(index_start + i * ratio) as f64, y as f64])
                                .collect(),
                        );
                        plot_ui.line(Line::new(re).name("inphase"));
                    }
                    Signal::Complex(signal) => {
                        let ratio =
                            auto_ratio(max_samples, signal.max_ratio(), index_end - index_start);
                        let data = signal.get(index_start..index_end, ratio);
                        let re = PlotPoints::new(
                            data.iter()
                                .enumerate()
                                .map(|(i, y)| [(index_start + i * ratio) as f64, y.re as f64])
                                .collect(),
                        );
                        let im = PlotPoints::new(
                            data.iter()
                                .enumerate()
                                .map(|(i, y)| [(index_start + i * ratio) as f64, y.im as f64])
                                .collect(),
                        );
                        plot_ui.line(Line::new(re).name("inphase"));
                        plot_ui.line(Line::new(im).name("quadrature"));
                    }
                }
            });
    }

    pub fn set_signal(&mut self, signal: Signal) {
        self.signal = Some(signal);
    }

    pub fn have_signal(&self) -> bool {
        self.signal.is_some()
    }

    pub fn signal(&self) -> &Signal {
        self.signal.as_ref().unwrap()
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.range.clone()
    }

    pub fn reset_view(&mut self) {
        self.reset_view = true;
    }

    pub fn return_last_view(&mut self) {
        self.reset_to_last_view = true;
    }
}
