use core;
use material;

use rand;
use rand::distributions::IndependentSample;

pub trait Prim : Sync + Send {
    fn num_components(&self) -> usize {
        1
    }
    fn display_color(&self) -> &core::Vec;
    fn material(&self) -> &material::Material;
    /**
     * Returns the bounding box in world space for all the geometry in this prim.
     * It's OK to compute this on demand (and not cache the bounding box) because it is the
     * responsibility of callers (such as acceleration structures) to cache the value.
     */
    fn bbox_world(&self, component: usize) -> core::BBox;
    /**
     * Intersects the given ray in world space with the prim, and returns the distance along the
     * ray and the surface properties at the point of intersection.
     * Implementations should be able to handle cases where the incoming ray is not unit length.
     * Implementations also do not have to return unit-length vectors in the SurfaceProperties,
     * although it is recommended.
     */
    fn intersect_world(&self, ray: &core::Ray, component: usize) -> (f32, SurfaceProperties);
    /**
     * Sample a random point in world space on the prim, with respect to the area of the prim.
     * Returns the position, surface properties, and pdf at the sampled point.
     */
    fn sample_world(&self, rng: &mut rand::XorShiftRng) -> (core::Vec, SurfaceProperties, f32);
    /**
     * Sample a random ray starting from a random point on the prim.
     * Returns the ray, surface properties at the origin, the pdf of the origin position, and the
     * pdf of the ray direction.
     */
    fn sample_ray_world(&self, rng: &mut rand::XorShiftRng)
        -> (core::Ray, SurfaceProperties, f32, f32)
    {
        let (point, surface_props, point_pdf) = self.sample_world(rng);

        let cosine_sample_hemis = core::CosineSampleHemisphere {flipped: false};
        let dir = cosine_sample_hemis.ind_sample(rng);
        let dir_pdf = core::CosineSampleHemisphere::pdf(&dir);

        let (tangent, binormal) = surface_props.normal.coord_system();
        let dir_world = dir.local_to_world(&tangent, &binormal, &surface_props.normal);

        let light_ray = core::Ray::new(point, dir_world);
        (light_ray, surface_props, point_pdf, dir_pdf)
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
