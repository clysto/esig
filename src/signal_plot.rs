use crate::series::MultiResolutionSeries;
use eframe::egui::{self, Color32, Key, Vec2b};
use egui_plot::{Legend, Line, PlotBounds, PlotPoints};
use rustfft::num_complex::Complex;

pub enum Signal {
    Real(MultiResolutionSeries<f32>),
    Complex(MultiResolutionSeries<Complex<f32>>),
}

pub struct SignalPlot {
    signal: Option<Signal>,
    signal_mag: Option<Signal>,
    range: std::ops::Range<usize>,
    first_render: bool,
    reset_view: bool,
    reset_to_last_view: bool,
    x_axis_time: bool,
    sample_rate: u32,
    zoom_history: Vec<PlotBounds>,
    bounds: PlotBounds,
    magnitude_visible: bool,
    measure_active: bool,
    measure_x1: Option<f64>,
    measure_x2: Option<f64>,
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
            signal_mag: None,
            range: 0..0,
            first_render: true,
            reset_view: false,
            reset_to_last_view: false,
            x_axis_time: true,
            sample_rate: 1,
            zoom_history: Vec::new(),
            bounds: PlotBounds::from_min_max([0., 0.], [0., 0.]),
            magnitude_visible: false,
            measure_active: false,
            measure_x1: None,
            measure_x2: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let mut z_pressed = false;
        let mut r_pressed = false;
        let mut space_pressed = false;
        if ui.ctx().input(|i| i.key_down(Key::Z)) {
            z_pressed = true;
        }
        if ui.ctx().input(|i| i.key_down(Key::R)) {
            r_pressed = true;
        }
        if ui.ctx().input(|i| i.key_down(Key::Space)) {
            space_pressed = true;
        }
        let max_samples = (ui.available_width() * 2.5) as usize;
        let x_label = if self.x_axis_time {
            "Time (s)"
        } else {
            "Samples"
        };
        let sample_rate = self.sample_rate;
        let x_axis_time = self.x_axis_time;
        let mut measure_x1_pos = egui::pos2(0., 0.);
        let mut measure_x2_pos = egui::pos2(0., 0.);
        let response = egui_plot::Plot::new("signal")
            .legend(Legend::default())
            .auto_bounds(Vec2b::new(false, false))
            .allow_double_click_reset(false)
            .allow_zoom(Vec2b::new(!z_pressed, z_pressed))
            .allow_drag(space_pressed && !r_pressed)
            .allow_boxed_zoom(!space_pressed && !r_pressed)
            .boxed_zoom_pointer_button(egui::PointerButton::Primary)
            .x_axis_formatter(move |mark, _range| {
                if x_axis_time {
                    let time = mark.value as f64 / sample_rate as f64;
                    format!("{}", time)
                } else {
                    format!("{}", mark.value)
                }
            })
            .x_axis_label(x_label)
            .show(ui, |plot_ui| {
                if self.first_render {
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max([0., -0.99], [1000., 1.]));
                    self.first_render = false;
                }

                let mut bounds = plot_ui.plot_bounds();
                if *bounds.range_y().start() < -9999999. {
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                        [*bounds.range_x().start(), -9999999.],
                        [*bounds.range_x().end(), *bounds.range_y().end()],
                    ));
                }
                if *bounds.range_y().end() > 9999999. {
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                        [*bounds.range_x().start(), *bounds.range_y().start()],
                        [*bounds.range_x().end(), 9999999.],
                    ));
                }
                bounds = plot_ui.plot_bounds();

                if self.reset_view {
                    if let Some(sig) = self.signal.as_ref() {
                        match sig {
                            Signal::Real(sig) => {
                                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                    [0., -0.99],
                                    [sig.len() as f64, 1.],
                                ));
                            }
                            Signal::Complex(sig) => {
                                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                    [0., -0.99],
                                    [sig.len() as f64, 1.],
                                ));
                            }
                        }
                    } else {
                        plot_ui.set_plot_bounds(PlotBounds::from_min_max([0., -0.99], [1000., 1.]));
                    }
                    self.reset_view = false;
                } else if self.reset_to_last_view {
                    if let Some(bounds) = self.zoom_history.pop() {
                        plot_ui.set_plot_bounds(bounds);
                    }
                    self.reset_to_last_view = false;
                }
                if plot_ui.response().clicked_by(egui::PointerButton::Primary) && r_pressed {
                    self.measure_active = false;
                    self.measure_x1 = None;
                    self.measure_x2 = None;
                }
                if plot_ui
                    .response()
                    .drag_stopped_by(egui::PointerButton::Primary)
                    && r_pressed
                {
                    self.measure_active = false;
                }
                if plot_ui
                    .response()
                    .drag_started_by(egui::PointerButton::Primary)
                {
                    self.zoom_history.push(bounds.clone());
                    if r_pressed {
                        self.measure_active = true;
                        self.measure_x1 = plot_ui.pointer_coordinate().map(|p| p.x);
                    }
                }

                if self.measure_active && self.measure_x1.is_some() {
                    self.measure_x2 = plot_ui.pointer_coordinate().map(|p| p.x);
                }
                if self.measure_x1.is_some() && self.measure_x2.is_some() {
                    let x1 = self.measure_x1.unwrap();
                    let x2 = self.measure_x2.unwrap();
                    let x_min = x1.min(x2);
                    let x_max = x1.max(x2);
                    measure_x1_pos = plot_ui.screen_from_plot(egui_plot::PlotPoint::new(x_min, 0.));
                    measure_x2_pos = plot_ui.screen_from_plot(egui_plot::PlotPoint::new(x_max, 0.));
                }
                bounds = plot_ui.plot_bounds();
                self.bounds = bounds.clone();
                if self.signal.is_none() {
                    return;
                }
                let signal = self.signal.as_ref().unwrap();

                let x1 = *bounds.range_x().start();
                let x2 = *bounds.range_x().end();
                match signal {
                    Signal::Real(signal) => {
                        let index_start = x1.floor().max(0.) as usize;
                        let index_end = x2.ceil().min(signal.len() as f64) as usize + 1;
                        if index_end <= index_start {
                            return;
                        }
                        self.range = index_start..index_end;
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
                        let index_start = x1.floor().max(0.) as usize;
                        let index_end = x2.ceil().min(signal.len() as f64) as usize + 1;
                        if index_end <= index_start {
                            return;
                        }
                        self.range = index_start..index_end;
                        let ratio =
                            auto_ratio(max_samples, signal.max_ratio(), index_end - index_start);
                        if self.magnitude_visible {
                            if let Signal::Real(signal_mag) = self.signal_mag.as_ref().unwrap() {
                                let data = signal_mag.get(index_start..index_end, ratio);
                                let mag = PlotPoints::new(
                                    data.iter()
                                        .enumerate()
                                        .map(|(i, &y)| [(index_start + i * ratio) as f64, y as f64])
                                        .collect(),
                                );
                                plot_ui.line(Line::new(mag).name("magnitude"));
                            }
                        } else {
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
                }
            });
        let rect = response.response.rect;
        if self.measure_x1.is_some() && self.measure_x2.is_some() {
            ui.painter().vline(
                measure_x1_pos.x,
                rect.y_range(),
                egui::Stroke::new(2., Color32::WHITE),
            );
            ui.painter().vline(
                measure_x2_pos.x,
                rect.y_range(),
                egui::Stroke::new(2., Color32::WHITE),
            );
            let left_mask = egui::Rect::from_x_y_ranges(
                egui::Rangef::new(rect.min.x, measure_x1_pos.x),
                rect.y_range(),
            );
            let right_mask = egui::Rect::from_x_y_ranges(
                egui::Rangef::new(measure_x2_pos.x, rect.max.x),
                rect.y_range(),
            );
            ui.painter().rect_filled(left_mask, 0., Color32::from_black_alpha(150));
            ui.painter().rect_filled(right_mask, 0., Color32::from_black_alpha(150));
        }
    }

    pub fn set_signal(&mut self, signal: Signal, signal_mag: Option<Signal>) {
        self.measure_x1 = None;
        self.measure_x2 = None;
        self.zoom_history.clear();
        self.signal = Some(signal);
        self.signal_mag = signal_mag;
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

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    pub fn window_time(&self) -> f64 {
        let x1;
        let x2;
        if self.measure_x1.is_some() && self.measure_x2.is_some() {
            x1 = self.measure_x1.unwrap();
            x2 = self.measure_x2.unwrap();
        } else {
            let range_x = self.bounds.range_x();
            x1 = *range_x.start();
            x2 = *range_x.end();
        }
        (x2 - x1).abs() / self.sample_rate as f64
    }

    pub fn window_samples(&self) -> usize {
        if self.signal.is_none() {
            return 0;
        }
        let x1;
        let x2;
        if let (Some(measure_x1), Some(measure_x2)) = (self.measure_x1, self.measure_x2) {
            x1 = if measure_x1 < measure_x2 {
                measure_x1.ceil() as usize
            } else {
                measure_x2.ceil() as usize
            };
            x2 = if measure_x1 < measure_x2 {
                measure_x2.floor() as usize
            } else {
                measure_x1.floor() as usize
            };
        } else {
            let range_x = self.bounds.range_x();
            x1 = range_x.start().ceil() as usize;
            x2 = range_x.end().floor() as usize;
        }
        let mut index_start = self.range.start;
        let mut index_end = self.range.end;
        while index_start < x1 {
            index_start += 1;
        }
        while index_end > x2 {
            index_end -= 1;
        }
        if index_end < index_start {
            return 0;
        }
        index_end + 1 - index_start
    }

    pub fn toggle_magnitude(&mut self) {
        self.magnitude_visible = !self.magnitude_visible;
    }

    pub fn measure_frequency(&self) -> Option<f64> {
        if self.measure_x1.is_some() && self.measure_x2.is_some() {
            return Some(1. / self.window_time());
        }
        None
    }
}
