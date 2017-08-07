use material::lights;
use material::lobes;

use core;

use std;
use rand;
use rand::distributions::IndependentSample;
use rand::distributions::range::Range;

pub struct Material {
    // pub base_color: core::Vec,
    // pub incandescence: core::Vec,
    //
    // pub metallic: f64,
    // pub specular_tint: f64,
    // pub roughness: f64,
    // pub anisotropic: f64,
    // pub sheen: f64,
    // pub sheen_tint: f64,
    // pub clearcoat: f64,
    // pub clearcoat_gloss: f64,
    // pub scatter_distance: core::Vec,
    // pub ior: f64,
    // pub spec_trans: f64,
    // pub diff_trans: f64,
    // pub flatness:f64,
    display: core::Vec,
    light: Box<lights::Light>,
    lobes: std::vec::Vec<Box<lobes::Lobe>>
}

impl Material {
    /// Creates a material with lobes that form the Disney principled BSSRDF shader.
    pub fn disney(base_color: core::Vec, incandescence: core::Vec) -> Material {
        Material {
            // base_color: base_color,
            // incandescence: incandescence,
            //
            // metallic: 0.0,
            // specular_tint: 0.0,
            // roughness: 1.0,
            // anisotropic: 0.0,
            // sheen: 0.0,
            // sheen_tint: 0.0,
            // clearcoat: 0.0,
            // clearcoat_gloss: 0.0,
            // ior: 0.0,
            // spec_trans: 0.0,
            // scatter_distance: core::Vec::zero(),
            display: base_color,
            light: Box::new(lights::DiffuseAreaLight {color: incandescence}),
            lobes: vec![
                Box::new(lobes::DisneyDiffuseRefl {color: base_color}),
                Box::new(lobes::DisneyRetroRefl {color: base_color, roughness: 1.0}),
            ]
        }
    }

    pub fn display_color(&self) -> &core::Vec {
        &self.display
    }

    /// See PBRT 3e, page 832.
    pub fn sample(&self, i: &core::Vec, rng: &mut rand::XorShiftRng) -> lobes::LobeSample {
        // Choose a lobe and sample it.
        let range = Range::new(0, self.lobes.len());
        let r = range.ind_sample(rng);
        let lobe = &self.lobes[r];
        let mut sample = lobe.sample_f(i, rng);

        // Compute overall PDF over all lobes (if the chosen lobe wasn't specular).
        if !lobe.kind().contains(lobes::LOBE_SPECULAR) {
            for idx in 0..self.lobes.len() {
                if idx != r {
                    sample.pdf += self.lobes[idx].pdf(i, &sample.outgoing);
                }
            }
        }
        sample.pdf /= self.lobes.len() as f64;

        // Compute overall BSDF over all lobes (if the chosen lobe wasn't specular).
        if !lobe.kind().contains(lobes::LOBE_SPECULAR) {
            // XXX: reflect should actually be based on geom normal, not shading normal.
            // Need to introduce concept of geom vs shading normals.
            // Doesn't matter at this point because we just have spheres.
            let reflect = i.is_local_same_hemisphere(&sample.outgoing);
            for idx in 0..self.lobes.len() {
                if idx != r &&
                        ((reflect && lobe.kind().contains(lobes::LOBE_REFLECTION)) ||
                        (!reflect && lobe.kind().contains(lobes::LOBE_TRANSMISSION))) {
                    sample.result = &sample.result + &self.lobes[idx].f(i, &sample.outgoing);
                }
            }
        }

        if sample.pdf == 0.0 {
            sample.result = core::Vec::zero();
            sample.pdf = 1.0;
        }

        debug_assert!(sample.result.is_finite());
        debug_assert!(sample.pdf.is_finite());
        debug_assert!(sample.pdf > 0.0);

        sample
    }

    pub fn light(&self, i: &core::Vec) -> core::Vec {
        self.light.l(i)
    }
}
