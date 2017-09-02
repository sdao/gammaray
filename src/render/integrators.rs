use core;
use geom;
use material;

use std;
use std::cell::RefCell;
use rand;
use rand::Rng;

// Sums the light reaching the eye by way of a given ray.
// The implementation of integrators is flexible; they can always return the same result for
// each ray, or they can perform Monte Carlo integration that takes many iterations to converge.
pub trait Integrator : Sync + Send {
    fn integrate(&self, initial_ray: &core::Ray, bvh: &geom::Bvh, rng: &mut rand::XorShiftRng)
        -> core::Vec;
}

pub struct DisplayColorIntegrator {
}

impl Integrator for DisplayColorIntegrator {
    fn integrate(&self, initial_ray: &core::Ray, bvh: &geom::Bvh, _: &mut rand::XorShiftRng)
        -> core::Vec
    {
        match bvh.intersect(initial_ray) {
            geom::Intersection::Hit {dist: _, surface_props: _, prim_index} => {
                bvh[prim_index].material().display_color().clone()
            },
            geom::Intersection::NoHit => {
                core::Vec::zero()
            }
        }
    }
}

/// The distance to push the origin of each new ray along the normal.
/// XXX: PBRT says that we should be reprojecting and computing an error bound instead.
/// XXX: Seems like we need around 1e-3 for floats and 1e-6 for doubles if hardcoding.
const RAY_PUSH_DIST: f32 = 1.0e-3;
const RUSSIAN_ROULETTE_DEPTH: usize = 10;
const RUSSIAN_ROULETTE_DEPTH_AGRESSIVE: usize = 20;

pub struct PathTracerIntegrator {
}

impl Integrator for PathTracerIntegrator {
    fn integrate(&self, initial_ray: &core::Ray, bvh: &geom::Bvh, rng: &mut rand::XorShiftRng)
        -> core::Vec
    {
        let mut depth = 0usize;
        let mut light = core::Vec::zero();
        let mut throughput = core::Vec::one();
        let mut current_ray = initial_ray.clone();
        while !throughput.is_exactly_zero() {
            match bvh.intersect(&current_ray) {
                geom::Intersection::Hit {dist, surface_props, prim_index} => {
                    // Check for scattering (reflection/transmission).
                    // Note: the material pipeline expects the incoming direction to face away from
                    // the hit point (i.e. toward the previous hit point or eye).
                    let incoming_world = -&current_ray.direction;
                    let prim = &bvh[prim_index];
                    let sample = prim.material().sample_world(&incoming_world, &surface_props, rng);

                    // Add illumination first, and then update throughput.
                    light = &light + &throughput.comp_mult(&sample.emission);
                    throughput = throughput.comp_mult(
                            &(&sample.radiance *
                            (f32::abs(surface_props.normal.dot(&sample.outgoing)) / sample.pdf)));
                    current_ray = core::Ray::new(
                            &current_ray.at(dist) + &(&sample.outgoing * RAY_PUSH_DIST),
                            sample.outgoing);

                    // Do Russian Roulette if this path is "old".
                    if depth > RUSSIAN_ROULETTE_DEPTH || throughput.is_nearly_zero() {
                        let rv = rng.next_f32();

                        let prob_live = if depth > RUSSIAN_ROULETTE_DEPTH_AGRESSIVE {
                            core::clamped_lerp(0.10, 0.75, throughput.luminance())
                        }
                        else {
                            core::clamped_lerp(0.25, 1.00, throughput.luminance())
                        };

                        if rv < prob_live {
                            // The ray lives (more energy = more likely to live).
                            // Increase its energy to balance out probabilities.
                            throughput = &throughput / prob_live;
                        }
                        else {
                            // The ray dies.
                            throughput = core::Vec::zero();
                        }
                    }
                },
                geom::Intersection::NoHit => {
                    throughput = core::Vec::zero();
                }
            }

            depth += 1;
        }

        light
    }
}

