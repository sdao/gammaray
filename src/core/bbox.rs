use core::math;
use core::ray;
use core::vector;

use std;
use std::ops::{Index, IndexMut};

#[derive(Clone, Copy)]
pub struct BBox {
    pub min: vector::Vec,
    pub max: vector::Vec
}

impl BBox {
    pub fn empty() -> BBox {
        BBox {
            min: vector::Vec::new(std::f32::MAX, std::f32::MAX, std::f32::MAX),
            max: vector::Vec::new(std::f32::MIN, std::f32::MIN, std::f32::MIN)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y || self.min.z >= self.max.z
    }

    pub fn union_with(&self, k: &vector::Vec) -> BBox {
        BBox {
            min: vector::Vec::new(
                f32::min(self.min.x, k.x), f32::min(self.min.y, k.y), f32::min(self.min.z, k.z)),
            max: vector::Vec::new(
                f32::max(self.max.x, k.x), f32::max(self.max.y, k.y), f32::max(self.max.z, k.z))
        }
    }

    pub fn combine_with(&self, b: &BBox) -> BBox {
        BBox {
            min: vector::Vec::new(
                f32::min(self.min.x, b.min.x),
                f32::min(self.min.y, b.min.y),
                f32::min(self.min.z, b.min.z)),
            max: vector::Vec::new(
                f32::max(self.max.x, b.max.x),
                f32::max(self.max.y, b.max.y),
                f32::max(self.max.z, b.max.z))
        }
    }

    pub fn diagonal(&self) -> vector::Vec {
        &self.max - &self.min
    }

    pub fn maximum_extent(&self) -> usize {
        let diagonal = self.diagonal();
        if diagonal.x >= diagonal.y && diagonal.y >= diagonal.z {
            0
        }
        else if diagonal.y >= diagonal.z {
            1
        }
        else {
            2
        }
    }

    // Returns the position of v relative to the corners of the bounding box, where (0, 0, 0)
    // represents the min corner and (1, 1, 1) represents the max corner.
    pub fn relative_offset(&self, v: &vector::Vec) -> vector::Vec {
        let a = v - &self.min;
        let b = self.diagonal();
        a.comp_div(&b)
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.diagonal();
        d.x * d.y * d.z
    }

    pub fn intersect(&self, ray: &ray::Ray, data: &ray::RayIntersectionData, max_dist: f32)
        -> bool
    {
        // Check for ray intersection against x and y slabs.
        let mut t_min =  (self[ data.dir_is_neg[0]].x - ray.origin.x) * data.inv_dir.x;
        let mut t_max =  (self[!data.dir_is_neg[0]].x - ray.origin.x) * data.inv_dir.x;
        let ty_min =     (self[ data.dir_is_neg[1]].y - ray.origin.y) * data.inv_dir.y;
        let mut ty_max = (self[!data.dir_is_neg[1]].y - ray.origin.y) * data.inv_dir.y;

        // Use PBRT gamma function to make more numerically stable.
        t_max *= 1.0 + 2.0 * math::gamma(3.0);
        ty_max *= 1.0 + 2.0 * math::gamma(3.0);
        if t_min > ty_max || ty_min > t_max {
            return false;
        }
        if ty_min > t_min {
            t_min = ty_min;
        }
        if ty_max < t_max {
            t_max = ty_max;
        }

        // Check for ray intersection against z slab.
        let tz_min =     (self[ data.dir_is_neg[2]].z - ray.origin.z) * data.inv_dir.z;
        let mut tz_max = (self[!data.dir_is_neg[2]].z - ray.origin.z) * data.inv_dir.z;

        // Use PBRT gamma function to make more numerically stable.
        tz_max *= 1.0 + 2.0 * math::gamma(3.0);
        if t_min > tz_max || tz_min > t_max {
            return false;
        }
        if tz_max < t_max {
            t_max = tz_max;
        }
        return t_min < max_dist && t_max > 0.0;
    }
}

impl Index<bool> for BBox {
    type Output = vector::Vec;

    fn index(&self, index: bool) -> &vector::Vec {
        if index {
            &self.max
        }
        else {
            &self.min
        }
    }
}

impl IndexMut<bool> for BBox {
    fn index_mut(&mut self, index: bool) -> &mut vector::Vec {
        if index {
            &mut self.max
        }
        else {
            &mut self.min
        }
    }
}
