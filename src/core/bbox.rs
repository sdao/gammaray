use core::vector;

use std;

#[derive(Clone, Copy)]
pub struct BBox {
    pub min: vector::Vec,
    pub max: vector::Vec
}

impl BBox {
    pub fn empty() -> BBox {
        BBox {
            min: vector::Vec::new(std::f64::MAX, std::f64::MAX, std::f64::MAX),
            max: vector::Vec::new(std::f64::MIN, std::f64::MIN, std::f64::MIN)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y || self.min.z >= self.max.z
    }

    pub fn union_with(&self, k: &vector::Vec) -> BBox {
        BBox {
            min: vector::Vec::new(
                f64::min(self.min.x, k.x), f64::min(self.min.y, k.y), f64::min(self.min.z, k.z)),
            max: vector::Vec::new(
                f64::max(self.max.x, k.x), f64::max(self.max.y, k.y), f64::max(self.max.z, k.z))
        }
    }

    pub fn combine_with(&self, b: &BBox) -> BBox {
        BBox {
            min: vector::Vec::new(
                f64::min(self.min.x, b.min.x),
                f64::min(self.min.y, b.min.y),
                f64::min(self.min.z, b.min.z)),
            max: vector::Vec::new(
                f64::max(self.max.x, b.max.x),
                f64::max(self.max.y, b.max.y),
                f64::max(self.max.z, b.max.z))
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

    pub fn surface_area(&self) -> f64 {
        let d = self.diagonal();
        d.x * d.y * d.z
    }
}
