use material::lights;
use material::lobes;

use core;
use geom;

use std;
use rand;
use rand::distributions::IndependentSample;
use rand::distributions::range::Range;

pub struct MaterialSample {
    pub emission: core::Vec,
    pub radiance: core::Vec,
    pub outgoing: core::Vec,
    pub pdf: f32,
}

pub struct Material {
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

    pub fn mirror() -> Material {
        Material {
            display: core::Vec::one(),
            light: Box::new(lights::DiffuseAreaLight {color: core::Vec::zero()}),
            lobes: vec![
                Box::new(lobes::PerfectMirror::new())
            ]
        }
    }

    /// Generates a builder to construct a Disney principled material.
    /// You'll need to call build() on the builder to finish building.
    pub fn disney() -> DisneyMaterialBuilder {
        DisneyMaterialBuilder::new()
    }

    pub fn display_color(&self) -> &core::Vec {
        &self.display
    }

    /// See PBRT 3e, page 832.
    /// Args:
    ///   incoming_world should face away from the intersection point.
    ///   surface_props should be in world-space.
    pub fn sample_world(&self, incoming_world: &core::Vec, surface_props: &geom::SurfaceProperties,
        rng: &mut rand::XorShiftRng) -> MaterialSample
    {
        // Convert from world-space to local space.
        let incoming_local = incoming_world.world_to_local(
                &surface_props.tangent, &surface_props.binormal, &surface_props.normal);
        debug_assert!(incoming_world.is_finite(), "incoming_world={}", incoming_world);
        debug_assert!(incoming_local.is_finite(), "il={}, tan={}, bin={}, norm={}",
                incoming_world,
                surface_props.tangent, surface_props.binormal, surface_props.normal);

        // Calculate emission. This doesn't depend on reflecting an outgoing ray.
        let emission = self.light.l(&incoming_local);

        if self.lobes.len() == 0 {
            return MaterialSample {
                emission: emission,
                radiance: core::Vec::zero(),
                outgoing: core::Vec::zero(),
                pdf: 1.0
            };
        }

        // Choose a lobe and sample it.
        let range = Range::new(0, self.lobes.len());
        let r = range.ind_sample(rng);
        let lobe = &self.lobes[r];
        let sample = lobe.sample_f(&incoming_local, rng);

        let outgoing_world = sample.outgoing.local_to_world(
                &surface_props.tangent, &surface_props.binormal, &surface_props.normal);
        let mut radiance = sample.result;
        let mut pdf = sample.pdf;

        // Compute overall PDF over all lobes (if the chosen lobe wasn't specular).
        if !lobe.kind().contains(lobes::LOBE_SPECULAR) {
            for idx in 0..self.lobes.len() {
                if idx != r {
                    pdf += self.lobes[idx].pdf(&incoming_local, &sample.outgoing);
                }
            }
        }
        pdf /= self.lobes.len() as f32;

        // Compute overall BSDF over all lobes (if the chosen lobe wasn't specular).
        if !lobe.kind().contains(lobes::LOBE_SPECULAR) {
            // Whether we're evalauting BTDFs or BRDFs should actually be based on geom normal,
            // not shading normal.
            let reflect = (incoming_world.dot(&surface_props.geom_normal) *
                    outgoing_world.dot(&surface_props.geom_normal)) > 0.0;
            for idx in 0..self.lobes.len() {
                if idx != r &&
                        ((reflect && lobe.kind().contains(lobes::LOBE_REFLECTION)) ||
                        (!reflect && lobe.kind().contains(lobes::LOBE_TRANSMISSION))) {
                    radiance = &radiance + &self.lobes[idx].f(&incoming_local, &sample.outgoing);
                }
            }
        }

        // Normalize; if pdf is zero, then make the radiance black to be safe.
        if pdf == 0.0 {
            radiance = core::Vec::zero();
            pdf = 1.0;
        }

        debug_assert!(radiance.is_finite());
        debug_assert!(pdf.is_finite());
        debug_assert!(pdf > 0.0);

        return MaterialSample {
            emission: emission,
            radiance: radiance,
            outgoing: outgoing_world,
            pdf: pdf,
        };
    }
}