thread_local!(static BDPT_CAMERA_STORAGE : RefCell<BdptPath> = RefCell::new(BdptPath::new()));

// XXX: This struct probably contains too much information.
// After we finish implementing the BDPT integrator, we should try to trim this struct down.
struct BdptVertex {
    // Incoming ray. (In our terminology, this is the camera ray.)
    pub ray: core::Ray,
    // Intersection of the ray at the surface.
    pub intersection: geom::Intersection,
    // Sample of the material properties at the surface interaction.
    pub sample: material::MaterialSample,
    // Light throughput *before* interacting with the surface.
    // (In our terminology, "before" means closer to the camera. This is the opposite of the
    // terminology used in PBRT.)
    pub throughput: core::Vec,
}

type BdptPath = std::vec::Vec<BdptVertex>;

pub struct BdptIntegrator {
}

impl BdptIntegrator {
    fn random_walk(
        initial_ray: &core::Ray, bvh: &geom::Bvh,
        rng: &mut rand::XorShiftRng, storage: &mut BdptPath)
    {
        let mut depth = 0usize;
        let mut throughput = core::Vec::one();
        let mut current_ray = initial_ray.clone();
        while !throughput.is_exactly_zero() {
            match bvh.intersect(&current_ray) {
                geom::Intersection::Hit {dist, surface_props, prim_index} => {
                    let prev_throughput = throughput;

                    // Check for scattering (reflection/transmission).
                    // Note: the material pipeline expects the incoming direction to face away from
                    // the hit point (i.e. toward the previous hit point or eye).
                    let incoming_world = -&current_ray.direction;
                    let prim = &bvh[prim_index];
                    let sample = prim.material().sample_world(&incoming_world, &surface_props, rng);

                    throughput = throughput.comp_mult(
                            &(&sample.radiance *
                            (f32::abs(surface_props.normal.dot(&sample.outgoing)) / sample.pdf)));
                    current_ray = core::Ray::new(
                            &current_ray.at(dist) + &(&sample.outgoing * RAY_PUSH_DIST),
                            sample.outgoing);

                    // XXX: Would have liked to do this up where prev_throughput is defined but
                    // that means moving surface_props, which makes Rust borrow checker unhappy.
                    storage.push(BdptVertex {
                            ray: current_ray.clone(),
                            intersection: geom::Intersection::Hit {dist, surface_props, prim_index},
                            sample: sample,
                            throughput: prev_throughput
                    });

                    // Do Russian Roulette if this path is "old".
                    if depth > RUSSIAN_ROULETTE_DEPTH || throughput.is_nearly_zero() {
                        let rv = rng.next_f32();

                        let prob_live = if depth > RUSSIAN_ROULETTE_DEPTH_AGRESSIVE {
                            core::clamped_lerp(0.10, 0.75, throughput.luminance())
                        }
                        else {
                            core::clamped_lerp(0.25, 1.00, throughput.luminance())
                        };

                        if rv < prob_live {
                            // The ray lives (more energy = more likely to live).
                            // Increase its energy to balance out probabilities.
                            throughput = &throughput / prob_live;
                        }
                        else {
                            // The ray dies.
                            throughput = core::Vec::zero();
                        }
                    }
                },
                geom::Intersection::NoHit => {
                    throughput = core::Vec::zero();
                }
            }

            depth += 1;
        }
    }
}

impl Integrator for BdptIntegrator {
    fn integrate(&self, initial_ray: &core::Ray, bvh: &geom::Bvh, rng: &mut rand::XorShiftRng)
        -> core::Vec
    {
        let mut light = core::Vec::zero();
        BDPT_CAMERA_STORAGE.with(|x| {
            let mut camera_storage = &mut x.borrow_mut();
            camera_storage.clear();
            BdptIntegrator::random_walk(initial_ray, bvh, rng, &mut camera_storage);

            for vertex in camera_storage.iter() {
                light = &light + &vertex.throughput.comp_mult(&vertex.sample.emission);
            }
        });

        light
    }
}
