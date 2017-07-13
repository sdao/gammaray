extern crate gammaray;
use gammaray::core;

fn main() {
    let i = -core::Vec3f::new(1.0, 2.0, 3.0);
    let j = core::Vec3i::new(1, 2, 3);
    println!("My variable: {}", i.magnitude());

    println!("My variable: {}", core::clamp(0.3, 0.5, 0.7));
}
