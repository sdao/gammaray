use core;

use std;
use rand;
use rand::Rng;

/// Assuming that we're coming from air into the material.
pub fn fresnel_schlick_weight(cos_theta: f32) -> f32 {
    let x = core::clamp_unit(1.0 - cos_theta);
    x * x * x * x * x
}

pub fn fresnel_schlick(cos_theta: f32, r0: core::Vec) -> core::Vec {
    r0.lerp(&core::Vec::one(), fresnel_schlick_weight(cos_theta))
}

pub fn fresnel_schlick_r0(ior: f32) -> f32 {
    ((ior - 1.0) * (ior - 1.0)) / ((ior + 1.0) * (ior + 1.0))
}

pub fn fresnel_dielectric(cos_theta_in: f32, ior: f32) -> f32 {
    // Potentially swap indices of refraction.
    let entering = cos_theta_in > 0.0;
    let (eta_i, eta_t, cos_theta_in_clamped) = if entering {
        (1.0, ior, core::clamp_unit(cos_theta_in))
    }
    else {
        (ior, 1.0, core::clamp_unit(-cos_theta_in))
    };

    // Compute cos_theta_trans using Snell's law.
    let sin_theta_in = f32::sqrt(
            f32::max(0.0, 1.0 - cos_theta_in_clamped * cos_theta_in_clamped));
    let sin_theta_trans = eta_i / eta_t * sin_theta_in;

    // Handle total internal reflection.
    if sin_theta_trans >= 1.0 {
        1.0
    }
    else {
        let cos_theta_trans = f32::sqrt(
                f32::max(0.0, 1.0 - sin_theta_trans * sin_theta_trans));
        let r_parl = ((eta_t * cos_theta_in_clamped) - (eta_i * cos_theta_trans)) /
                     ((eta_t * cos_theta_in_clamped) + (eta_i * cos_theta_trans));
        let r_perp = ((eta_i * cos_theta_in_clamped) - (eta_t * cos_theta_trans)) /
                     ((eta_i * cos_theta_in_clamped) + (eta_t * cos_theta_trans));
        (r_parl * r_parl + r_perp * r_perp) / 2.0
    }
}

// The Disney Fresnel is a blend of dielectric and metallic models.
pub struct DisneyFresnel {
    color: core::Vec,
    spec_color: core::Vec,
    ior: f32,
    metallic: f32,
}

impl DisneyFresnel {
    pub fn new(ior: f32, color: core::Vec, specular_tint: f32, metallic: f32)
        -> DisneyFresnel
    {
        let spec_color = core::Vec::one().lerp(&color.tint(), specular_tint);
        DisneyFresnel {color: color, spec_color: spec_color, ior: ior, metallic: metallic}
    }

    pub fn fresnel(&self, cos_theta: f32) -> core::Vec {
        let dielectric = &self.spec_color * fresnel_dielectric(cos_theta, self.ior);
        let conductor = fresnel_schlick(cos_theta, self.color);
        dielectric.lerp(&conductor, self.metallic)
    }
}

pub struct FresnelDielectric {
    ior: f32,
}

impl FresnelDielectric {
    pub fn new(ior: f32) -> FresnelDielectric {
        FresnelDielectric {ior: ior}
    }

    pub fn fresnel(&self, cos_theta: f32) -> core::Vec {
        &core::Vec::one() * fresnel_dielectric(cos_theta, self.ior)
    }
}

pub struct FresnelSchlick {
    pub r0: core::Vec,
}

impl FresnelSchlick {
    pub fn new(r0: core::Vec) -> FresnelSchlick {
        FresnelSchlick {r0: r0}
    }

    pub fn fresnel(&self, cos_theta: f32) -> core::Vec {
        fresnel_schlick(cos_theta, self.r0)
    }
}

pub struct GgxDistribution {
    ax: f32,
    ay: f32
}

