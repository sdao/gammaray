use material::lobes;

use core;

use rand;

pub struct BxdfSample {
    pub result: core::Vec,
    pub outgoing: core::Vec,
    pub pdf: f64
}

pub struct Disney {
    pub base_color: core::Vec,
    pub incandescence: core::Vec,

    pub metallic: f64,
    pub specular_tint: f64,
    pub roughness: f64,
    pub anisotropic: f64,
    pub sheen: f64,
    pub sheen_tint: f64,
    pub clearcoat: f64,
    pub clearcoat_gloss: f64,
    pub scatter_distance: core::Vec,
    pub ior: f64,
    pub spec_trans: f64,
    pub diff_trans: f64,
    pub flatness:f64,
}

impl Disney {
    pub fn new(base_color: core::Vec, incandescence: core::Vec) -> Disney {
        Disney {
            base_color: base_color,
            incandescence: incandescence,

            metallic: 0.0,
            specular_tint: 0.0,
            roughness: 1.0,
            anisotropic: 0.0,
            sheen: 0.0,
            sheen_tint: 0.0,
            clearcoat: 0.0,
            clearcoat_gloss: 0.0,
            scatter_distance: core::Vec::zero(),
            ior: 0.0,
            spec_trans: 0.0,
            diff_trans: 0.0,
            flatness: 0.0
        }
    }

    pub fn sample(&self, i: &core::Vec, rng: &mut rand::XorShiftRng) -> BxdfSample {
        lobes::sample_diffuse(self, i, rng)
    }
}
