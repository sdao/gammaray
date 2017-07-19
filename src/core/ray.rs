use core::vector;
use std::fmt;
use std::fmt::Display;

pub struct Ray {
    pub origin: vector::Vec3<f64>,
    pub direction: vector::Vec3<f64>,
}

impl Ray {
    pub fn new(origin: vector::Vec3<f64>, direction: vector::Vec3<f64>) -> Ray {
        Ray {origin: origin, direction: direction}
    }

    pub fn at(&self, k: f64) -> vector::Vec3<f64> {
        self.origin + k * self.direction
    }
}

impl Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ray {{origin: {}, direction: {}}}", self.origin, self.direction)
    }
}
