use render::film;
use render::kernel;

use core::Camera;
use core::Ray;
use core::Vec;

use prim::Prim;

use std;
use rand;
use rand::{Rng, SeedableRng};

enum Intersection<'a> {
    Hit {
        dist: f64,
        normal: Vec,
        prim: &'a Box<Prim + Sync>
    },
    NoHit
}

impl<'a> Intersection<'a> {
    pub fn hit(dist: f64, normal: Vec, prim: &'a Box<Prim + Sync>) -> Intersection<'a> {
        Intersection::Hit {dist: dist, normal: normal, prim: prim}
    }

    pub fn no_hit() -> Intersection<'a> {
        Intersection::NoHit
    }
}

pub struct Stage {
    pub prims: std::vec::Vec<Box<Prim + Sync>>
}

impl Stage {
    pub fn new(prims: std::vec::Vec<Box<Prim + Sync>>) -> Stage {
        Stage {prims: prims}
    }

    fn intersect_world(&self, ray: &Ray) -> Intersection {
        let mut closest_dist = std::f64::MAX;
        let mut closest: Intersection = Intersection::no_hit();
        for prim in &self.prims {
            let (dist, normal) = prim.intersect_world(&ray);
            if dist != 0.0 && dist < closest_dist {
                closest = Intersection::hit(dist, normal, prim);
                closest_dist = dist;
            }
        }
        closest
    }

    fn trace_single_ray(&self, initial_ray: &Ray, kernel: &kernel::Kernel) -> Vec {
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
        while !current_ray.direction.is_exactly_zero() {
            let intersection = self.intersect_world(&current_ray);
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

    pub fn trace(&self,
        camera: &Camera,
        kernel: &(kernel::Kernel + Sync),
        film: &mut film::Film)
    {
        film.add_samples(&|u, v| {
            let ray = camera.compute_ray(u, v);
            self.trace_single_ray(&ray, kernel)
        });
    }
}
