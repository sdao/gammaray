use material::util;

use core;

use std;
use rand;
use rand::distributions::IndependentSample;

pub struct LobeSample {
    pub result: core::Vec,
    pub outgoing: core::Vec,
    pub pdf: f64
}

bitflags! {
    pub struct LobeKind: u32 {
        /// PDF is non-delta-distributed.
        const LOBE_DIFFUSE      = 0b00000001;
        /// PDF is delta-distributed.
        const LOBE_SPECULAR     = 0b00000010;
        /// Out direction is same hemisphere as in direction.
        const LOBE_REFLECTION   = 0b00000100;
        /// Out and in direction are different hemispheres.
        const LOBE_TRANSMISSION = 0b00001000;
    }
}

pub trait Lobe : Sync + Send {
    fn f(&self, i: &core::Vec, o: &core::Vec) -> core::Vec;
    fn pdf(&self, i: &core::Vec, o: &core::Vec) -> f64 {
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
    pub color: core::Vec
}

impl Lobe for DisneyDiffuseRefl {
    fn f(&self, i: &core::Vec, o: &core::Vec) -> core::Vec {
        let f_in = util::schlick(i);
        let f_out = util::schlick(o);
        &self.color * (std::f64::consts::FRAC_1_PI * (1.0 - 0.5 * f_in) * (1.0 - 0.5 * f_out))
    }
}

pub struct DisneyRetroRefl {
    pub color: core::Vec,
    pub roughness: f64
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

        let f_in = util::schlick(i);
        let f_out = util::schlick(o);

        &self.color * (std::f64::consts::FRAC_1_PI * r_r
                * (f_out + f_in + f_out * f_in * (r_r - 1.0)))
    }
}

pub struct DisneySpecularRefl {
    pub color: core::Vec,
    pub metallic: f64,
    pub ior: f64
}

impl DisneySpecularRefl {
    fn d(&self, half: &core::Vec) -> core::Vec {
        core::Vec::zero()
    }

    fn g(&self, i: &core::Vec, o: &core::Vec) -> core::Vec {
        core::Vec::zero()
    }

    fn fresnel(&self, diff_angle: f64) -> core::Vec {
        core::Vec::zero()
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
        let d = self.d(&half);
        let g = self.g(i, o);
        let fresnel = self.fresnel(o.dot(&half));

        &self.color.comp_mult(&d).comp_mult(&g).comp_mult(&fresnel) /
                (4.0 * cos_theta_in * cos_theta_out)
    }
}
