use geom::prim;

use core;
use material;

use std;
use rand;
use rand::distributions::IndependentSample;

pub struct Sphere {
    mat: material::Material,
    xform: core::Xform,
    radius: f32,
}

impl Sphere {
    pub fn new(material: material::Material, mat: core::Mat, radius: f32) -> Sphere
    {
        Sphere {
            mat: material,
            xform: core::Xform::new(mat),
            radius: radius,
        }
    }
}

impl Sphere {
    fn compute_surface_props(&self, pt: &core::Vec) -> prim::SurfaceProperties {
        // Example: normal = (1, 0, 0)
        //          tangent = (0, 0, -1)
        //          binormal: (0, -1, 0)
        let normal = pt.normalized();
        if core::is_nearly_zero(normal.x) && core::is_nearly_zero(normal.z) {
            // Singularity at top or bottom.
            let tangent = core::Vec::x_axis();
            let binormal = normal.cross(&tangent);
            prim::SurfaceProperties::new(normal, tangent, binormal, normal)
        }
        else {
            // Normal point.
            let tangent = core::Vec::new(-normal.z, 0.0, normal.x).normalized();
            let binormal = normal.cross(&tangent);
            prim::SurfaceProperties::new(normal, tangent, binormal, normal)
        }
    }
}

impl prim::Prim for Sphere {
    fn display_color(&self) -> &core::Vec {
        &self.mat.display_color()
    }

    fn material(&self) -> &material::Material {
        &self.mat
    }

    fn local_to_world_xform(&self) -> &core::Xform {
        &self.xform
    }

    fn bbox_local(&self, _: usize) -> core::BBox {
        core::BBox {
            min: core::Vec::new(-self.radius, -self.radius, -self.radius),
            max: core::Vec::new(self.radius, self.radius, self.radius)
        }
    }

    fn intersect_local(&self, ray: &core::Ray, _: usize) -> (f32, prim::SurfaceProperties) {
        let origin = &ray.origin;
        let l = &ray.direction;

        // See Wikipedia:
        // <http://en.wikipedia.org/wiki/Line%E2%80%93sphere_intersection>
        let a = l.dot(l);
        let b = l.dot(origin);
        let c = origin.dot(origin) - (self.radius * self.radius);

        let discriminant = (b * b) - (a * c);

        if discriminant > 0.0 {
            let sqrt_discriminant = f32::sqrt(discriminant);
            // Quadratic has at most 2 results.
            let res_pos = -b + sqrt_discriminant;
            let res_neg = -b - sqrt_discriminant;

            // Neg before pos because we want to return closest isect first.
            if core::is_positive(res_neg) {
                let pt = ray.at(res_neg);
                return (res_neg, self.compute_surface_props(&pt));
            }
            else if core::is_positive(res_pos) {
                let pt = ray.at(res_pos);
                return (res_pos, self.compute_surface_props(&pt));
            }
        }

        // Either no isect was found or it was behind us.
        return (0.0, prim::SurfaceProperties::zero())
    }

    fn sample_local(&self, rng: &mut rand::XorShiftRng) -> (core::Vec, prim::SurfaceProperties, f32) {
        let uniform_sample_sphere = core::UniformSampleSphere {};
        let pt = &uniform_sample_sphere.ind_sample(rng) * self.radius;
        let surface_props = self.compute_surface_props(&pt);
        let pdf = 1.0 / (4.0 * std::f32::consts::PI * self.radius * self.radius);
        (pt, surface_props, pdf)
    }
}
