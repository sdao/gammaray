use core;

use std;
use rand;
use rand::Rng;

/// Assuming that we're coming from air into the material.
pub fn schlick_r0_from_ior(ior: f32) -> f32 {
    f32::sqrt(ior - 1.0) / f32::sqrt(ior + 1.0)
}

pub fn schlick_weight(cos_theta: f32) -> f32 {
    let x = core::clamp_unit(1.0 - cos_theta);
    x * x * x * x * x
}

pub fn fresnel_schlick(r0: &core::Vec, cos_theta: f32) -> core::Vec {
    core::Vec::zero().lerp(r0, schlick_weight(cos_theta))
}

/// Quick Fresnel evaluation where ior is real (i.e. dielectrics).
/// See PBRT 3e page 519.
pub fn fresnel_dielectric(mut ior_incident: f32, mut ior_transmit: f32, cos_theta_incident: f32)
    -> core::Vec
{
    let mut cost_i = core::clamp(cos_theta_incident, -1.0, 1.0);

    // Potentially swap indices of reflection if the incident dir is on the inside
    // of the medium.
    let exiting = cost_i < 0.0;
    if exiting {
        std::mem::swap(&mut ior_incident, &mut ior_transmit);
        cost_i = f32::abs(cost_i);
    }

    // Compute Cos[Theta_t] using Snell's law; we'll need to handle total internal
    // reflection as well.
    let sint_i = f32::sqrt(f32::max(0.0f32, 1.0 - cost_i * cost_i));
    let sint_t = ior_incident / ior_transmit * sint_i;
    if sint_t >= 1.0 {
        // Total internal reflection.
        return core::Vec::one();
    }
    let cost_t = f32::sqrt(f32::max(0.0f32, 1.0 - sint_t * sint_t));

    // Actual computation.
    let r_parl = ((ior_transmit * cost_i) - (ior_incident * cost_t)) /
                 ((ior_transmit * cost_i) + (ior_incident * cost_t));
    let r_perp = ((ior_incident * cost_i) - (ior_transmit * cost_t)) /
                 ((ior_incident * cost_i) + (ior_incident * cost_t));
    let x = (r_parl * r_parl + r_perp * r_perp) / 2.0;
    core::Vec::new(x, x, x)
}

pub struct GgxDistribution {
    ax: f32,
    ay: f32
}

impl GgxDistribution {
    pub fn new(roughness: f32, anisotropic: f32) -> GgxDistribution {
        let aspect = f32::sqrt(1.0 - anisotropic * 0.9);
        let ax = f32::max(0.001, f32::sqrt(roughness) / aspect);
        let ay = f32::max(0.001, f32::sqrt(roughness) * aspect);
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

    pub fn lambda(&self, v: &core::Vec) -> f32 {
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

    pub fn g1(&self, v: &core::Vec) -> f32 {
        1.0 / (1.0 + self.lambda(v))
    }

    pub fn g(&self, i: &core::Vec, o: &core::Vec) -> f32 {
        1.0 / (1.0 + self.lambda(i) + self.lambda(o))
    }

    /// Taken from PBRT 3e.
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
