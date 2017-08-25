use core;

pub trait Light : Sync + Send {
    fn l(&self, i: &core::Vec) -> core::Vec;
}

pub struct DiffuseAreaLight {
    pub color: core::Vec
}

impl Light for DiffuseAreaLight {
    fn l(&self, i: &core::Vec) -> core::Vec {
        // Only emit light if the vector is on the same side as the normal.
        if i.z > 0.0 {
            self.color
        }
        else {
            core::Vec::zero()
        }
    }
}

pub struct NullLight {
}

impl Light for NullLight {
    fn l(&self, _: &core::Vec) -> core::Vec {
        core::Vec::zero()
    }
}