/// This is based off the TrowbridgeReitzDistribution in PBRT 3e and the
/// Disney BRDF shader source at:
/// https://github.com/wdas/brdf/blob/master/src/brdfs/disney.brdf
impl GgxDistribution {
    pub fn new(roughness: f32, anisotropic: f32) -> GgxDistribution {
        let aspect = f32::sqrt(1.0 - anisotropic * 0.9);
        let ax = f32::max(0.001, roughness * roughness / aspect);
        let ay = f32::max(0.001, roughness * roughness * aspect);
        GgxDistribution {ax: ax, ay: ay}
    }

    pub fn d(&self, half: &core::Vec) -> f32 {
        let tan2_theta = half.tan2_theta();
        if tan2_theta.is_finite() {
            let cos4_theta = half.cos2_theta() * half.cos2_theta();
            let e = (half.cos2_phi() / (self.ax * self.ax) + half.sin2_phi() / (self.ay * self.ay))
                    * tan2_theta;
            1.0 / (std::f32::consts::PI * self.ax * self.ay * cos4_theta * (1.0 + e) * (1.0 + e))
        } else {
            0.0
        }
    }

    fn lambda(&self, v: &core::Vec) -> f32 {
        let abs_tan_theta = f32::abs(v.tan_theta());
        if abs_tan_theta.is_finite() {
            let alpha = f32::sqrt(
                    (v.cos2_phi() * (self.ax * self.ax) + v.sin2_phi() * (self.ay * self.ay)));
            let alpha2_tan2_theta = (alpha * abs_tan_theta) * (alpha * abs_tan_theta);
            (-1.0 + f32::sqrt(1.0 + alpha2_tan2_theta)) * 0.5
        }
        else {
            0.0
        }
    }

    fn g1(&self, v: &core::Vec) -> f32 {
        1.0 / (1.0 + self.lambda(v))
    }

    pub fn g(&self, i: &core::Vec, o: &core::Vec) -> f32 {
        1.0 / (1.0 + self.lambda(i) + self.lambda(o))
    }

    fn sample11(cos_theta: f32, u1: f32, u2: f32) -> (f32, f32) {
        // Special case (normal incidence).
        if cos_theta > 0.9999 {
            let r = f32::sqrt(u1 / (1.0 - u1));
            let phi = core::TWO_PI * u2;
            (r * f32::cos(phi), r * f32::sin(phi))
        }
        else {
            let sin_theta = f32::sqrt(f32::max(0.0, 1.0 - cos_theta * cos_theta));
            let tan_theta = sin_theta / cos_theta;
            let g1 = 2.0 / (1.0 + f32::sqrt(1.0 + tan_theta * tan_theta));

            // Sample x-slope.
            let a = 2.0 * u1 / g1 - 1.0;
            let tmp = f32::min(1.0 / (a * a - 1.0), 1e10);
            let b = tan_theta;
            let d = f32::sqrt(f32::max(b * b * tmp * tmp - (a * a - b * b) * tmp, 0.0));
            let slope_x_1 = b * tmp - d;
            let slope_x_2 = b * tmp + d;
            let slope_x = if a < 0.0 || slope_x_2 > 1.0 / tan_theta {
                slope_x_1
            }
            else {
                slope_x_2
            };
            debug_assert!(slope_x.is_finite());

            // Sample y-slope.
            let (s, u) = if u2 > 0.5 {
                (1.0, 2.0 * (u2 - 0.5))
            }
            else {
                (-1.0, 2.0 * (0.5 - u2))
            };
            let z =
                    (u * (u * (u * 0.27385 - 0.73369) + 0.46341)) /
                    (u * (u * (u * 0.093073 + 0.309420) - 1.000000) + 0.597999);
            let slope_y = s * z * f32::sqrt(1.0 + slope_x * slope_x);
            debug_assert!(slope_y.is_finite());

            (slope_x, slope_y)
        }
    }

