use core;

use geom;

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
    /// Computes an outgoing direction for the given incoming direction on the surface.
    /// The depth is given as a hint, e.g. for Russian roulette.
    /// The normal of the intersection and a reference to the prim are provided for material
    /// computations.
    /// The RNG should be re-used across bounces for performance reasons.
    /// Note that the incoming direction and outgoing directions should be assumed to be
    /// unit-length. Failure to maintain the unit-length invariant may cause rendering errors
    /// in other parts of the pipeline that assume that rays are unit-length.
    fn bounce(&self, depth: usize, incoming_direction: &core::Vec, normal: &core::Vec,
            prim: &Box<geom::Prim + Sync + Send>, rng: &mut rand::XorShiftRng) -> KernelResult;
}

pub struct DisplayColorKernel {
    pub max_depth: usize,
}

impl DisplayColorKernel {
    pub fn new() -> DisplayColorKernel {
        DisplayColorKernel {max_depth: 1}
    }
    pub fn with_max_depth(max_depth: usize) -> DisplayColorKernel {
        DisplayColorKernel {max_depth: max_depth}
    }
}

impl Kernel for DisplayColorKernel {
    fn bounce(&self, depth: usize, _: &core::Vec, normal: &core::Vec,
        prim: &Box<geom::Prim + Sync + Send>, rng: &mut rand::XorShiftRng) -> KernelResult
    {
        if depth == self.max_depth {
            KernelResult {
                throughput: core::Vec::zero(),
                light: core::Vec::one(),
                direction: core::Vec::zero()
            }
        }
        else {
            let (tangent, binormal) = normal.coord_system();
            let cosine_sample_hemis = core::CosineSampleHemisphere {flipped: false};
            let outgoing_local = cosine_sample_hemis.ind_sample(rng);
            let outgoing_world = outgoing_local.local_to_world(&tangent, &binormal, &normal);
            KernelResult {
                throughput: prim.display_color().clone(),
                light: core::Vec::zero(),
                direction: outgoing_world
            }
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
        prim: &Box<geom::Prim + Sync + Send>, rng: &mut rand::XorShiftRng)
        -> KernelResult
    {
        let material = prim.material();

        // Check for scattering (reflection/transmission).
        let (tangent, binormal) = normal.coord_system();
        // XXX let incoming_local = incoming_direction.world_to_local(&tangent, &binormal, &normal);
        let cosine_sample_hemis = core::CosineSampleHemisphere {flipped: false};
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
