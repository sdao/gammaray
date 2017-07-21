use core::Mat;
use core::Ray;
use core::Vec;

pub trait Prim {
    fn display_color(&self) -> &Vec;
    fn local_to_world_xform(&self) -> &Mat;
    fn world_to_local_xform(&self) -> &Mat;
    /**
     * Intersects the given ray in local space with the prim, and returns the distance along the
     * ray and the normal at the point of intersection.
     */
    fn intersect_local(&self, ray: &Ray) -> (f64, Vec);
    /**
     * Intersects the given ray in world space with the prim, and returns the distance along the
     * ray and the normal at the point of intersection.
     */
    fn intersect_world(&self, ray: &Ray) -> (f64, Vec) {
        let local_ray = self.world_to_local_xform().transform_ray(ray);
        let (dist, normal) = self.intersect_local(&local_ray);
        (dist, self.local_to_world_xform().transform_dir(&normal))
    }
}

mod sphere;
pub use prim::sphere::Sphere;
