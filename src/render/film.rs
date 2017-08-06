use core;

use std;
use std::cmp;
use rand;
use rand::distributions::IndependentSample;
use rand::distributions::range::Range;
use rayon::prelude::*;

const FILTER_WIDTH: f64 = 2.0;

#[derive(Clone)]
pub struct FilmSample {
    pub color: core::Vec,
    // Column of the sample, in lens space. Samples may extend beyond [-1, 1] depending on
    // filtering.
    pub s: f64,
    // Row of the sample, in lens space. Samples may extend beyond [-1, 1] depending on filtering.
    pub t: f64,
}

impl FilmSample {
    pub fn zero() -> FilmSample {
        FilmSample {color: core::Vec::zero(), s: 0.0, t: 0.0}
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

        let (widthf, heightf) = (self.width as f64, self.height as f64);
        for row_discr in 0..self.height {
            let row_cont = 0.5 + row_discr as f64;
            for col_discr in 0..self.width {
                let col_cont = 0.5 + col_discr as f64;

                let row_cont_jitter = row_cont + filter_range.ind_sample(&mut thread_rng);
                let col_cont_jitter = col_cont + filter_range.ind_sample(&mut thread_rng);

                let s = core::lerp(-1.0, 1.0, col_cont_jitter / widthf);
                let t = core::lerp(-1.0, 1.0, row_cont_jitter / heightf);
                samples.push(FilmSample {color: core::Vec::zero(), s: s, t: t});
            }
        }
    }

    pub fn report_samples(&mut self, samples: &std::vec::Vec<FilmSample>) {
        let (widthf, heightf) = (self.width as f64, self.height as f64);
        for sample in samples {
            let col_cont = core::lerp(0.0, widthf, 0.5 * (sample.s + 1.0));
            let row_cont = core::lerp(0.0, heightf, 0.5 * (sample.t + 1.0));
            let col_discr = col_cont - 0.5;
            let row_discr = row_cont - 0.5;

            // Note: the min values must be casted to isize first because they may contain negative
            // values. The max values can be casted to usize first because we don't have to deal with
            // negatives.
            let min_col = cmp::max((col_discr - FILTER_WIDTH).ceil() as isize, 0) as usize;
            let max_col = cmp::min((col_discr + FILTER_WIDTH).floor() as usize, self.width - 1);
            let min_row = cmp::max((row_discr - FILTER_WIDTH).ceil() as isize, 0) as usize;
            let max_row = cmp::min((row_discr + FILTER_WIDTH).floor() as usize, self.height - 1);

            for y in (min_row)..(max_row + 1) {
                for x in (min_col)..(max_col + 1) {
                    let mut pixel = &mut self.pixels[core::index(y, x, self.width)];
                    let weight = core::mitchell_filter2(
                        x as f64 - col_discr,
                        y as f64 - row_discr,
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
