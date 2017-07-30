use core::bbox;
use core::vector;

use std::fmt;
use std::fmt::Display;

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

    pub fn at(&self, k: f64) -> vector::Vec {
        &self.origin + &(k * &self.direction)
    }

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

    pub fn intersect_bbox(&self, bbox: &bbox::BBox, max_dist: f64, data: &RayIntersectionData)
        -> bool
    {
        // Check for ray intersection against x and y slabs.
        let mut t_min = (bbox[ data.dir_is_neg[0]].x - self.origin.x) * data.inv_dir.x;
        let mut t_max = (bbox[!data.dir_is_neg[0]].x - self.origin.x) * data.inv_dir.x;
        let ty_min =     (bbox[ data.dir_is_neg[1]].y - self.origin.y) * data.inv_dir.y;
        let ty_max =     (bbox[!data.dir_is_neg[1]].y - self.origin.y) * data.inv_dir.y;

        // XXX: May need to use PBRT gamma function to make more numerically stable.
        if t_min > ty_max || ty_min > t_max {
            return false;
        }
        if ty_min > t_min {
            t_min = ty_min;
        }
        if ty_max < t_max {
            t_max = ty_max;
        }

        // Check for ray intersection against $z$ slab
        let tz_min = (bbox[ data.dir_is_neg[2]].z - self.origin.z) * data.inv_dir.z;
        let tz_max = (bbox[!data.dir_is_neg[2]].z - self.origin.z) * data.inv_dir.z;

        if t_min > tz_max || tz_min > t_max {
            return false;
        }
        if tz_max < t_max {
            t_max = tz_max;
        }
        return t_min < max_dist && t_max > 0.0;
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
