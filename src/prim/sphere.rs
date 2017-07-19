use prim::Prim;
use core;
use core::Intersection;
use core::Mat4f;
use core::Ray;

pub struct Sphere {
    pub xform: Mat4f,
    pub radius: f64,
    pub inverted: bool,
}

impl Sphere {
    pub fn new(xform: &Mat4f, radius: f64) -> Sphere {
        Sphere {xform: xform.clone(), radius: radius, inverted: false}
    }
}

impl Prim for Sphere {
    fn local_to_world_xform(&self) -> &Mat4f {
        &self.xform
    }

    fn intersect_local(&self, ray: &Ray) -> Option<Intersection> {
        let diff = &ray.origin;
        let l = &ray.direction;

        // See Wikipedia:
        // <http://en.wikipedia.org/wiki/Line%E2%80%93sphere_intersection>
        let a = l.dot(l);
        let b = l.dot(diff);
        let c = diff.dot(diff) - (self.radius * self.radius);

        let discriminant = (b * b) - (a * c);

        if discriminant > 0.0 {
            let sqrt_discriminant = discriminant.sqrt();
            // Quadratic has at most 2 results.
            let resPos = -b + sqrt_discriminant;
            let resNeg = -b - sqrt_discriminant;

            // Neg before pos because we want to return closest isect first.
            if core::is_positive(resNeg) {
                let pt = ray.at(resNeg);
                let normal = if self.inverted {
                    -pt
                }
                else {
                    pt
                }.normalized();

                return Some(Intersection {});
            }
            else if core::is_positive(resPos) {
                let pt = ray.at(resPos);
                let normal = if self.inverted {
                    -pt
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
