use core;
use material;

use rand;

pub trait Prim : Sync + Send {
    fn num_components(&self) -> usize {
        1
    }
    fn display_color(&self) -> &core::Vec;
    fn material(&self) -> &material::Material;
    fn local_to_world_xform(&self) -> &core::Xform;
    /**
     * Returns the bounding box in local space for all the geometry in this prim.
     * It's OK to compute this on demand (and not cache the bounding box) because it is the
     * responsibility of callers (such as acceleration structures) to cache the value.
     */
    fn bbox_local(&self, component: usize) -> core::BBox;
    fn bbox_world(&self, component: usize) -> core::BBox {
        self.local_to_world_xform().transform_bbox(&self.bbox_local(component))
    }
    /**
     * Intersects the given ray in local space with the prim, and returns the distance along the
     * ray and the surface properties at the point of intersection.
     * Implementations should be able to handle cases where the incoming ray is not unit length.
     * Implementations also do not have to return unit-length vectors in the SurfaceProperties,
     * although it is recommended.
     */
    fn intersect_local(&self, ray: &core::Ray, component: usize) -> (f32, SurfaceProperties);
    /**
     * Intersects the given ray in world space with the prim, and returns the distance along the
     * ray and the surface properties at the point of intersection.
     */
    fn intersect_world(&self, ray: &core::Ray, component: usize) -> (f32, SurfaceProperties) {
        let xform = self.local_to_world_xform();
        let local_ray = xform.untransform_ray(ray);
        let (dist, surface_props) = self.intersect_local(&local_ray, component);
        if dist == 0.0 {
            (0.0, SurfaceProperties::zero())
        }
        else {
            // Normalize here so that the intersect_local function implementations can return
            // unit-length vectors in their local space, even when the incoming ray is not
            // unit-length in the local space.
            let world_surface_props = SurfaceProperties {
                normal: xform.transform_normal(&surface_props.normal).normalized(),
                tangent: xform.transform_dir(&surface_props.tangent).normalized(),
                binormal: xform.transform_dir(&surface_props.binormal).normalized(),
                geom_normal: xform.transform_normal(&surface_props.geom_normal).normalized()
            };
            (dist, world_surface_props)
        }
    }
    /**
     * Sample a random point on the prim, with respect to the area of the prim.
     * Returns the position, surface properties, and pdf at the sampled point. 
     */
    fn sample_local(&self, rng: &mut rand::XorShiftRng) -> (core::Vec, SurfaceProperties, f32);

    fn sample_world(&self, rng: &mut rand::XorShiftRng) -> (core::Vec, SurfaceProperties, f32) {
        let xform = self.local_to_world_xform();
        let foo = self.sample_local(rng);
        
        let world_pos = xform.transform(&foo.0);
        let surface_props = foo.1;
        let world_surface_props = SurfaceProperties {
                normal: xform.transform_normal(&surface_props.normal).normalized(),
                tangent: xform.transform_dir(&surface_props.tangent).normalized(),
                binormal: xform.transform_dir(&surface_props.binormal).normalized(),
                geom_normal: xform.transform_normal(&surface_props.geom_normal).normalized()
            };

        (world_pos, world_surface_props, foo.2) // XXX pdf scale by area; need to reconsider xform behavior
    }
}

/// Properties of the prim surface at the point of an intersection.
/// The coordinate system formed by normal, tangent, and binormal should satisfy the condition
/// tangent × binormal = normal.
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
