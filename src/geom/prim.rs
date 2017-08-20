use core;
use material;

pub trait Prim : Sync + Send {
    fn num_components(&self) -> usize {
        1
    }
    fn display_color(&self) -> &core::Vec;
    fn material(&self) -> &material::Material;
    fn local_to_world_xform(&self) -> &core::Mat;
    fn world_to_local_xform(&self) -> &core::Mat;
    fn bbox_local(&self, component: usize) -> core::BBox;
    fn bbox_world(&self, component: usize) -> core::BBox {
        self.local_to_world_xform().transform_bbox(&self.bbox_local(component))
    }
    /**
     * Intersects the given ray in local space with the prim, and returns the distance along the
     * ray and the surface properties at the point of intersection.
     */
    fn intersect_local(&self, ray: &core::Ray, component: usize) -> (f32, SurfaceProperties);
    /**
     * Intersects the given ray in world space with the prim, and returns the distance along the
     * ray and the surface properties at the point of intersection.
     */
    fn intersect_world(&self, ray: &core::Ray, component: usize) -> (f32, SurfaceProperties) {
        let local_ray = self.world_to_local_xform().transform_ray(ray);
        let (dist, surface_props) = self.intersect_local(&local_ray, component);
        if dist == 0.0 {
            (dist, surface_props)
        }
        else {
            let world_surface_props = SurfaceProperties {
                normal: self.local_to_world_xform().transform_dir(&surface_props.normal),
                tangent: self.local_to_world_xform().transform_dir(&surface_props.tangent),
                binormal: self.local_to_world_xform().transform_dir(&surface_props.binormal),
                geom_normal: self.local_to_world_xform().transform_dir(&surface_props.geom_normal)
            };
            (dist, world_surface_props)
        }
    }
}

pub struct SurfaceProperties {
    pub normal: core::Vec,
    pub tangent: core::Vec,
    pub binormal: core::Vec,
    pub geom_normal: core::Vec,
}

impl SurfaceProperties {
    pub fn new(normal: core::Vec, tangent: core::Vec, binormal: core::Vec, geom_normal: core::Vec)
        -> SurfaceProperties
    {
        SurfaceProperties {
            normal: normal,
            tangent: tangent,
            binormal: binormal,
            geom_normal: geom_normal
        }
    }
    pub fn zero() -> SurfaceProperties {
        Self::new(core::Vec::zero(), core::Vec::zero(), core::Vec::zero(), core::Vec::zero())
    }
}
