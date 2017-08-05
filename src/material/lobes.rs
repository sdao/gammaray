use material::disney;
use material::util;

use core;

use std;
use rand;
use rand::distributions::IndependentSample;

pub fn sample_diffuse(disney: &disney::Disney, i: &core::Vec, rng: &mut rand::XorShiftRng)
    -> disney::BxdfSample
{
    let cosine_sample_hemis = core::CosineSampleHemisphere {flipped: i.z < 0.0};
    let o = cosine_sample_hemis.ind_sample(rng);
    let pdf = core::CosineSampleHemisphere::pdf(&o);
    disney::BxdfSample {
        result: f_d(disney, i, &o),
        outgoing: o,
        pdf: pdf
    }
}

fn f_d(disney: &disney::Disney, i: &core::Vec, o: &core::Vec) -> core::Vec {
    &f_lambert(disney, i, o) + &f_retro(disney, i, o)
}

fn f_lambert(disney: &disney::Disney, i: &core::Vec, o: &core::Vec) -> core::Vec {
    let f_in = util::schlick(i);
    let f_out = util::schlick(o);
    &disney.base_color * (std::f64::consts::FRAC_1_PI * (1.0 - 0.5 * f_in) * (1.0 - 0.5 * f_out))
}

fn f_retro(disney: &disney::Disney, i: &core::Vec, o: &core::Vec) -> core::Vec {
    let half = i + o;
    if half.is_exactly_zero() {
        return core::Vec::zero();
    }

    let cos_theta_d = i.dot(&half.normalized()); // Note: could have used o here also.
    let r_r = 2.0 * disney.roughness * cos_theta_d * cos_theta_d;

    let f_in = util::schlick(i);
    let f_out = util::schlick(o);

    &disney.base_color * (std::f64::consts::FRAC_1_PI * r_r
            * (f_out + f_in + f_out * f_in * (r_r - 1.0)))
}
