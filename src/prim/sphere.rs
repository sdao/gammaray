use prim;

use core;

pub struct Sphere {
    color: core::Vec,
    radius: f64,
    xform: core::Mat,
    xform_inv: core::Mat,
}

impl Sphere {
    pub fn new(display_color: &core::Vec, xform: &core::Mat, radius: f64) -> Sphere {
        Sphere {
            color: display_color.clone(),
            radius: radius, xform: xform.clone(),
            xform_inv: xform.inverted()
        }
    }
}

impl prim::Prim for Sphere {
    fn display_color(&self) -> &core::Vec {
        &self.color
    }

    fn local_to_world_xform(&self) -> &core::Mat {
        &self.xform
    }

    fn world_to_local_xform(&self) -> &core::Mat {
        &self.xform_inv
    }

    fn intersect_local(&self, ray: &core::Ray) -> (f64, core::Vec) {
        let origin = &ray.origin;
        let l = &ray.direction;

        // See Wikipedia:
        // <http://en.wikipedia.org/wiki/Line%E2%80%93sphere_intersection>
        let a = l.dot(l);
        let b = l.dot(origin);
        let c = origin.dot(origin) - (self.radius * self.radius);

        let discriminant = (b * b) - (a * c);

        if discriminant > 0.0 {
            let inside = c < 0.0;
            let sqrt_discriminant = discriminant.sqrt();
            // Quadratic has at most 2 results.
            let res_pos = -b + sqrt_discriminant;
            let res_neg = -b - sqrt_discriminant;

            // Neg before pos because we want to return closest isect first.
            if core::is_positive(res_neg) {
                let pt = ray.at(res_neg);
                let normal = if inside {
                    -&pt
                }
                else {
                    pt
                }.normalized();

                return (res_neg, normal)
            }
            else if core::is_positive(res_pos) {
                let pt = ray.at(res_pos);
                let normal = if inside {
                    -&pt
                }
                else {
                    pt
                }.normalized();

                return (res_neg, normal)
            }
        }

        // Either no isect was found or it was behind us.
        return (0.0, core::Vec::zero())
    }
}
