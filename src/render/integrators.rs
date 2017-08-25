use core;
use geom;

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
            geom::Intersection::Hit {dist: _, surface_props: _, prim} => {
                prim.material().display_color().clone()
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
                geom::Intersection::Hit {dist, surface_props, prim} => {
                    // Check for scattering (reflection/transmission).
                    // Note: the material pipeline expects the incoming direction to face away from
                    // the hit point (i.e. toward the previous hit point or eye).
                    let incoming_world = -&current_ray.direction;
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
                            core::clamped_lerp(0.0, 0.75, throughput.luminance())
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
