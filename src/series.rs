use rayon::prelude::*;
use rustfft::num_complex::Complex;

pub trait Downconvert<T> {
    fn minmax_downconvert(src: &[T], n: usize) -> Vec<T>;
}

impl Downconvert<f32> for f32 {
    fn minmax_downconvert(src: &[f32], n: usize) -> Vec<f32> {
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
}

impl Downconvert<Complex<f32>> for Complex<f32> {
    fn minmax_downconvert(src: &[Complex<f32>], n: usize) -> Vec<Complex<f32>> {
        let mut dst;
        if src.len() / n & 1 == 1 {
            dst = vec![Complex::default(); src.len() / n + 1];
        } else {
            dst = vec![Complex::default(); src.len() / n];
        }
        dst.par_chunks_mut(2)
            .zip(src.par_chunks(2 * n))
            .for_each(|(dst_chunk, src_chunk)| {
                let mut re_max = f32::NEG_INFINITY;
                let mut re_min = f32::INFINITY;
                let mut im_max = f32::NEG_INFINITY;
                let mut im_min = f32::INFINITY;
                for i in 0..src_chunk.len() {
                    re_max = re_max.max(src_chunk[i].re);
                    re_min = re_min.min(src_chunk[i].re);
                    im_max = im_max.max(src_chunk[i].im);
                    im_min = im_min.min(src_chunk[i].im);
                }
                dst_chunk[0] = Complex::new(re_max, im_max);
                dst_chunk[1] = Complex::new(re_min, im_min);
            });
        dst
    }
}

pub struct MultiResolutionSeries<T> {
    data: Vec<Vec<T>>,
}

impl<T: Downconvert<T> + Clone> MultiResolutionSeries<T> {
    pub fn build(data: &[T], min_len: usize) -> Self {
        let mut s = Self {
            data: vec![data.to_vec()],
        };
        let mut ratio = 2;
        while data.len() / ratio > min_len {
            s.data.push(T::minmax_downconvert(s.data.last().unwrap(), 2));
            println!("x{}", ratio);
            ratio <<= 1;
        }
        s
    }

    pub fn get(&self, range: std::ops::Range<usize>, ratio: usize) -> &[T] {
        assert!((ratio & (ratio - 1)) == 0);
        let start = range.start / ratio;
        let end = (range.end / ratio).min(self.data[0].len() / ratio);
        &self.data[ratio.trailing_zeros() as usize][start..end]
    }

    pub fn max_ratio(&self) -> usize {
        1 << (self.data.len() - 1)
    }
}
