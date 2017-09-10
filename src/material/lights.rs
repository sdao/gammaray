use core;
use geom;

pub trait Light : Sync + Send {
    fn l_world(&self, i: &core::Vec, surface_props: &geom::SurfaceProperties) -> core::Vec;
}

pub struct DiffuseAreaLight {
    pub color: core::Vec
}

impl Light for DiffuseAreaLight {
    fn l_world(&self, i: &core::Vec, surface_props: &geom::SurfaceProperties) -> core::Vec {
        // Only emit light if the vector is on the same side as the normal.
        if i.dot(&surface_props.geom_normal) > 0.0 {
            self.color
        }
        else {
            core::Vec::zero()
        }
    }
}
