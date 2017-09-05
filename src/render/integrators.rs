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
                    let sample = prim.material().sample_f_world(
                            &incoming_world, &surface_props, rng);

                    // Add illumination first, and then update throughput.
                    light = &light + &throughput.comp_mult(&sample.emission);
                    throughput = throughput.comp_mult(
                            &(&sample.radiance *
                            (f32::abs(surface_props.normal.dot(&sample.outgoing)) / sample.pdf)));
                    current_ray = core::Ray::new(current_ray.at(dist), sample.outgoing).nudge();

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
thread_local!(static BDPT_LIGHT_STORAGE : RefCell<BdptPath> = RefCell::new(BdptPath::new()));

// XXX: This struct probably contains too much information.
// After we finish implementing the BDPT integrator, we should try to trim this struct down.
struct BdptVertex {
    // Point of the surface intersection.
    pub point: core::Vec,
    pub incoming_world: core::Vec,
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
                    let hit_point = current_ray.at(dist);

                    // Check for scattering (reflection/transmission).
                    // Note: the material pipeline expects the incoming direction to face away from
                    // the hit point (i.e. toward the previous hit point or eye).
                    let incoming_world = -&current_ray.direction;
                    let prim = &bvh[prim_index];
                    let sample = prim.material().sample_f_world(
                            &incoming_world, &surface_props, rng);

                    throughput = throughput.comp_mult(
                            &(&sample.radiance *
                            (f32::abs(surface_props.normal.dot(&sample.outgoing)) / sample.pdf)));
                    current_ray = core::Ray::new(current_ray.at(dist), sample.outgoing).nudge();

                    // XXX: Would have liked to do this up where prev_throughput is defined but
                    // that means moving surface_props, which makes Rust borrow checker unhappy.
                    storage.push(BdptVertex {
                            point: hit_point,
                            incoming_world: incoming_world,
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
            BDPT_LIGHT_STORAGE.with(|y| {
                let mut camera_storage = &mut x.borrow_mut();
                camera_storage.clear();
                BdptIntegrator::random_walk(initial_ray, bvh, rng, &mut camera_storage);

                let mut light_storage = &mut y.borrow_mut();
                light_storage.clear();
                let (pt, intersection, pdf) = bvh.sample_light(rng);
                light_storage.push(BdptVertex {
                    point: pt,
                    incoming_world: core::Vec::zero(),
                    intersection: intersection,
                    sample: material::MaterialSample {
                        emission: core::Vec::zero(),
                        radiance: core::Vec::zero(),
                        outgoing: core::Vec::zero(),
                        pdf: pdf,
                        kind: material::LOBE_NONE,
                    },
                    throughput: &core::Vec::one() / pdf
                });

                for path_len in 1..(camera_storage.len() + light_storage.len() + 1) {
                    let mut path_total = core::Vec::zero();
                    let mut weight_total = 0.0; // oh god this is a hacky MIS implementation; using power heuristic

                    let min_camera = std::cmp::max(1, path_len - light_storage.len());
                    let max_camera = std::cmp::min(camera_storage.len(), path_len);
                    debug_assert!(min_camera >= 1);
                    debug_assert!(min_camera <= camera_storage.len());
                    debug_assert!(max_camera >= 1);
                    debug_assert!(max_camera <= camera_storage.len());
                    // for camera_len in min_camera..(max_camera + 1) {
                        // let light_len = path_len - camera_len;
                    for light_len in 0..2 {
                        let camera_len = path_len - light_len;
                        if camera_len < 1 || camera_len > camera_storage.len() {
                            continue;
                        }

                        // cam vertex always exists
                        let camera_vertex = &camera_storage[camera_len - 1];
                        if light_len == 0 {
                            // Camera path only.
                            let contrib = camera_vertex.throughput.comp_mult(&camera_vertex.sample.emission);
                            path_total = &path_total + &(&contrib * 1.0);
                            weight_total += 1.0;
                        }
                        else if light_len == 1 && !camera_vertex.sample.kind.contains(material::LOBE_SPECULAR) {
                            // doesn't always exist (e.g. light_len = 0)
                            let light_vertex = &light_storage[light_storage.len() - light_len];

                            // Direct illumination of camera path by the light.
                            if let geom::Intersection::Hit {dist: _, surface_props: ref camera_surface_props, prim_index: camera_prim_index} = camera_vertex.intersection {
                                if let geom::Intersection::Hit {dist: _, surface_props: ref light_surface_props, prim_index: light_prim_index} = light_vertex.intersection {
                                    let camera_to_light = (&light_vertex.point - &camera_vertex.point).normalized();
                                    let light_to_camera = -&camera_to_light;
                                    let connect_radiance = bvh[camera_prim_index].material().f_world(&camera_vertex.incoming_world, &camera_to_light, camera_surface_props).radiance;
                                    let connect_emission = bvh[light_prim_index].material().f_world(&light_to_camera, &core::Vec::zero(), light_surface_props).emission;
                                    let norm_correct = f32::abs(camera_surface_props.normal.dot(&camera_to_light));
                                    let vis = bvh.visibility(&camera_vertex.point, &light_vertex.point);
                                    if vis {
                                        let dist = (&light_vertex.point - &camera_vertex.point).magnitude();
                                        // pdf2 converts from area to solid angle measure
                                        let pdf2 =
                                                (dist * dist)
                                                / f32::abs(light_surface_props.normal.dot(&light_to_camera));

                                        let contrib = &camera_vertex.throughput.comp_mult(&connect_radiance).comp_mult(&connect_emission).comp_mult(&light_vertex.throughput) * (norm_correct / pdf2);
                                        path_total = &path_total + &(&contrib * 1.0);
                                    }
                                }
                            }
                            // mucho importante: this needs to happen outside of the vis conditional. because we've already sampled and no take-backsies!
                            weight_total += 1.0;
                        }
                        else {
                            unimplemented!();
                        }
                    }

                    if weight_total > 0.0 {
                        light = &light + &(&path_total / weight_total);
                    }
                }

                // for camera_vertex in camera_storage.iter() {
                //     if let geom::Intersection::Hit {dist: _, surface_props: ref camera_surface_props, prim_index: camera_prim_index} = camera_vertex.intersection {
                //         let light_vertex = &light_storage[0];
                //         if let geom::Intersection::Hit {dist: _, surface_props: ref light_surface_props, prim_index: light_prim_index} = light_vertex.intersection {
                //             let camera_to_light = (&light_vertex.point - &camera_vertex.point).normalized();
                //             let light_to_camera = -&camera_to_light;
                //             let connect_radiance = bvh[camera_prim_index].material().f_world(&camera_vertex.incoming_world, &camera_to_light, camera_surface_props).radiance;
                //             let connect_emission = bvh[light_prim_index].material().f_world(&light_to_camera, &core::Vec::zero(), light_surface_props).emission;
                //             let norm_correct = f32::abs(camera_surface_props.normal.dot(&camera_to_light));
                //             let mut vis = bvh.visibility(&camera_vertex.point, &light_vertex.point);
                //             if vis {
                //                 let dist = (&light_vertex.point - &camera_vertex.point).magnitude();
                //                 // pdf2 converts from area to solid angle measure
                //                 let pdf2 =
                //                         (dist * dist)
                //                         / f32::abs(light_surface_props.normal.dot(&light_to_camera));

                //                 light = &light + &(&camera_vertex.throughput.comp_mult(&(&connect_radiance * norm_correct)).comp_mult(&connect_emission).comp_mult(&light_vertex.throughput) * (1.0 / pdf2));
                //             }
                //         }
                //     }
                // }
            });
        });

        light
    }
}
