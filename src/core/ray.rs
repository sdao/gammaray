use core::vector;
use std::fmt;
use std::fmt::Display;

pub struct Ray {
    pub origin: vector::Vec,
    pub direction: vector::Vec,
}

impl Ray {
    pub fn new(origin: vector::Vec, direction: vector::Vec) -> Ray {
        Ray {origin: origin, direction: direction}
    }

    pub fn at(&self, k: f64) -> vector::Vec {
        &self.origin + &(k * &self.direction)
    }
}

impl Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ray {{origin: {}, direction: {}}}", self.origin, self.direction)
    }
}
