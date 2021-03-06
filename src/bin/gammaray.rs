extern crate gammaray;
use gammaray::core;
use gammaray::geom;
use gammaray::material;
use gammaray::render;

pub fn main() {
    let c = core::Camera::default();
    let s1 = geom::Mesh::from_obj(
        material::Material::disney()
                .base_color(core::Vec::new(0.0, 0.5, 1.0))
                .roughness(0.5)
                .metallic(1.0)
                .anisotropic(1.0)
                .sheen(1.0)
                .build(),
        &core::Mat::scale(0.9) *
                &core::Mat::translation(&core::Vec::new(-4.0, -5.0, -100.0)),
        "dragon100k_uvs.obj").unwrap();
    let s2 = geom::Sphere::new(
        material::Material::diffuse_light(core::Vec::new(2.0, 2.0, 2.0)),
        core::Mat::translation(&core::Vec::new(12.0, 3.0, -90.0)),
        5.0);
    let s3 = geom::Sphere::new(
        material::Material::disney()
                .base_color(core::Vec::new(0.5, 0.9, 0.0))
                .roughness(0.5)
                .metallic(1.0)
                .build(),
        core::Mat::translation(&core::Vec::new(-25.0, 0.0, -50.0)),
        75.0);
    let s4 = geom::Sphere::new(
        material::Material::disney()
                .base_color(core::Vec::new(1.0, 1.0, 1.0))
                .specular_trans(1.0)
                .roughness(0.2)
                .ior(1.8)
                .metallic(0.0)
                .build(),
        core::Mat::translation(&core::Vec::new(6.0, -10.0, -90.0)),
        4.0);

    let mut prims: Vec<Box<geom::Prim>> = vec![
            Box::new(s1), Box::new(s2), Box::new(s3), Box::new(s4)];
    for i in 0..20usize {
        let color = core::Vec::new(
                if i % 2 == 0 { 0.9 } else { 0.2 },
                if i % 3 == 0 { 0.9 } else { 0.2 },
                if i % 5 == 0 { 0.9 } else { 0.2 }
        );
        prims.push(Box::new(geom::Sphere::new(
            material::Material::disney()
                    .base_color(color)
                    .roughness(0.5)
                    .metallic(1.0)
                    .anisotropic(0.8)
                    .clearcoat(1.0)
                    .clearcoat_gloss(1.0)
                    .build(),
            core::Mat::translation(&core::Vec::new(-20.0 + (i * 4) as f32, 12.0, -100.0)),
            1.8)));
    }
    for i in 0..20usize {
        let color = core::Vec::new(
                if i % 2 == 0 { 0.9 } else { 0.2 },
                if i % 3 == 0 { 0.9 } else { 0.2 },
                if i % 5 == 0 { 0.9 } else { 0.2 }
        );
        prims.push(Box::new(geom::Sphere::new(
            material::Material::disney()
                    .base_color(color)
                    .roughness(0.2)
                    .ior(1.8)
                    .metallic(0.0)
                    .sheen(1.0)
                    .build(),
            core::Mat::translation(&core::Vec::new(-20.0 + (i * 4) as f32, -12.0, -100.0)),
            1.8)));
    }

    let height: usize = 512;
    let width = (height as f32 * c.aspect_ratio()) as usize;
    println!("Aspect ratio: {}, Width: {}, Height: {}", c.aspect_ratio(), width, height);

    let mut stage = render::Stage::new(prims);
    let integrator = render::BdptIntegrator {};

    let mut film = render::Film::new(width, height);
    let mut writer = render::ExrWriter::new("output.exr");
    let mut iter_count = 0usize;
    let mut total_secs = 0.0;
    let limit = 200;
    loop {
        if iter_count == limit {
                return;
        }

        let start = std::time::Instant::now();
        stage.trace(&c, &integrator, &mut film);
        let stop = std::time::Instant::now();

        writer.update(&film);
        writer.write();

        let duration = stop - start;
        let secs = duration.as_secs() as f32 + duration.subsec_nanos() as f32 * 1e-9;
        total_secs += secs;
        println!("Iteration {} [duration: {:.3} sec / {:.3} fps] [total: {:.3} sec]",
                iter_count, secs, 1.0 / secs, total_secs);

        iter_count += 1;
    }
}
