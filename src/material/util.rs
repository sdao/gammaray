use core;

pub fn schlick(v: &core::Vec) -> f32 {
    let x = core::clamp_unit(1.0 - v.abs_cos_theta());
    x * x * x * x * x
}
