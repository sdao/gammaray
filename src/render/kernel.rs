use core;

use prim;

use std;
use rand;
use rand::distributions::IndependentSample;
use rand::Rng;

pub struct KernelResult {
    pub throughput: core::Vec,
    pub light: core::Vec,
    pub direction: core::Vec,
}

pub trait Kernel {
    // XXX: ray could just be direction here.
    fn bounce(&self, depth: usize, incoming_direction: &core::Vec, normal: &core::Vec,
            prim: &Box<prim::Prim + Sync>, rng: &mut rand::XorShiftRng) -> KernelResult;
}

pub struct DisplayColorKernel {
}

impl DisplayColorKernel {
    pub fn new() -> DisplayColorKernel {
        DisplayColorKernel {}
    }
}

impl Kernel for DisplayColorKernel {
    fn bounce(&self, _: usize, _: &core::Vec, _: &core::Vec, prim: &Box<prim::Prim + Sync>,
        _: &mut rand::XorShiftRng) -> KernelResult
    {
        KernelResult {
            throughput: core::Vec::zero(),
            light: prim.display_color().clone(),
            direction: core::Vec::zero()
        }
    }
}

pub struct PathTracerKernel {
}

impl PathTracerKernel {
    pub fn new() -> PathTracerKernel {
        PathTracerKernel {}
    }
}

const RUSSIAN_ROULETTE_DEPTH: usize = 10;

impl Kernel for PathTracerKernel {
    fn bounce(&self, depth: usize, incoming_direction: &core::Vec, normal: &core::Vec,
        prim: &Box<prim::Prim + Sync>, rng: &mut rand::XorShiftRng)
        -> KernelResult
    {
        let material = prim.material();

        // Check for scattering (reflection/transmission).
        let (tangent, binormal) = normal.coord_system();
        let incoming_local = incoming_direction.world_to_local(&tangent, &binormal, &normal);
        let cosine_sample_hemis = core::CosineSampleHemisphere {flipped: incoming_local.z < 0.0};
        let outgoing_local = cosine_sample_hemis.ind_sample(rng);
        let bsdf = &material.albedo * std::f64::consts::FRAC_1_PI;
        let pdf = core::CosineSampleHemisphere::pdf(&outgoing_local);
        let outgoing_world = outgoing_local.local_to_world(&tangent, &binormal, &normal);

        let light = material.incandescence.clone();
        let mut throughput = &bsdf * (normal.dot(&outgoing_world).abs() / pdf);
        let mut dir = outgoing_world;

        // Do Russian Roulette if this path is "old".
        if depth > RUSSIAN_ROULETTE_DEPTH || throughput.is_nearly_zero() {
            let rv = rng.next_f64();

            let prob_live = core::clamped_lerp(0.25, 1.00, throughput.luminance());

            if rv < prob_live {
                // The ray lives (more energy = more likely to live).
                // Increase its energy to balance out probabilities.
                throughput = &throughput / prob_live;
            }
            else {
                // The ray dies.
                dir = core::Vec::zero();
            }
        }

        KernelResult {throughput: throughput, light: light, direction: dir}
    }
}