pub struct DisneyMaterialBuilder {
    _base_color: core::Vec,
    _roughness: f32,
    _anisotropic: f32,
    _ior: f32,
    _metallic: f32,
    _specular_trans: f32,
    _specular_tint: f32,
    _sheen: f32,
    _sheen_tint: f32,
    _clearcoat: f32,
    _clearcoat_gloss: f32,
}

/// Creates a material with lobes that form the Disney principled BSSRDF shader.
/// Burley's 2012 SIGGRAPH course notes presents the basic BRDF:
/// http://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_notes_v3.pdf
/// Burley's 2015 SIGGRAPH course notes extends it to transmissive effects:
/// http://blog.selfshadow.com/publications/s2015-shading-course/burley/s2015_pbs_disney_bsdf_notes.pdf
impl DisneyMaterialBuilder {
    pub fn new() -> DisneyMaterialBuilder {
        DisneyMaterialBuilder {
            _base_color: core::Vec::one(),
            _roughness: 0.5,
            _anisotropic: 0.0,
            _ior: 1.5,
            _metallic: 0.0,
            _specular_trans: 0.0,
            _specular_tint: 0.0,
            _sheen: 0.0,
            _sheen_tint: 0.5,
            _clearcoat: 0.0,
            _clearcoat_gloss: 0.1,
        }
    }

    pub fn build(&self) -> Material {
        // Combo of three models: diffuse_weight + trans_weight + metallic = 1.0
        let diffuse_weight = (1.0 - self._metallic) * (1.0 - self._specular_trans);
        let trans_weight = (1.0 - self._metallic) * self._specular_trans;
        let mut lobes_list = std::vec::Vec::<Box<lobes::Lobe>>::new();
        
        // Diffuse, retro-reflection, and sheen
        if diffuse_weight > 0.0 {
            lobes_list.push(Box::new(lobes::DisneyDiffuseRefl::new(
                    self._base_color, self._roughness, self._sheen, self._sheen_tint,
                    diffuse_weight)));
        }

        // Specular reflection
        if self._ior > 1.0 {
            lobes_list.push(Box::new(lobes::DisneySpecularRefl::new_aniso(
                    self._base_color, self._roughness, self._anisotropic, self._ior,
                    self._specular_tint, self._metallic)))
        }

        // Clearcoat (second specular lobe)
        if self._clearcoat > 0.0 {
            lobes_list.push(Box::new(lobes::DisneyClearcoatRefl::new(
                    self._clearcoat, self._clearcoat_gloss)));
        }

        // Specular transmission
        if trans_weight > 0.0 {
            // PBRT suggests that we take scale up the base color to its sqrt
            // for art-direction purposes; it makes it so that light that enters and exits
            // will have the base color instead of being darker.
            let specular_trans_color = trans_weight * &self._base_color.sqrt();
            lobes_list.push(Box::new(lobes::DisneySpecularTrans::new_aniso(
                    specular_trans_color, self._roughness, self._anisotropic, self._ior)));
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

    pub fn anisotropic(&mut self, val: f32) -> &mut Self {
        self._anisotropic = val;
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

    pub fn sheen(&mut self, val: f32) -> &mut Self {
        self._sheen = val;
        self
    }

    pub fn sheen_tint(&mut self, val: f32) -> &mut Self {
        self._sheen_tint = val;
        self
    }

    pub fn clearcoat(&mut self, val: f32) -> &mut Self {
        self._clearcoat = val;
        self
    }

    pub fn clearcoat_gloss(&mut self, val: f32) -> &mut Self {
        self._clearcoat_gloss = val;
        self
    }
}
