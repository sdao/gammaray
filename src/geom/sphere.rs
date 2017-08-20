use geom::prim;

use core;
use material;

pub struct Sphere {
    mat: material::Material,
    radius: f32,
    xform: core::Mat,
    xform_inv: core::Mat,
}

impl Sphere {
    pub fn new(material: material::Material, xform: core::Mat, radius: f32) -> Sphere
    {
        let xf = xform;
        let inverted = xf.inverted();
        Sphere {
            mat: material,
            radius: radius,
            xform: xf,
            xform_inv: inverted
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

    fn local_to_world_xform(&self) -> &core::Mat {
        &self.xform
    }

    fn world_to_local_xform(&self) -> &core::Mat {
        &self.xform_inv
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
                let normal = pt.normalized();
                let surface_props = prim::SurfaceProperties::new(
                        normal, core::Vec::zero(), core::Vec::zero(), normal);
                return (res_neg, surface_props);
            }
            else if core::is_positive(res_pos) {
                let pt = ray.at(res_pos);
                let normal = pt.normalized();
                let surface_props = prim::SurfaceProperties::new(
                        normal, core::Vec::zero(), core::Vec::zero(), normal);
                return (res_pos, surface_props);
            }
        }

        // Either no isect was found or it was behind us.
        return (0.0, prim::SurfaceProperties::zero())
    }
}
