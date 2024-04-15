use egui_plot::PlotPoints;
use rayon::prelude::*;
use std::collections::HashMap;

pub fn peak_downconvert(src: &[f32], n: usize) -> Vec<f32> {
    let mut dst;
    if src.len() / n & 1 == 1 {
        dst = vec![0.0; src.len() / n + 1];
    } else {
        dst = vec![0.0; src.len() / n];
    }
    dst.par_chunks_mut(2)
        .zip(src.par_chunks(2 * n))
        .for_each(|(dst_chunk, src_chunk)| {
            let max = src_chunk.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let min = src_chunk.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            dst_chunk[0] = max;
            dst_chunk[1] = min;
        });
    dst
}

pub struct MultiResolutionSignal {
    data: HashMap<usize, Vec<f32>>,
    max_points: usize,
}

impl MultiResolutionSignal {
    pub fn new(data: &[f32], max_points: usize) -> Self {
        let mut sig = Self {
            data: HashMap::new(),
            max_points,
        };
        sig.data.insert(1, data.to_vec());
        let mut ratio = 2;
        while data.len() / ratio > max_points {
            let sig_down = peak_downconvert(data, ratio);
            println!("x{}: {}", ratio, sig_down.len());
            sig.data.insert(ratio as usize, sig_down);
            ratio <<= 1;
        }
        sig
    }

    fn auto_ratio(&self, n_samples: usize) -> usize {
        let mut ratio = 1;
        while n_samples / ratio > self.max_points {
            if self.data.get(&(ratio << 1 as usize)).is_none() {
                break;
            }
            ratio <<= 1;
        }
        ratio
    }

    pub fn points(&self, start_index: usize, end_index: usize) -> PlotPoints {
        let start_index = std::cmp::max(start_index, 0);
        let end_index = std::cmp::min(end_index, self.data.get(&1).unwrap().len());
        let ratio = self.auto_ratio(end_index - start_index);
        let sig = self.data.get(&(ratio as usize)).unwrap();
        PlotPoints::new(
            sig[start_index / ratio..end_index / ratio]
                .iter()
                .enumerate()
                .map(|(i, &y)| [(start_index + i * ratio) as f64, y as f64])
                .collect(),
        )
    }

    pub fn get(&self, start_index: usize, end_index: usize) -> Vec<f32> {
        let start_index = std::cmp::max(start_index, 0);
        let end_index = std::cmp::min(end_index, self.data.get(&1).unwrap().len());
        self.data.get(&1).unwrap()[start_index..end_index].to_vec()
    }
}