    pub fn sample_half(&self, i: &core::Vec, rng: &mut rand::XorShiftRng) -> core::Vec {
        // Flip coordinates so that we're on the same side as the normal.
        let flip = i.z < 0.0;
        let i_flipped = if flip { -i } else { *i };

        // 1. Stretch incoming vector.
        let i_stretched = core::Vec::new(
                self.ax * i_flipped.x, self.ay * i_flipped.y, i_flipped.z).normalized();

        // 2. Simulate P22.
        let cos_theta = i_stretched.cos_theta();
        let u1: f32 = rng.next_f32();
        let u2: f32 = rng.next_f32();
        let (slope_x, slope_y) = GgxDistribution::sample11(cos_theta, u1, u2);

        // 3. Rotate and 4. Unstretch.
        let cos_phi = i_stretched.cos_phi();
        let sin_phi = i_stretched.sin_phi();
        let slope_x_rot = self.ax * (cos_phi * slope_x - sin_phi * slope_y);
        let slope_y_rot = self.ay * (sin_phi * slope_x + cos_phi * slope_y);

        // 5. Compute normal.
        let half = core::Vec::new(-slope_x_rot, -slope_y_rot, 1.0).normalized();

        // Flip coordinates back if necessary.
        if flip { -&half } else { half }
    }

    pub fn pdf(&self, i: &core::Vec, half: &core::Vec) -> f32 {
        let cos_theta = i.cos_theta();
        if cos_theta == 0.0 {
            0.0
        }
        else {
            self.d(half) * self.g1(i) * f32::abs(i.dot(half)) / f32::abs(cos_theta)
        }
    }
}

/// This is derived from the equations in the Disney BRDF paper:
/// http://blog.selfshadow.com/publications/s2012-shading-course/burley/s2012_pbs_disney_brdf_notes_v3.pdf
pub struct Gtr1Distribution {
    alpha: f32
}

impl Gtr1Distribution {
    pub fn new(clearcoat_gloss: f32) -> Gtr1Distribution {
        Gtr1Distribution {
            alpha: core::lerp(0.1, 0.001, clearcoat_gloss)
        }
    }

    pub fn d(&self, half: &core::Vec) -> f32 {
        let alpha2 = self.alpha * self.alpha;
        let cos_theta = half.abs_cos_theta();
        (alpha2 - 1.0) /
                (std::f32::consts::PI *
                f32::ln(alpha2) *
                (1.0 + (alpha2 - 1.0) * cos_theta * cos_theta))
    }

    fn lambda(&self, v: &core::Vec) -> f32 {
        let alpha_g = 0.25; // According to Disney's BRDF, the Gr term uses alpha=0.25.
        let cos_theta = v.abs_cos_theta();

        let alpha2 = alpha_g * alpha_g;
        let cos_theta2 = cos_theta * cos_theta;

        1.0 / (cos_theta + f32::sqrt(alpha2 + cos_theta2 - (alpha2 * cos_theta2)))
    }

    pub fn g(&self, i: &core::Vec, o: &core::Vec) -> f32 {
        1.0 / (1.0 + self.lambda(i) + self.lambda(o))
    }

    pub fn sample_half(&self, i: &core::Vec, rng: &mut rand::XorShiftRng) -> core::Vec {
        let alpha2 = self.alpha * self.alpha;
        let phi = 2.0 * std::f32::consts::PI * rng.next_f32();
        let cos_theta = f32::sqrt(core::clamp_unit(
                (1.0 - f32::powf(alpha2, 1.0 - rng.next_f32()) / (1.0 - alpha2))));
        let h = core::Vec::from_spherical(cos_theta, phi);
        if h.is_local_same_hemisphere(i) {
            h
        }
        else {
            -&h
        }
    }

    pub fn pdf(&self, _: &core::Vec, half: &core::Vec) -> f32 {
        // Sampling exactly follows GTR1, so the pdf is the same as the value.
        self.d(half)
    }
}
