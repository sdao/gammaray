extern crate gammaray;
use gammaray::core;
use gammaray::prim;
use gammaray::prim::Prim;
use gammaray::ui;

extern crate image;
use image::ImageBuffer;

extern crate time;

extern crate rayon;
use rayon::prelude::*;
use rayon::iter::ParallelIterator;

pub fn main() {
    let c = core::Camera::default();
    let xform_s = core::Mat::translation(&core::Vec::new(0.0, 0.0, -100.0));
    let s = prim::Sphere::new(&xform_s, 5.0);

    let height: usize = 512;
    let width = (height as f64 * c.aspect_ratio()) as usize;
    let num_pixels = width * height;
    let mut data = vec![false; num_pixels];

    let start = time::now();
    (0..num_pixels).into_par_iter().with_min_len(width).map(|i| {
        let (row, col) = core::row_col(i, width);

        let i = core::lerp(-1.0, 1.0, (col as f64 / (width - 1) as f64));
        let j = core::lerp(-1.0, 1.0, (row as f64 / (height - 1) as f64));
        let r = c.compute_ray(i, j);
        match s.intersect_world(&r) {
            Some(_) => true,
            None => false,
        }
    }).collect_into(&mut data);
    println!("Duration: {:?}", time::now() - start);

    let img = ImageBuffer::from_fn(width as u32, height as u32, |col, row| {
        let pixel = core::index(row as usize, col as usize, width);
        match data[pixel] {
            true => image::Rgba([255u8, 255u8, 255u8, 255u8]),
            false => image::Rgba([0u8, 0u8, 0u8, 255u8]),
        }
    });

    ui::image_preview_window(&[&img], width as u32, height as u32);
}
