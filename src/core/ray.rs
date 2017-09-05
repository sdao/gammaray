use core::vector;

use std::fmt;
use std::fmt::Display;

/// The distance to push the origin of each new ray along the normal.
/// XXX: PBRT says that we should be reprojecting and computing an error bound instead.
/// XXX: Seems like we need around 1e-3 for floats and 1e-6 for doubles if hardcoding.
pub const RAY_PUSH_DIST: f32 = 1.0e-3;

#[derive(Clone)]
pub struct Ray {
    pub origin: vector::Vec,
    pub direction: vector::Vec,
}

impl Ray {
    pub fn new(origin: vector::Vec, direction: vector::Vec) -> Ray {
        Ray {origin: origin, direction: direction}
    }

    pub fn zero() -> Ray {
        Ray {origin: vector::Vec::zero(), direction: vector::Vec::zero()}
    }

    pub fn at(&self, k: f32) -> vector::Vec {
        &self.origin + &(k * &self.direction)
    }

    /// Pre-computes some data used to accelerate intersection computations.
    pub fn compute_intersection_data(&self) -> RayIntersectionData {
        RayIntersectionData {
            inv_dir: vector::Vec::new(
                1.0 / self.direction.x,
                1.0 / self.direction.y,
                1.0 / self.direction.z),
            dir_is_neg: [
                self.direction.x < 0.0,
                self.direction.y < 0.0,
                self.direction.z < 0.0]
        }
    }

    pub fn nudge(&self) -> Ray {
        Ray {origin: &self.origin + &(&self.direction * RAY_PUSH_DIST), direction: self.direction}
    }
}

impl Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ray {{origin: {}, direction: {}}}", self.origin, self.direction)
    }
}

pub struct RayIntersectionData {
    pub inv_dir: vector::Vec,
    pub dir_is_neg: [bool; 3]
}
