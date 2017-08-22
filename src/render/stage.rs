use render::film;
use render::integrators;

use core;
use geom;

use std;
use rand;
use rand::{Rng, SeedableRng};
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
            let mut thread_rng = rand::thread_rng();
            let mut rng = rand::XorShiftRng::from_seed([
                    thread_rng.next_u32(),
                    thread_rng.next_u32(),
                    thread_rng.next_u32(),
                    thread_rng.next_u32()]);
            sample.color = integrator.integrate(&ray, bvh, &mut rng);
        });
        film.report_samples(&self.sample_storage);
    }
}
