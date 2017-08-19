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
    // pub metallic: f32,
    // pub specular_tint: f32,
    // pub roughness: f32,
    // pub anisotropic: f32,
    // pub sheen: f32,
    // pub sheen_tint: f32,
    // pub clearcoat: f32,
    // pub clearcoat_gloss: f32,
    // pub scatter_distance: core::Vec,
    // pub ior: f32,
    // pub spec_trans: f32,
    // pub diff_trans: f32,
    // pub flatness:f32,
    display: core::Vec,
    light: Box<lights::Light>,
    lobes: std::vec::Vec<Box<lobes::Lobe>>
}

impl Material {
    pub fn diffuse_light(incandescence: core::Vec) -> Material {
        Material {
            display: incandescence,
            light: Box::new(lights::DiffuseAreaLight {color: incandescence}),
            lobes: vec![]
        }
    }
    /// Creates a material with lobes that form the Disney principled BSSRDF shader.
    /// Burley's 2012 SIGGRAPH course notes presents the basic BRDF:
    /// http://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_notes_v3.pdf
    /// Burley's 2015 SIGGRAPH course notes extends it to transmissive effects:
    /// http://blog.selfshadow.com/publications/s2015-shading-course/burley/s2015_pbs_disney_bsdf_notes.pdf
    pub fn disney() -> DisneyMaterialBuilder {
        DisneyMaterialBuilder::new()
    }

    pub fn mirror() -> Material {
        Material {
            display: core::Vec::one(),
            light: Box::new(lights::DiffuseAreaLight {color: core::Vec::zero()}),
            lobes: vec![
                Box::new(lobes::PerfectMirror::new())
            ]
        }
    }

    pub fn display_color(&self) -> &core::Vec {
        &self.display
    }

    /// See PBRT 3e, page 832.
    pub fn sample(&self, i: &core::Vec, rng: &mut rand::XorShiftRng) -> lobes::LobeSample {
        if self.lobes.len() == 0 {
            return lobes::LobeSample::zero();
        }

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
        sample.pdf /= self.lobes.len() as f32;

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

pub struct DisneyMaterialBuilder {
    _base_color: core::Vec,
    _roughness: f32,
    _ior: f32,
    _metallic: f32,
    _specular_trans: f32,
    _specular_tint: f32,
}

impl DisneyMaterialBuilder {
    pub fn new() -> DisneyMaterialBuilder {
        DisneyMaterialBuilder {
            _base_color: core::Vec::one(),
            _roughness: 0.5,
            _ior: 1.5,
            _metallic: 0.0,
            _specular_trans: 0.0,
            _specular_tint: 0.0,
        }
    }

    pub fn build(&self) -> Material {
        // Combo of three models: diffuse_weight + trans_weight + metallic = 1.0
        let diffuse_weight = (1.0 - self._metallic) * (1.0 - self._specular_trans);
        let trans_weight = (1.0 - self._metallic) * self._specular_trans;
        let mut lobes_list = std::vec::Vec::<Box<lobes::Lobe>>::new();
        
        // Diffuse
        if diffuse_weight > 0.0 {
            let diffuse_color = &self._base_color * diffuse_weight;
            lobes_list.push(Box::new(lobes::DisneyDiffuseRefl::new(diffuse_color)));
            lobes_list.push(Box::new(lobes::DisneyRetroRefl::new(diffuse_color, self._roughness)));
        }

        // Specular reflection
        if self._ior > 1.0 {
            lobes_list.push(Box::new(lobes::DisneySpecularRefl::new(
                    self._base_color, self._roughness, self._ior, self._specular_tint,
                    self._metallic)))
        }

        // Specular transmission
        if trans_weight > 0.0 {
            // PBRT suggests that we take scale up the base color to its sqrt
            // for art-direction purposes; it makes it so that light that enters and exits
            // will have the base color instead of being darker.
            let specular_trans_color = trans_weight * &self._base_color.sqrt();
            lobes_list.push(Box::new(lobes::DisneySpecularTrans::new(
                    specular_trans_color, self._roughness, self._ior)));
        }

        Material {
            display: self._base_color,
            light: Box::new(lights::NullLight {}),
            lobes: lobes_list
        }
    }

    pub fn base_color(&mut self, val: core::Vec) -> &mut Self {
        self._base_color = val;
        self
    }

    pub fn roughness(&mut self, val: f32) -> &mut Self {
        self._roughness = val;
        self
    }

    pub fn ior(&mut self, val: f32) -> &mut Self {
        self._ior = val;
        self
    }

    pub fn metallic(&mut self, val: f32) -> &mut Self {
        self._metallic = val;
        self
    }

    pub fn specular_trans(&mut self, val: f32) -> &mut Self {
        self._specular_trans = val;
        self
    }

    pub fn specular_tint(&mut self, val: f32) -> &mut Self {
        self._specular_tint = val;
        self
    }
}
