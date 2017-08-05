use core;
use material;

pub trait Prim {
    fn num_components(&self) -> usize {
        1
    }
    fn display_color(&self) -> &core::Vec;
    fn material(&self) -> &material::Disney;
    fn local_to_world_xform(&self) -> &core::Mat;
    fn world_to_local_xform(&self) -> &core::Mat;
    fn bbox_local(&self, component: usize) -> core::BBox;
    fn bbox_world(&self, component: usize) -> core::BBox {
        self.local_to_world_xform().transform_bbox(&self.bbox_local(component))
    }
    /**
     * Intersects the given ray in local space with the prim, and returns the distance along the
     * ray and the normal at the point of intersection.
     */
    fn intersect_local(&self, ray: &core::Ray, component: usize) -> (f64, core::Vec);
    /**
     * Intersects the given ray in world space with the prim, and returns the distance along the
     * ray and the normal at the point of intersection.
     */
    fn intersect_world(&self, ray: &core::Ray, component: usize) -> (f64, core::Vec) {
        let local_ray = self.world_to_local_xform().transform_ray(ray);
        let (dist, normal) = self.intersect_local(&local_ray, component);
        (dist, self.local_to_world_xform().transform_dir(&normal))
    }
}
