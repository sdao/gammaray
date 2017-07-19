use rand::ThreadRng;

mod camera;
pub use core::camera::Camera;

mod intersection;
pub use core::intersection::Intersection;

mod math;
pub use core::math::*;

mod matrix;
pub use core::matrix::Mat4;
pub type Mat4f = matrix::Mat4<f64>;

mod quat;
pub use core::quat::Quat;

mod random;
pub use core::random::AreaSampleDisk;
pub use core::random::CosineSampleHemisphere;
pub use core::random::UniformSampleSphere;
pub use core::random::UniformSampleCone;
pub use core::random::RngHelper;
pub type ThreadRngHelper = random::RngHelper<ThreadRng>;

mod ray;
pub use core::ray::Ray;

mod vector;
pub use core::vector::Vec3;
pub type Vec3f = vector::Vec3<f64>;
pub type Vec3i = vector::Vec3<i32>;
