use core;

use geom;

use rand;
use rand::Rng;

pub struct KernelResult {
    pub throughput: core::Vec,
    pub light: core::Vec,
    pub direction: core::Vec,
}

pub trait Kernel : Sync + Send {
    /// Computes an outgoing direction for the given incoming direction on the surface.
    /// The depth is given as a hint, e.g. for Russian roulette.
    /// The surface properties at the intersection and a reference to the prim are provided for
    /// material computations.
    /// The RNG should be re-used across bounces for performance reasons.
    /// Note that the incoming direction and outgoing directions should be assumed to be
    /// unit-length. Failure to maintain the unit-length invariant may cause rendering errors
    /// in other parts of the pipeline that assume that rays are unit-length.
    fn bounce(&self, depth: usize, incoming_direction: &core::Vec,
            surface_props: &geom::SurfaceProperties, prim: &Box<geom::Prim>,
            rng: &mut rand::XorShiftRng) -> KernelResult;
}

/// This is a kernel used for debugging; it encodes the bounce directions in the colors.
pub struct BounceKernel {
}

impl BounceKernel {
    pub fn new() -> BounceKernel { BounceKernel {} }
}

impl Kernel for BounceKernel {
    fn bounce(&self, _: usize, incoming_direction: &core::Vec,
        surface_props: &geom::SurfaceProperties, prim: &Box<geom::Prim>,
        rng: &mut rand::XorShiftRng) -> KernelResult
    {
        let material = prim.material();
        let normal = &surface_props.normal;
        let (tangent, binormal) = normal.coord_system();
        let incoming_local = (-incoming_direction).world_to_local(&tangent, &binormal, &normal);
        let sample = material.sample(&incoming_local, rng);
        let outgoing_world = sample.outgoing.local_to_world(&tangent, &binormal, &normal);

        KernelResult {
            throughput: core::Vec::zero(),
            light: outgoing_world,
            direction: core::Vec::zero()
        }
    }
}

/// This is a kernel used for debugging; it shows the display colors.
pub struct DisplayColorKernel {
}

impl DisplayColorKernel {
    pub fn new() -> DisplayColorKernel { DisplayColorKernel {} }
}

impl Kernel for DisplayColorKernel {
    fn bounce(&self, _: usize, _: &core::Vec, _: &geom::SurfaceProperties,
        prim: &Box<geom::Prim>, _: &mut rand::XorShiftRng) -> KernelResult
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
const RUSSIAN_ROULETTE_DEPTH_AGRESSIVE: usize = 20;

impl Kernel for PathTracerKernel {
    fn bounce(&self, depth: usize, incoming_direction: &core::Vec,
        surface_props: &geom::SurfaceProperties, prim: &Box<geom::Prim>,
        rng: &mut rand::XorShiftRng) -> KernelResult
    {
        let material = prim.material();

        // Check for scattering (reflection/transmission).
        let normal = &surface_props.normal;
        let (tangent, binormal) = normal.coord_system();
        let incoming_local = (-incoming_direction).world_to_local(&tangent, &binormal, &normal);
        let sample = material.sample(&incoming_local, rng);
        let outgoing_world = sample.outgoing.local_to_world(&tangent, &binormal, &normal);

        let light = material.light(&incoming_local);
        let mut throughput = &sample.result * (f32::abs(normal.dot(&outgoing_world)) / sample.pdf);
        let mut dir = outgoing_world;

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
                dir = core::Vec::zero();
            }
        }

        KernelResult {throughput: throughput, light: light, direction: dir}
    }
}
