use prim::Prim;
use core;
use core::Intersection;
use core::Mat;
use core::Ray;

pub struct Sphere {
    radius: f64,
    xform: Mat,
    xform_inv: Mat,
}

impl Sphere {
    pub fn new(xform: &Mat, radius: f64) -> Sphere {
        Sphere {radius: radius, xform: xform.clone(), xform_inv: xform.inverted()}
    }
}

impl Prim for Sphere {
    fn local_to_world_xform(&self) -> &Mat {
        &self.xform
    }

    fn world_to_local_xform(&self) -> &Mat {
        &self.xform_inv
    }

    fn intersect_local(&self, ray: &Ray) -> Option<Intersection> {
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

                return Some(Intersection {});
            }
            else if core::is_positive(res_pos) {
                let pt = ray.at(res_pos);
                let normal = if inside {
                    -&pt
                }
                else {
                    pt
                }.normalized();

                return Some(Intersection {});
            }
        }

        // Either no isect was found or it was behind us.
        return None;
    }
}
