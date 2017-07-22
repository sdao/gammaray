use render::film;
use render::kernel;

use core::Camera;
use core::Ray;
use core::Vec;

use std;
use prim::Prim;

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
        let mut color: Vec = Vec::one();
        let mut current_ray: Ray = initial_ray.clone();
        while !current_ray.direction.is_exactly_zero() {
            let intersection = self.intersect_world(&current_ray);
            match intersection {
                Intersection::Hit {dist, normal, prim} => {
                    let pt = current_ray.at(dist);
                    let kernel_result = kernel.bounce(dist, &normal, &prim);
                    color = color.comp_mult(&kernel_result.color);
                    current_ray = Ray::new(
                            &pt + &(&kernel_result.direction * 1e-6),
                            kernel_result.direction);
                },
                Intersection::NoHit => {
                    color = Vec::zero();
                    current_ray = Ray::zero();
                }
            }
        }
        color
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
