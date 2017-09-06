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
                    let sample = prim.material().sample_world(
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

struct BdptVertex {
    // Incoming ray that hit the surface.
    pub incoming_world: core::Vec,
    // Point of the surface intersection.
    pub point: core::Vec,
    // Surface properties at the intersection.
    pub surface_props: geom::SurfaceProperties,
    // Radiance/importance throughput *before* interacting with the surface.
    pub throughput: core::Vec,
    // Emission, if the hit occurred on a light.
    pub emission: core::Vec,
    // Type of lobe that was sampled at the hit point.
    pub lobe_kind: material::LobeKind,
    // Prim that was hit.
    pub prim_index: usize,
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
                    let sample = prim.material().sample_world(
                            &incoming_world, &surface_props, rng);

                    throughput = throughput.comp_mult(
                            &(&sample.radiance *
                            (f32::abs(surface_props.normal.dot(&sample.outgoing)) / sample.pdf)));
                    current_ray = core::Ray::new(current_ray.at(dist), sample.outgoing).nudge();

                    // Add to the random walk path.
                    storage.push(BdptVertex {
                        incoming_world: incoming_world,
                        point: hit_point,
                        surface_props: surface_props,
                        throughput: prev_throughput,
                        emission: sample.emission,
                        lobe_kind: sample.kind,
                        prim_index: prim_index,
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

    /// Connects the path according to the given strategy, and returns the radiance collected on
    /// the path as well as the weight of the path.
    fn connect(&self,
        camera_len: usize,
        light_len: usize,
        camera_storage: &BdptPath,
        light_storage: &BdptPath,
        bvh: &geom::Bvh) -> (core::Vec, f32)
    {
        // We only deal with strategies with at least one camera point.
        debug_assert!(camera_len >= 1);
        let camera_vertex = &camera_storage[camera_len - 1];

        if light_len == 0 {
            // Camera path only.
            let contrib = camera_vertex.throughput.comp_mult(&camera_vertex.emission);
            return (contrib, 1.0);
        }
        else if light_len == 1 {
            // Direct illumination of camera path by the light.
            // Connect a camera vertex directly to a light point.
            let light_vertex = &light_storage[light_storage.len() - light_len];

            // Strategy requires connecting camera and light subpaths.
            // We can't do that for specular camera samples, so we must skip.
            // (We ignore reflectance properties for the light vertex in this case.)
            if camera_vertex.lobe_kind.contains(material::LOBE_SPECULAR) {
                return (core::Vec::zero(), 0.0);
            }

            let contrib = if bvh.visibility(&camera_vertex.point, &light_vertex.point) {
                let camera_to_light = (&light_vertex.point - &camera_vertex.point).normalized();
                let light_to_camera = -&camera_to_light;

                let camera_material = bvh[camera_vertex.prim_index].material();
                let light_material = bvh[light_vertex.prim_index].material();
                let connect_radiance = camera_material.f_world(
                        &camera_vertex.incoming_world, &camera_to_light,
                        &camera_vertex.surface_props);
                let connect_emission = light_material.light_world(
                        &light_to_camera, &light_vertex.surface_props);

                let normal_falloff = f32::abs(
                        camera_vertex.surface_props.normal.dot(&camera_to_light));
                let dist = (&light_vertex.point - &camera_vertex.point).magnitude();

                // Light path pdf was compute as an area measure.
                // We need to convert from area to solid angle measure.
                let convert_pdf = (dist * dist)
                        / f32::abs(light_vertex.surface_props.normal.dot(&light_to_camera));

                &camera_vertex.throughput.comp_mult(&connect_radiance).comp_mult(&connect_emission)
                        .comp_mult(&light_vertex.throughput) * (normal_falloff / convert_pdf)
            }
            else {
                core::Vec::zero()
            };
    
            return (contrib, 1.0);
        }
        else {
            unimplemented!();
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
                let light_sample = bvh.sample_light(rng);
                light_storage.push(BdptVertex {
                    incoming_world: core::Vec::zero(),
                    point: light_sample.point,
                    surface_props: light_sample.surface_props,
                    throughput: &core::Vec::one() / light_sample.pdf,
                    emission: core::Vec::zero(),
                    lobe_kind: material::LOBE_NONE,
                    prim_index: light_sample.prim_index,
                });

                for path_len in 1..(camera_storage.len() + light_storage.len() + 1) {
                    let mut path_light = core::Vec::zero();
                    let mut path_weight = 0.0; // XXX: do actual MIS instead of even weights.

                    // Determine from camera and light path lengths what connection strategies are
                    // actually available for paths of this length.
                    let min_camera = std::cmp::max(1, path_len - light_storage.len());
                    let max_camera = std::cmp::min(camera_storage.len(), path_len);
                    debug_assert!(min_camera >= 1);
                    debug_assert!(min_camera <= camera_storage.len());
                    debug_assert!(max_camera >= 1);
                    debug_assert!(max_camera <= camera_storage.len());

                    // Execute all connection strategies.
                    for camera_len in min_camera..(max_camera + 1) {
                        let light_len = path_len - camera_len;
                        let (l, w) = self.connect(
                                camera_len, light_len, camera_storage, light_storage, bvh);
                        path_light = &path_light + &l;
                        path_weight += w;
                    }

                    if path_weight > 0.0 {
                        light = &light + &(&path_light / path_weight);
                    }
                }
            });
        });

        light
    }
}
