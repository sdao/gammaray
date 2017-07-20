use core::Intersection;
use core::Mat;
use core::Ray;

pub trait Prim {
    fn local_to_world_xform(&self) -> &Mat;
    fn intersect_local(&self, ray: &Ray) -> Option<Intersection>;
    fn intersect_world(&self, ray: &Ray) -> Option<Intersection> {
        let world_to_local_xform = self.local_to_world_xform().inverted();
        let local_ray = world_to_local_xform.transform_ray(ray);
        self.intersect_local(&local_ray)
    }
}

mod sphere;
pub use prim::sphere::Sphere;
