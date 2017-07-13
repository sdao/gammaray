use rand::ThreadRng;

mod math;
pub use core::math::*;

mod random;
pub use core::random::AreaSampleDisk;
pub use core::random::CosineSampleHemisphere;
pub use core::random::UniformSampleSphere;
pub use core::random::UniformSampleCone;
pub use core::random::RngHelper;
pub type ThreadRngHelper = random::RngHelper<ThreadRng>;

mod vector;
pub use core::vector::Vec3;
pub type Vec3f = vector::Vec3<f64>;
pub type Vec3i = vector::Vec3<i32>;
