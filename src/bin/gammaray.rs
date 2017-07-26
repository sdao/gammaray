extern crate gammaray;
use gammaray::core;
use gammaray::prim;
use gammaray::render;
use gammaray::ui;

use std::thread;

pub fn main() {
    let c = core::Camera::default();
    let s1 = prim::Sphere::new(
        prim::Material::new(core::Vec::new(0.0, 0.5, 1.0), core::Vec::zero()),
        core::Mat::translation(&core::Vec::new(0.0, 0.0, -100.0)),
        5.0);
    let s2 = prim::Sphere::new(
        prim::Material::new(core::Vec::zero(), core::Vec::new(2.0, 2.0, 2.0)),
        core::Mat::translation(&core::Vec::new(15.0, 3.0, -100.0)),
        5.0);
    let s3 = prim::Sphere::new(
        prim::Material::new(core::Vec::new(0.0, 0.5, 0.0), core::Vec::zero()),
        core::Mat::translation(&core::Vec::new(0.0, 0.0, -50.0)),
        100.0);

    let prims: Vec<Box<prim::Prim + Sync + Send>> = vec![Box::new(s1), Box::new(s2), Box::new(s3)];

    let height: usize = 512;
    let width = (height as f64 * c.aspect_ratio()) as usize;
    println!("Aspect ratio: {}, Width: {}, Height: {}", c.aspect_ratio(), width, height);

    let mut stage = render::Stage::new(prims);
    let kernel = render::PathTracerKernel::new();

    let mut film = render::Film::new(width, height);
    let shared_data = ui::SharedData::new(width, height);
    let thread_shared_data = shared_data.clone();
    thread::spawn(move || {
        let mut iter_count = 1usize;
        loop {
            let start = std::time::Instant::now();
            stage.trace(&c, &kernel, &mut film);
            let stop = std::time::Instant::now();

            match thread_shared_data.store() {
                Some(guard) => {
                    film.write_to_rgba8(&mut guard.get());
                },
                None => {}
            }

            let duration = stop - start;
            let secs = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
            println!("Iteration {} [duration: {:.3} sec / {:.3} fps]",
                    iter_count, secs, 1.0 / secs);

            iter_count += 1;
        }
    });

    ui::image_preview_window(shared_data);
}
