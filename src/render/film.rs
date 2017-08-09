use core;

use std;
use rand;
use rand::distributions::IndependentSample;
use rand::distributions::range::Range;

const FILTER_WIDTH: f32 = 2.0;

#[derive(Clone)]
pub struct FilmSample {
    pub color: core::Vec,
    // Column of the sample, in lens space. Samples may extend beyond [-1, 1] depending on
    // filtering.
    pub s: f32,
    // Row of the sample, in lens space. Samples may extend beyond [-1, 1] depending on filtering.
    pub t: f32,
}

impl FilmSample {
    pub fn zero() -> FilmSample {
        FilmSample {color: core::Vec::zero(), s: 0.0, t: 0.0}
    }
}

#[derive(Clone, Copy)]
pub struct FilmPixel {
    pub accum: core::Vec,
    pub weight: f32
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

        let (widthf, heightf) = (self.width as f32, self.height as f32);
        for row_discr in 0..self.height {
            let row_cont = 0.5 + row_discr as f32;
            for col_discr in 0..self.width {
                let col_cont = 0.5 + col_discr as f32;

                let row_cont_jitter = row_cont + filter_range.ind_sample(&mut thread_rng);
                let col_cont_jitter = col_cont + filter_range.ind_sample(&mut thread_rng);

                let s = core::lerp(-1.0, 1.0, col_cont_jitter / widthf);
                let t = core::lerp(-1.0, 1.0, row_cont_jitter / heightf);
                samples.push(FilmSample {color: core::Vec::zero(), s: s, t: t});
            }
        }
    }

    pub fn report_samples(&mut self, samples: &std::vec::Vec<FilmSample>) {
        let (widthf, heightf) = (self.width as f32, self.height as f32);
        let (last_col, last_row) = (self.width as isize - 1, self.height as isize - 1);
        for sample in samples {
            let col_cont = core::lerp(0.0, widthf, 0.5 * (sample.s + 1.0));
            let row_cont = core::lerp(0.0, heightf, 0.5 * (sample.t + 1.0));
            let col_discr = col_cont - 0.5;
            let row_discr = row_cont - 0.5;

            // Note: the min values must be casted to isize first because they may contain negative
            // values. The max values can be casted to usize first because we don't have to deal
            // with negatives.
            let min_col = core::clamp(
                    (col_discr - FILTER_WIDTH).ceil() as isize, 0, last_col) as usize;
            let max_col = core::clamp(
                    (col_discr + FILTER_WIDTH).floor() as isize, 0, last_col) as usize;
            let min_row = core::clamp(
                    (row_discr - FILTER_WIDTH).ceil() as isize, 0, last_row) as usize;
            let max_row = core::clamp(
                    (row_discr + FILTER_WIDTH).floor() as isize, 0, last_row) as usize;
            debug_assert!(max_row < self.height, "max_row {} >= {}", max_row, self.height);
            debug_assert!(max_col < self.width, "max_col {} >= {}", max_col, self.width);

            for y in (min_row)..(max_row + 1) {
                for x in (min_col)..(max_col + 1) {
                    let mut pixel = &mut self.pixels[core::index(y, x, self.width)];
                    let weight = core::mitchell_filter2(
                            x as f32 - col_discr,
                            y as f32 - row_discr,
                            FILTER_WIDTH);

                    pixel.accum = &pixel.accum + &(&sample.color * weight);
                    pixel.weight += weight;
                }
            }
        }
    }
}
