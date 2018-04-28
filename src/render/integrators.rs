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
                            &incoming_world, &surface_props, true, rng);

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

const BDPT_RUSSIAN_ROULETTE_DEPTH: usize = 4;
const BDPT_MAX_DEPTH: usize = 16;
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
    // Type of lobe that was sampled at the hit point. Not indicative of all lobes in material.
    pub lobe_kind: material::LobeKind,
    // Whether the vertex can be connected to other vertices, based on its lobe kinds.
    pub connectible: bool,
    // Prim that was hit.
    pub prim_index: usize,
    // Probability of obtaining this vertex from the previous vertex in the path.
    pub pdf_forward: f32,
    // Probability of hypothetically obtaining this vertex from the next vertex in the path, e.g.
    // the probability if importance (light-to-camera) were sampled instead of radiance
    // (camera-to-light) or vice versa.
    pub pdf_reverse: f32,
}

type BdptPath = std::vec::Vec<BdptVertex>;

pub struct BdptIntegrator {
}

impl BdptIntegrator {
    fn random_walk(
        initial_ray: &core::Ray, initial_throughput: &core::Vec, camera_to_light: bool,
        bvh: &geom::Bvh, rng: &mut rand::XorShiftRng, storage: &mut BdptPath)
    {
        let mut throughput = initial_throughput.clone();
        let mut current_ray = initial_ray.clone();
        while !throughput.is_exactly_zero() && storage.len() < BDPT_MAX_DEPTH {
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
                            &incoming_world, &surface_props, camera_to_light, rng);
                    let pdf_reverse = prim.material().pdf_world(
                            &sample.outgoing, &incoming_world, &surface_props);
                    let connectible = prim.material().count_lobes(
                            material::LobeKind::LOBE_DIFFUSE | material::LobeKind::LOBE_GLOSSY) != 0;

                    throughput = throughput.comp_mult(
                            &(&sample.radiance *
                            (f32::abs(surface_props.normal.dot(&sample.outgoing)) / sample.pdf)));
                    throughput = &throughput *
                            BdptIntegrator::correct_shading_normal(
                            &incoming_world, &sample.outgoing, &surface_props, camera_to_light);
                    current_ray = core::Ray::new(current_ray.at(dist), sample.outgoing).nudge();

                    // Set the pdf_reverse of the last vertex (if one exists).
                    match storage.last_mut() {
                        Some(x) => {
                            let convert_density = BdptIntegrator::convert_density(
                                    &x.point, &hit_point, &surface_props);
                            x.pdf_reverse = pdf_reverse * convert_density;
                        },
                        None => {}
                    }

                    // Add to the random walk path.
                    storage.push(BdptVertex {
                        incoming_world: incoming_world,
                        point: hit_point,
                        surface_props: surface_props,
                        throughput: prev_throughput,
                        emission: sample.emission,
                        lobe_kind: sample.kind,
                        connectible: connectible,
                        prim_index: prim_index,
                        pdf_forward: sample.pdf,
                        pdf_reverse: 1.0,
                    });

                    // Do Russian Roulette if this path is "old".
                    if storage.len() >= BDPT_RUSSIAN_ROULETTE_DEPTH || throughput.is_nearly_zero() {
                        let rv = rng.next_f32();
                        let prob_live = core::clamped_lerp(0.25, 0.75, throughput.luminance());

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

                    // As if we hit a black infinite area light.
                    storage.push(BdptVertex {
                        incoming_world: -&current_ray.direction,
                        point: core::Vec::zero(),
                        surface_props: geom::SurfaceProperties::zero(),
                        throughput: throughput,
                        emission: core::Vec::zero(),
                        lobe_kind: material::LobeKind::LOBE_NONE,
                        connectible: false,
                        prim_index: std::usize::MAX,
                        pdf_forward: 1.0,
                        pdf_reverse: 1.0,
                    });
                }
            }
        }
    }

    /// See PBRT 3e p. 963.
    fn correct_shading_normal(
        incoming_world: &core::Vec, outgoing_world: &core::Vec,
        surface_props: &geom::SurfaceProperties, camera_to_light: bool) -> f32
    {
        if camera_to_light {
            1.0
        }
        else {
            let num = f32::abs(incoming_world.dot(&surface_props.normal)) *
                    f32::abs(outgoing_world.dot(&surface_props.geom_normal));
            let denom = f32::abs(incoming_world.dot(&surface_props.geom_normal)) *
                    f32::abs(outgoing_world.dot(&surface_props.normal));
            if denom == 0.0 {
                0.0
            }
            else {
                num / denom
            }
        }
    }

    fn convert_density(
        prev_pos: &core::Vec,
        next_pos: &core::Vec,
        next_surface_props: &geom::SurfaceProperties) -> f32
    {
        let dist = (next_pos - prev_pos).magnitude();
        if dist == 0.0 {
            0.0
        }
        else {
            let prev_to_next = &(next_pos - prev_pos) / dist;
            f32::abs(next_surface_props.normal.dot(&prev_to_next)) / (dist * dist)
        }
    }

    /// Connects the path according to the given strategy, and returns the unweighted radiance.
    fn connect(&self,
        camera_len: usize,
        light_len: usize,
        camera_storage: &BdptPath,
        light_storage: &BdptPath,
        bvh: &geom::Bvh) -> core::Vec
    {
        // We only deal with strategies with at least one camera point.
        debug_assert!(camera_len >= 1);
        let camera_vertex = &camera_storage[camera_len - 1];

        if light_len == 0 {
            if camera_vertex.prim_index == std::usize::MAX {
                return core::Vec::zero();
            }

            // Camera path only.
            return camera_vertex.throughput.comp_mult(&camera_vertex.emission);
        }
        else {
            // Camera path connects with light path.
            let light_vertex = &light_storage[light_len - 1];

            if camera_vertex.prim_index == std::usize::MAX ||
                    light_vertex.prim_index == std::usize::MAX {
                return core::Vec::zero();
            }

            // Specular camera or light vertex means that it's impossible to connect the
            // two (the pdf and radiance would be 0.0 unless the directions were sampled).
            // Optimize and early-out so that we don't have to do computations.
            if !camera_vertex.connectible || !light_vertex.connectible {
                return core::Vec::zero();
            }

            let camera_to_light = (&light_vertex.point - &camera_vertex.point).normalized();
            let light_to_camera = -&camera_to_light;

            let camera_material = bvh[camera_vertex.prim_index].material();
            let light_material = bvh[light_vertex.prim_index].material();
            let connect_radiance = camera_material.f_world(
                    &camera_vertex.incoming_world, &camera_to_light,
                    &camera_vertex.surface_props, true);
            let connect_emission = if light_len == 1 {
                light_material.light_world(
                        &light_to_camera, &light_vertex.surface_props)
            }
            else {
                light_material.f_world(
                        &light_vertex.incoming_world, &light_to_camera,
                        &light_vertex.surface_props, false)
            };

            let g = f32::abs(camera_vertex.surface_props.normal.dot(&camera_to_light));
            let convert_density = BdptIntegrator::convert_density(
                    &camera_vertex.point, &light_vertex.point, &light_vertex.surface_props);

            let contrib = &camera_vertex.throughput
                    .comp_mult(&connect_radiance)
                    .comp_mult(&connect_emission)
                    .comp_mult(&light_vertex.throughput) * (g * convert_density);
            
            // Delay visibility testing until the very end. Try to avoid visibility testing if the
            // contribution is already black.
            if contrib.is_nearly_zero() {
                return contrib;
            }
            else if bvh.visibility(&camera_vertex.point, &light_vertex.point) {
                return contrib;
            }
            else {
                return core::Vec::zero();
            }
        }
    }

    fn weight(&self, camera_len: usize, light_len: usize) -> f32 {
        // There are path_len ways to make the path in this rendering system:
        // cam: 1              + light: (path_len - 1)
        // cam: 2              + light: (path_len - 2)
        // ...
        // cam: (path_len - 1) + light: 1
        // cam: path_len       + light: 0
        let path_len = camera_len + light_len;
        1.0 / path_len as f32
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
                BdptIntegrator::random_walk(
                        initial_ray, &core::Vec::one(), true, bvh, rng, &mut camera_storage);
                if camera_storage.len() == 0 {
                    return; // Camera ray escaped.
                }

                let mut light_storage = &mut y.borrow_mut();
                let light_sample = bvh.sample_light(rng);
                if light_sample.point_pdf == 0.0 || light_sample.dir_pdf == 0.0 {
                    return; // Can't divide by zero; chance of this happening is very low.
                }
                let light_dir = &light_sample.ray.direction;
                let light_material = bvh[light_sample.prim_index].material();
                let initial_emission =
                        &light_material.light_world(light_dir, &light_sample.surface_props)
                        * (f32::abs(light_sample.surface_props.geom_normal.dot(light_dir))
                        / (light_sample.point_pdf * light_sample.dir_pdf));
                light_storage.clear();
                light_storage.push(BdptVertex {
                    incoming_world: core::Vec::zero(),
                    point: light_sample.ray.origin,
                    surface_props: light_sample.surface_props,
                    throughput: &core::Vec::one() / light_sample.point_pdf,
                    emission: core::Vec::zero(),
                    lobe_kind: material::LobeKind::LOBE_NONE,
                    connectible: true,
                    prim_index: light_sample.prim_index,
                    pdf_forward: light_sample.point_pdf * light_sample.dir_pdf,
                    pdf_reverse: 1.0,
                });
                BdptIntegrator::random_walk(
                        &light_sample.ray.nudge(), &initial_emission, false, bvh, rng,
                        &mut light_storage);

                // Execute all connection strategies.
                for camera_len in 1..(camera_storage.len() + 1) {
                    for light_len in 0..(light_storage.len() + 1) {
                        if camera_len + light_len > BDPT_MAX_DEPTH {
                            continue;
                        }

                        let l = self.connect(
                                camera_len, light_len, camera_storage, light_storage, bvh);
                        let w = self.weight(camera_len, light_len);
                        light = &light + &(&l * w);
                    }
                }
            });
        });

        light
    }
}
