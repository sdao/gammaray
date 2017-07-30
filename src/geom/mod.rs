mod bvh;
pub use geom::bvh::{Bvh, Intersection};

mod prim;
pub use geom::prim::{Material, Prim};

mod sphere;
pub use geom::sphere::Sphere;

mod util;
pub use geom::util::*;
