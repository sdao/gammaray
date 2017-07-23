use core;

use std;
use rayon::prelude::*;

const FILTER_WIDTH: f64 = 2.0;

#[derive(Clone)]
pub struct FilmSample {
    pub color: core::Vec,
    // Column of the sample, in image space. Can be fractional.
    pub u: f64,
    // Row of the sample, in image space. Can be fractional.
    pub v: f64,
}

impl FilmSample {
    pub fn zero() -> FilmSample {
        FilmSample {color: core::Vec::zero(), u: 0.0, v: 0.0}
    }
}

#[derive(Clone)]
pub struct FilmPixel {
    pub accum: core::Vec,
    pub weight: f64
}

impl FilmPixel {
    pub fn zero() -> FilmPixel {
        FilmPixel {accum: core::Vec::zero(), weight: 0.0}
    }
}

pub struct Film {
    pub width: usize,
    pub height: usize,
    pub samples: std::vec::Vec<FilmSample>,
    pub pixels: std::vec::Vec<FilmPixel>
}

impl Film {
    pub fn new(width: usize, height: usize) -> Film {
        Film {
            width: width,
            height: height,
            samples: vec![FilmSample::zero(); width * height],
            pixels: vec![FilmPixel::zero(); width * height]
        }
    }

    fn commit_samples(&mut self) {
        for sample in &self.samples {
            let u = sample.u;
            let v = sample.v;

            let min_u = core::clamp((u - FILTER_WIDTH).ceil() as usize, 0, self.width - 1);
            let max_u = core::clamp((u + FILTER_WIDTH).floor() as usize, 0, self.width - 1);
            let min_v = core::clamp((v - FILTER_WIDTH).ceil() as usize, 0, self.height - 1);
            let max_v = core::clamp((v + FILTER_WIDTH).floor() as usize, 0, self.height - 1);

            for row in min_v..(max_v + 1) {
                for col in min_u..(max_u + 1) {
                    let mut pixel = &mut self.pixels[core::index(row, col, self.width)];
                    let weight = core::mitchell_filter2(
                        u - col as f64,
                        v - row as f64,
                        FILTER_WIDTH);

                    pixel.accum = &pixel.accum + &(&sample.color * weight);
                    pixel.weight += weight;
                }
            }
        }
    }

    pub fn add_samples(&mut self, sampler: &(Fn(f64, f64) -> core::Vec + Sync)) {
        let width = self.width;

        // XXX: This is wrong. Look at sampling in PBRT.
        let last_col = (self.width - 1) as f64;
        let last_row = (self.height - 1) as f64;

        self.samples.par_iter_mut().enumerate().for_each(|(i, sample)| {
            let (row, col) = core::row_col(i, width);

            let s = core::lerp(-1.0, 1.0, (col as f64 / last_col));
            let t = core::lerp(-1.0, 1.0, (row as f64 / last_row));
            sample.color = sampler(s, t);
            sample.u = col as f64; // XXX: apply jitter to row, col.
            sample.v = row as f64;
        });

        self.commit_samples();
    }
}
