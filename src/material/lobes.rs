use material::util;
use material::util::Fresnel;
use material::util::MicrofacetDistribution;

use core;

use std;
use std::fmt;
use std::fmt::Display;
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

pub trait Lobe : Display + Sync + Send {
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

/// Implements diffuse, retro-reflection, and sheen for the Disney BRDF.
pub struct DisneyDiffuseRefl {
    color: core::Vec,
    sheen_color: core::Vec,
    roughness: f32,
}

impl DisneyDiffuseRefl {
    pub fn new(color: core::Vec, roughness: f32, sheen: f32, sheen_tint: f32, diffuse_weight: f32)
        -> DisneyDiffuseRefl
    {
        let diffuse_color = &color * diffuse_weight;
        let sheen_color = (sheen * diffuse_weight) *
                &core::Vec::one().lerp(&color.tint(), sheen_tint);
        DisneyDiffuseRefl {color: diffuse_color, sheen_color: sheen_color, roughness: roughness}
    }
}

impl Lobe for DisneyDiffuseRefl {
    fn f(&self, i: &core::Vec, o: &core::Vec) -> core::Vec {
        let f_in = util::fresnel_schlick_weight(i.abs_cos_theta());
        let f_out = util::fresnel_schlick_weight(o.abs_cos_theta());
        let diffuse = &self.color *
                (std::f32::consts::FRAC_1_PI * (1.0 - 0.5 * f_in) * (1.0 - 0.5 * f_out));
        
        let half_unnorm = i + o;
        if half_unnorm.is_exactly_zero() {
            // Retro-reflection and sheen can't be computed.
            return diffuse;
        }

        let half = half_unnorm.normalized();
        let cos_theta_d = o.dot(&half); // Note: could have used i here also.
        let r_r = 2.0 * self.roughness * cos_theta_d * cos_theta_d;

        let retro = &self.color * (std::f32::consts::FRAC_1_PI * r_r
                * (f_out + f_in + f_out * f_in * (r_r - 1.0)));
        let sheen = &self.sheen_color * util::fresnel_schlick_weight(cos_theta_d);

        return &diffuse + &(&retro + &sheen);
    }
}

impl Display for DisneyDiffuseRefl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DisneyDiffuseRefl(color={}, sheen_color={}, roughness={})",
                self.color, self.sheen_color, self.roughness)
    }
}

/// This implementation is derived from the MicrofacetReflection in PBRT 3e.
pub struct StandardMicrofacetRefl<Dist: util::MicrofacetDistribution, Fr: util::Fresnel> {
    microfacet: Dist,
    fresnel: Fr,
    color: core::Vec
}

impl<Dist, Fr> Lobe for StandardMicrofacetRefl<Dist, Fr>
    where Dist: util::MicrofacetDistribution, Fr: util::Fresnel
{
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
        &self.color.comp_mult(&fresnel) * (d * g / (4.0 * cos_theta_out * cos_theta_in))
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
            let o = i.reflect(&half);
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

impl<Dist, Fr> Display for StandardMicrofacetRefl<Dist, Fr>
    where Dist: util::MicrofacetDistribution, Fr: util::Fresnel
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StandardMicrofacetRefl(color={})", self.color)
    }
}

pub struct DisneySpecularRefl {
}

impl DisneySpecularRefl {
    pub fn new(
            color: core::Vec, roughness: f32, ior: f32,
            specular_tint: f32, metallic: f32)
            -> StandardMicrofacetRefl<util::GgxDistribution, util::DisneyFresnel>
    {
        DisneySpecularRefl::new_aniso(
                color, roughness, 0.0, ior, specular_tint, metallic)
    }

    pub fn new_aniso(
            color: core::Vec, roughness: f32, anisotropic: f32, ior: f32,
            specular_tint: f32, metallic: f32)
            -> StandardMicrofacetRefl<util::GgxDistribution, util::DisneyFresnel>
    {
        // Note: The color will be computed by the DisneyFresnel, so we just set it to white on the
        // lobe itself.
        let ior_adjusted = f32::max(ior, 1.01);
        StandardMicrofacetRefl {
            microfacet: util::GgxDistribution::new(roughness, anisotropic),
            fresnel: util::DisneyFresnel::new(ior_adjusted, color, specular_tint, metallic),
            color: core::Vec::one()
        }
    }
}

pub struct DisneyClearcoatRefl {
}

impl DisneyClearcoatRefl {
    pub fn new(clearcoat: f32, clearcoat_gloss: f32)
        -> StandardMicrofacetRefl<util::Gtr1Distribution, util::SchlickFresnel>
    {
        // Note: Disney BRDF: (ior = 1.5 -> F0 = 0.04).
        // Disney also scales the clearcoat amount by 0.25.
        StandardMicrofacetRefl {
            microfacet: util::Gtr1Distribution::new(clearcoat_gloss),
            fresnel: util::SchlickFresnel {r0: 0.04 * &core::Vec::one()},
            color: (0.25 * clearcoat) * &core::Vec::one()
        }
    }
}

