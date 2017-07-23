use core;

use prim;

use std;
use rand;
use rand::distributions::IndependentSample;
use rand::Rng;

pub struct KernelResult {
    pub life: core::Vec,
    pub illum: core::Vec,
    pub direction: core::Vec,
}

pub trait Kernel {
    // XXX: ray could just be direction here.
    fn bounce(&self, depth: usize, ray: &core::Ray, color: &core::Vec, normal: &core::Vec,
            prim: &Box<prim::Prim + Sync>) -> KernelResult;
}

pub struct DisplayColorKernel {
}

impl DisplayColorKernel {
    pub fn new() -> DisplayColorKernel {
        DisplayColorKernel {}
    }
}

impl Kernel for DisplayColorKernel {
    fn bounce(&self, _: usize, _: &core::Ray, _: &core::Vec, _: &core::Vec,
        prim: &Box<prim::Prim + Sync>)
        -> KernelResult
    {
        KernelResult {life: core::Vec::zero(), illum: prim.display_color().clone(), direction: core::Vec::zero()}
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
    fn bounce(&self, depth: usize, ray: &core::Ray, color: &core::Vec, normal: &core::Vec,
        prim: &Box<prim::Prim + Sync>)
        -> KernelResult
    {
        let mut rng = rand::thread_rng();
        let mut beta = color.clone();
        let mut dir = core::Vec::zero();

        // Check for scattering (reflection/transmission).
        // XXX adjust beta here.
        let (tangent, binormal) = normal.coord_system();
        let incoming_local = ray.direction.world_to_local(&tangent, &binormal, &normal);
        let cosine_sample_hemis = core::CosineSampleHemisphere {flipped: incoming_local.z < 0.0};
        let outgoing_local = cosine_sample_hemis.ind_sample(&mut rng);
        let bsdf = &prim.material().albedo * std::f64::consts::FRAC_1_PI;
        let pdf = core::CosineSampleHemisphere::pdf(&outgoing_local);
        let outgoing_world = outgoing_local.local_to_world(&tangent, &binormal, &normal);

        let mut life: core::Vec = (&bsdf * (normal.dot(&outgoing_world).abs() / pdf)).comp_mult(color);
        let mut illum = prim.material().incandescence.clone();
        let mut dir = outgoing_world;

        // Do Russian Roulette if this path is "old".
        if depth > RUSSIAN_ROULETTE_DEPTH || beta.is_nearly_zero() {
            let rv = rng.next_f64();

            let prob_live = core::clamped_lerp(0.25, 1.00, beta.luminance());

            if rv < prob_live {
                // The ray lives (more energy = more likely to live).
                // Increase its energy to balance out probabilities.
                life = &life / prob_live;
            }
            else {
                // The ray dies.
                dir = core::Vec::zero();
            }
        }

        KernelResult {life: life.clone(), illum: illum, direction: dir}
    }
}
