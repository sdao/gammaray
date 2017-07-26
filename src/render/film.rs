use core;

use std;
use rand;
use rand::distributions::IndependentSample;
use rand::distributions::range::Range;
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

#[derive(Clone, Copy)]
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

    pub fn compute_sample_points(&self, samples: &mut std::vec::Vec<FilmSample>) {
        let mut thread_rng = rand::thread_rng();
        let filter_range = Range::new(-FILTER_WIDTH, FILTER_WIDTH);

        samples.clear();
        samples.reserve_exact(self.width * self.height);
        for row in 0..self.height {
            for col in 0..self.width {
                // XXX: This is wrong. Look at sampling in PBRT.
                let last_col = (self.width - 1) as f64;
                let last_row = (self.height - 1) as f64;

                let c = col as f64 + filter_range.ind_sample(&mut thread_rng);
                let r = row as f64 + filter_range.ind_sample(&mut thread_rng);

                let s = core::lerp(-1.0, 1.0, (c / last_col));
                let t = core::lerp(-1.0, 1.0, (r / last_row));
                samples.push(FilmSample {color: core::Vec::zero(), u: s, v: t});
            }
        }
    }

    pub fn commit_samples(&mut self, samples: &std::vec::Vec<FilmSample>) {
        for sample in samples {
            let uu = sample.u;
            let vv = sample.v;

            // XXX: This is wrong. Look at sampling in PBRT.
            let last_col = (self.width - 1) as f64;
            let last_row = (self.height - 1) as f64;
            let u = core::lerp(0.0, last_col, 0.5 * (uu + 1.0));
            let v = core::lerp(0.0, last_row, 0.5 * (vv + 1.0));

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

    pub fn write_to_rgba8(&mut self, rgba8: &mut std::vec::Vec<[u8; 4]>) {
        assert!(self.pixels.len() == rgba8.len());

        rgba8.par_iter_mut().enumerate().for_each(|(i, rgba8_pixel)| {
            let val = (&self.pixels[i].accum / self.pixels[i].weight).to_rgba8();
            rgba8_pixel.copy_from_slice(&val);
        });
    }
}
