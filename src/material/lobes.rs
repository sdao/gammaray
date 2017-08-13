use material::util;

use core;

use std;
use rand;
use rand::distributions::IndependentSample;

pub struct LobeSample {
    pub result: core::Vec,
    pub outgoing: core::Vec,
    pub pdf: f32
}

impl LobeSample {
    pub fn zero() -> LobeSample {
        LobeSample {result: core::Vec::zero(), outgoing: core::Vec::zero(), pdf: 0.0}
    }
}

bitflags! {
    pub struct LobeKind: u32 {
        /// PDF is non-delta-distributed.
        const LOBE_DIFFUSE      = 0b00000001;
        /// PDF is delta-distributed.
        const LOBE_SPECULAR     = 0b00000010;
        const LOBE_GLOSSY       = 0b00000100;
        /// Out direction is same hemisphere as in direction.
        const LOBE_REFLECTION   = 0b00001000;
        /// Out and in direction are different hemispheres.
        const LOBE_TRANSMISSION = 0b00010000;
    }
}

pub trait Lobe : Sync + Send {
    fn f(&self, i: &core::Vec, o: &core::Vec) -> core::Vec;

    fn pdf(&self, i: &core::Vec, o: &core::Vec) -> f32 {
        if !i.is_local_same_hemisphere(o) {
            0.0
        }
        else {
            core::CosineSampleHemisphere::pdf(&o)
        }
    }

    fn sample_f(&self, i: &core::Vec, rng: &mut rand::XorShiftRng) -> LobeSample {
        // Take a sample direction on the same side of the normal as the incoming direction.
        let cosine_sample_hemis = core::CosineSampleHemisphere {flipped: i.z < 0.0};
        let o = cosine_sample_hemis.ind_sample(rng);
        let result = self.f(i, &o);
        let pdf = self.pdf(i, &o);

        LobeSample {
            result: result,
            outgoing: o,
            pdf: pdf
        }
    }

    fn kind(&self) -> LobeKind {
        LOBE_DIFFUSE | LOBE_REFLECTION
    }
}

pub struct DisneyDiffuseRefl {
    color: core::Vec
}

impl DisneyDiffuseRefl {
    pub fn new(color: core::Vec) -> DisneyDiffuseRefl {
        DisneyDiffuseRefl {color: color}
    }
}

impl Lobe for DisneyDiffuseRefl {
    fn f(&self, i: &core::Vec, o: &core::Vec) -> core::Vec {
        let f_in = util::fresnel_schlick_weight(i.abs_cos_theta());
        let f_out = util::fresnel_schlick_weight(o.abs_cos_theta());
        &self.color * (std::f32::consts::FRAC_1_PI * (1.0 - 0.5 * f_in) * (1.0 - 0.5 * f_out))
    }
}

pub struct DisneyRetroRefl {
    color: core::Vec,
    roughness: f32
}

impl DisneyRetroRefl {
    pub fn new(color: core::Vec, roughness: f32) -> DisneyRetroRefl {
        DisneyRetroRefl {color: color, roughness: roughness}
    }
}

impl Lobe for DisneyRetroRefl {
    fn f(&self, i: &core::Vec, o: &core::Vec) -> core::Vec {
        let half_unnorm = i + o;
        if half_unnorm.is_exactly_zero() {
            return core::Vec::zero();
        }

        let half = half_unnorm.normalized();
        let cos_theta_d = o.dot(&half); // Note: could have used i here also.
        let r_r = 2.0 * self.roughness * cos_theta_d * cos_theta_d;

        let f_in = util::fresnel_schlick_weight(i.abs_cos_theta());
        let f_out = util::fresnel_schlick_weight(o.abs_cos_theta());

        &self.color * (std::f32::consts::FRAC_1_PI * r_r
                * (f_out + f_in + f_out * f_in * (r_r - 1.0)))
    }
}

pub struct DisneySpecularRefl {
    microfacet: util::GgxDistribution,
    fresnel: util::DisneyFresnel
}

impl DisneySpecularRefl {
    pub fn new(
            color: core::Vec, roughness: f32, ior: f32, specular: f32, specular_tint: f32,
            metallic: f32) -> DisneySpecularRefl
    {
        DisneySpecularRefl::new_aniso(
                color, roughness, 0.0, ior, specular, specular_tint, metallic)
    }

    pub fn new_aniso(
            color: core::Vec, roughness: f32, anisotropic: f32, ior: f32, specular: f32,
            specular_tint: f32, metallic: f32) -> DisneySpecularRefl
    {
        // XXX: We don't actually have proper tangents on surfaces, so anisotropy isn't going
        // to really make sense at the moment.
        DisneySpecularRefl {
            microfacet: util::GgxDistribution::new(roughness, anisotropic),
            fresnel: util::DisneyFresnel::new(ior, color, specular, specular_tint, metallic)
        }
    }
}

impl Lobe for DisneySpecularRefl {
    fn f(&self, i: &core::Vec, o: &core::Vec) -> core::Vec {
        let cos_theta_in = i.abs_cos_theta();
        let cos_theta_out = o.abs_cos_theta();
        let half_unnorm = i + o;
        if half_unnorm.is_exactly_zero() || cos_theta_in == 0.0 || cos_theta_out == 0.0 {
            return core::Vec::zero();
        }

        let half = half_unnorm.normalized();
        let fresnel = self.fresnel.fresnel(o.dot(&half));
        let d = self.microfacet.d(&half);
        let g = self.microfacet.g(i, o);
        &fresnel * (d * g / (4.0 * cos_theta_out * cos_theta_in))
    }

    fn pdf(&self, i: &core::Vec, o: &core::Vec) -> f32 {
        if !i.is_local_same_hemisphere(o) {
            0.0
        }
        else {
            let half = (i + o).normalized();
            self.microfacet.pdf(i, &half) / (4.0 * i.dot(&half))
        }
    }

    fn sample_f(&self, i: &core::Vec, rng: &mut rand::XorShiftRng) -> LobeSample {
        // Sample microfacet orientation (half) and reflected direction (o).
        if i.z == 0.0 {
            LobeSample::zero()
        }
        else {
            let half = self.microfacet.sample_half(i, rng);
            let o = (-i).reflect(&half);
            if !i.is_local_same_hemisphere(&o) {
                LobeSample::zero()
            }
            else {
                // Compute PDF of outoing vector for microfacet reflection.
                let result = self.f(i, &o);
                let pdf = self.microfacet.pdf(i, &half) / (4.0 * i.dot(&half));
                LobeSample {
                    result: result,
                    outgoing: o,
                    pdf: pdf
                }
            }
        }
    }

    fn kind(&self) -> LobeKind {
        LOBE_GLOSSY | LOBE_REFLECTION
    }
}

pub struct PerfectMirror {
}

impl PerfectMirror {
    pub fn new() -> PerfectMirror {
        PerfectMirror {}
    }
}

impl Lobe for PerfectMirror {
    fn f(&self, _: &core::Vec, o: &core::Vec) -> core::Vec {
        &core::Vec::one() / o.abs_cos_theta()
    }

    fn pdf(&self, _: &core::Vec, _: &core::Vec) -> f32 {
        1.0
    }

    fn sample_f(&self, i: &core::Vec, _: &mut rand::XorShiftRng) -> LobeSample {
        let o = core::Vec::new(-i.x, -i.y, i.z);
        let result = self.f(i, &o);
        let pdf = self.pdf(i, &o);

        LobeSample {
            result: result,
            outgoing: o,
            pdf: pdf
        }
    }

    fn kind(&self) -> LobeKind {
        LOBE_SPECULAR | LOBE_REFLECTION
    }
}
