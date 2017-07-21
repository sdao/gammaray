extern crate gammaray;
use gammaray::core;
use gammaray::prim;
use gammaray::prim::Prim;
use gammaray::render;
use gammaray::ui;

extern crate image;
use image::ImageBuffer;

extern crate time;

pub fn main() {
    let c = core::Camera::default();
    let s1 = prim::Sphere::new(
        &core::Vec::red(),
        &core::Mat::translation(&core::Vec::new(0.0, 0.0, -100.0)),
        5.0);
    let s2 = prim::Sphere::new(
        &core::Vec::green(),
        &core::Mat::translation(&core::Vec::new(15.0, 0.0, -100.0)),
        5.0);

    let prims: Vec<Box<Prim + Sync>> = vec![Box::new(s1), Box::new(s2)];
    let stage = render::Stage::new(prims);
    let kernel = render::DisplayColorKernel::new();

    let height: usize = 512;
    let width = (height as f64 * c.aspect_ratio()) as usize;
    let num_pixels = width * height;

    let mut data = vec![render::Sample::zero(); num_pixels];

    let start = time::now();
    stage.trace(&c, width, height, &kernel, &mut data);
    println!("Duration: {:?}", time::now() - start);

    let img = ImageBuffer::from_fn(width as u32, height as u32, |col, row| {
        let pixel = core::index(row as usize, col as usize, width);
        image::Rgba(data[pixel].accum.to_rgba8())
    });

    ui::image_preview_window(&[&img], width as u32, height as u32);
}
