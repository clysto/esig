use rayon::prelude::*;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f64::consts::PI;

fn hanning_window(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f64 / (size as f64 - 1.0)).cos()))
        .collect()
}

pub fn compute_psd(
    input: &[Complex<f64>],
    nfft: usize,
    noverlap: usize,
    sample_rate: f64,
) -> (Vec<f64>, Vec<f64>) {
    if input.len() < nfft {
        return compute_psd(
            &input
                .iter()
                .chain(std::iter::repeat(&Complex::new(0.0, 0.0)))
                .take(nfft)
                .copied()
                .collect::<Vec<Complex<f64>>>(),
            nfft,
            noverlap,
            sample_rate,
        );
    }
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(nfft);
    let window = hanning_window(nfft);
    // 使用并行迭代器处理每个段
    let step = nfft - noverlap;
    let psd_sum: Vec<f64> = (0..input.len() - nfft + 1)
        .into_par_iter()
        .step_by(step)
        .map(|start| {
            let end = start + nfft;
            let segment = &input[start..end];
            // 应用窗函数
            let mut windowed_segment: Vec<Complex<f64>> =
                segment.iter().zip(&window).map(|(&s, &w)| s * w).collect();
            // 执行 FFT
            fft.process(&mut windowed_segment);
            // 计算并返回每个段的功率谱
            windowed_segment
                .iter()
                .map(|&x| x.norm_sqr() / (window.iter().sum::<f64>() * window.iter().sum::<f64>()))
                .collect::<Vec<f64>>()
        })
        .reduce_with(|mut a, b| {
            for (ai, bi) in a.iter_mut().zip(b.iter()) {
                *ai += *bi;
            }
            a
        })
        .unwrap_or_else(|| vec![0.0; nfft]);
    // 计算平均功率谱
    let count = ((input.len() - nfft + 1) as f64 / step as f64).floor();
    let psd: Vec<f64> = psd_sum.into_iter().map(|x| x / count).collect();
    let freqs: Vec<f64> = (0..nfft)
        .map(|i| {
            let freq = i as f64 * sample_rate / nfft as f64;
            if i < nfft / 2 {
                freq
            } else {
                freq - sample_rate
            }
        })
        .collect();
    // 换算成对数刻度
    let log_psd: Vec<f64> = psd.iter().map(|&x| 10.0 * x.log10()).collect();
    let half_nfft = nfft / 2;
    let (first_half_freqs, second_half_freqs) = freqs.split_at(half_nfft);
    let shifted_freqs: Vec<f64> = second_half_freqs
        .iter()
        .chain(first_half_freqs.iter())
        .copied()
        .collect();
    let (first_half_psd, second_half_psd) = log_psd.split_at(half_nfft);
    let shifted_psd: Vec<f64> = second_half_psd
        .iter()
        .chain(first_half_psd.iter())
        .copied()
        .collect();
    (shifted_freqs, shifted_psd)
}
