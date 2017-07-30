use render::film;
use render::kernel;

use core::Camera;
use core::Ray;
use core::Vec;

use geom::Bvh;
use geom::Intersection;
use geom::Prim;

use std;
use rand;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;

pub struct Stage {
    bvh: Bvh,
    sample_storage: std::vec::Vec<film::FilmSample>
}

impl Stage {
    pub fn new(prims: std::vec::Vec<Box<Prim + Sync + Send>>) -> Stage {
        Stage {
            bvh: Bvh::build(prims),
            sample_storage: vec![]
        }
    }

    pub fn trace_single_ray(initial_ray: &Ray, bvh: &Bvh, kernel: &kernel::Kernel) -> Vec {
        let mut thread_rng = rand::thread_rng();
        let mut rng = rand::XorShiftRng::from_seed([
                thread_rng.next_u32(),
                thread_rng.next_u32(),
                thread_rng.next_u32(),
                thread_rng.next_u32()]);

        let mut depth = 0usize;
        let mut throughput: Vec = Vec::one();
        let mut light: Vec = Vec::zero();
        let mut current_ray: Ray = initial_ray.clone();
        while !throughput.is_exactly_zero() {
            let intersection = bvh.intersect(&current_ray);
            match intersection {
                Intersection::Hit {dist, normal, prim} => {
                    let kernel_result = kernel.bounce(
                            depth, &current_ray.direction, &normal, &prim, &mut rng);
                    light = &light + &throughput.comp_mult(&kernel_result.light);
                    throughput = throughput.comp_mult(&kernel_result.throughput);
                    current_ray = Ray::new(
                            &current_ray.at(dist) + &(&kernel_result.direction * 1e-6),
                            kernel_result.direction);
                },
                Intersection::NoHit => {
                    throughput = Vec::zero();
                    current_ray = Ray::zero();
                }
            }

            depth += 1;
        }

        light
    }

    pub fn trace(&mut self,
        camera: &Camera,
        kernel: &(kernel::Kernel + Sync + Send),
        film: &mut film::Film)
    {
        film.compute_sample_points(&mut self.sample_storage);
        let bvh = &self.bvh;
        self.sample_storage.par_iter_mut().for_each(|sample| {
            let ray = camera.compute_ray(sample.s, sample.t);
            sample.color = Stage::trace_single_ray(&ray, bvh, kernel);
        });
        film.report_samples(&self.sample_storage);
    }
}
