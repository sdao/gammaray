use render::film;
use render::integrators;

use core;
use geom;

use std;
use rayon::prelude::*;

pub struct Stage {
    bvh: geom::Bvh,
    sample_storage: std::vec::Vec<film::FilmSample>
}

impl Stage {
    pub fn new(prims: std::vec::Vec<Box<geom::Prim>>) -> Stage {
        Stage {
            bvh: geom::Bvh::build(prims),
            sample_storage: vec![]
        }
    }

    pub fn trace(&mut self,
        camera: &core::Camera,
        integrator: &integrators::Integrator,
        film: &mut film::Film)
    {
        film.compute_sample_points(&mut self.sample_storage);
        let bvh = &self.bvh;
        self.sample_storage.par_iter_mut().for_each(|sample| {
            let ray = camera.compute_ray(sample.s, sample.t);
            let mut rng = core::new_xor_shift_rng();
            sample.color = integrator.integrate(&ray, bvh, &mut rng);
        });
        film.report_samples(&self.sample_storage);
    }
}
