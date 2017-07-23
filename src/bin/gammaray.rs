extern crate gammaray;
use gammaray::core;
use gammaray::prim;
use gammaray::render;
use gammaray::ui;

extern crate image;
use image::ImageBuffer;

extern crate time;

pub fn main() {
    let c = core::Camera::default();
    let s1 = prim::Sphere::new(
        core::Vec::red(),
        prim::Material::new(core::Vec::red(), core::Vec::zero()),
        core::Mat::translation(&core::Vec::new(0.0, 0.0, -100.0)),
        5.0);
    let s2 = prim::Sphere::new(
        core::Vec::one(),
        prim::Material::new(core::Vec::zero(), core::Vec::one()),
        core::Mat::translation(&core::Vec::new(15.0, 0.0, -100.0)),
        5.0);

    let prims: Vec<Box<prim::Prim + Sync>> = vec![Box::new(s1), Box::new(s2)];
    let stage = render::Stage::new(prims);
    let kernel = render::PathTracerKernel::new();

    let height: usize = 512;
    let width = (height as f64 * c.aspect_ratio()) as usize;
    println!("Aspect ratio: {}, Width: {}, Height: {}", c.aspect_ratio(), width, height);
    let mut film = render::Film::new(width, height);

    let start = time::now();
    for _ in 0..100 {
        stage.trace(&c, &kernel, &mut film);
    }
    println!("Duration: {:?}", time::now() - start);

    let img = ImageBuffer::from_fn(width as u32, height as u32, |col, row| {
        let pixel = core::index(row as usize, col as usize, width);
        let val = &film.pixels[pixel].accum / film.pixels[pixel].weight;
        image::Rgba(val.to_rgba8())
    });

    img.save(&std::path::Path::new("out.png")).unwrap();
    ui::image_preview_window(&[&img], width as u32, height as u32);
}