/// This implementation is derived from the MicrofacetTransmission in PBRT 3e.
pub struct DisneySpecularTrans {
    microfacet: util::GgxDistribution,
    fresnel: util::DielectricFresnel,
    ior: f32,
    color: core::Vec,
}

impl DisneySpecularTrans {
    pub fn new(color: core::Vec, roughness: f32, ior: f32) -> DisneySpecularTrans {
        DisneySpecularTrans::new_aniso(color, roughness, 0.0, ior)
    }

    pub fn new_aniso(color: core::Vec, roughness: f32, anisotropic: f32, ior: f32)
        -> DisneySpecularTrans
    {
        let ior_adjusted = f32::max(ior, 1.01);
        DisneySpecularTrans {
            microfacet: util::GgxDistribution::new(roughness, anisotropic),
            fresnel: util::DielectricFresnel::new(ior_adjusted),
            ior: ior_adjusted,
            color: color
        }
    }
}

impl Lobe for DisneySpecularTrans {
    fn f(&self, i: &core::Vec, o: &core::Vec) -> core::Vec {
        // This is defined for transmission only.
        if i.is_local_same_hemisphere(&o) {
            return core::Vec::zero();
        }

        let cos_theta_in = i.cos_theta();
        let cos_theta_out = o.cos_theta();
        if cos_theta_in == 0.0 || cos_theta_out == 0.0 {
            return core::Vec::zero();
        }

        let eta = if cos_theta_in > 0.0 {
            // Entering.
            self.ior
        }
        else {
            // Exiting.
            1.0 / self.ior
        };

        let half_unnorm = i + &(o * eta);
        let half = if half_unnorm.z > 0.0 {
            half_unnorm.normalized()
        }
        else {
            -&half_unnorm.normalized()
        };

        debug_assert!(i.is_finite());
        debug_assert!(o.is_finite());
        debug_assert!(half.is_finite(), "{} {} {}", half_unnorm, half, self.ior);

        let fresnel = self.fresnel.fresnel(o.dot(&half));
        let d = self.microfacet.d(&half);
        let g = self.microfacet.g(i, o);

        let sqrt_denom = i.dot(&half) + eta * &o.dot(&half);
        let fresnel_inverse = &core::Vec::one() - &fresnel; // Amount transmitted!

        let res = &self.color.comp_mult(&fresnel_inverse) *
                f32::abs(
                    d * g * f32::abs(o.dot(&half)) * f32::abs(i.dot(&half)) /
                    (cos_theta_out * cos_theta_in * sqrt_denom * sqrt_denom)
                );
        return res;
    }

    fn pdf(&self, i: &core::Vec, o: &core::Vec) -> f32 {
        if i.is_local_same_hemisphere(&o) {
            0.0
        }
        else {
            let eta = if i.cos_theta() > 0.0 {
                // Entering.
                self.ior
            }
            else {
                // Exiting.
                1.0 / self.ior
            };

            // Compute half from i and o for microfacet transmission.
            let half_unnorm = i + &(o * eta);
            let half = if half_unnorm.z > 0.0 {
                half_unnorm
            }
            else {
                -&half_unnorm.normalized()
            };

            // Compute change of variables for microfacet transmission.
            let sqrt_denom = i.dot(&half) + eta * o.dot(&half);
            let dwh_dwi = f32::abs((eta * eta * o.dot(&half)) / (sqrt_denom * sqrt_denom));
            return self.microfacet.pdf(&i, &half) * dwh_dwi;
        }
    }

    fn sample_f(&self, i: &core::Vec, rng: &mut rand::XorShiftRng) -> LobeSample {
        // Sample microfacet orientation (half) and reflected direction (o).
        if i.z == 0.0 {
            LobeSample::zero()
        }
        else {
            let half = self.microfacet.sample_half(i, rng);
            let eta = if i.cos_theta() > 0.0 {
                // Entering.
                1.0 / self.ior
            }
            else {
                // Exiting.
                self.ior
            };

            let o = i.refract(&half, eta);
            debug_assert!(o.is_finite());

            if o.is_exactly_zero() {
                LobeSample::zero()
            }
            else {
                // Compute PDF of outoing vector for microfacet transmission.
                let result = self.f(i, &o);
                let pdf = self.pdf(i, &o);
                debug_assert!(result.is_finite());

                LobeSample {
                    result: result,
                    outgoing: o,
                    pdf: pdf
                }
            }
        }
    }

    fn kind(&self) -> LobeKind {
        LOBE_GLOSSY | LOBE_TRANSMISSION
    }
}

impl Display for DisneySpecularTrans {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DisneySpecularTrans(color={}, ior={})", self.color, self.ior)
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

impl Display for PerfectMirror {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PerfectMirror")
    }
}
