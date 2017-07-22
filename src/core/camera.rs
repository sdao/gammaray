use core::matrix;
use core::quat;
use core::ray;
use core::vector;

pub const HORIZONTAL_APERTURE_35MM: f64 = 2.2;
pub const VERTICAL_APERTURE_35MM: f64 = 1.6;

/** Perspective camera representation. */
pub struct Camera {
    /**
     * The distance from the eye to the focal plane.
     * A longer focal length means greater magnification, and vice versa.
     */
    pub focal_length: f64,
    /**
     * The width of the projector aperture.
     * This must be in the same units as other scene dimensions, e.g. for a 35mm camera in a
     * cm-world, you should use 3.5cm (but really 3.6cm because it's actually 36mm...).
     */
    pub horizontal_aperture: f64,
    /**
     * The height of the projector aperture. See the documentation for horizontal_aperture
     * for the same unit restrictions. */
    pub vertical_aperture: f64,
    /**
     * The f-number or focal ratio. A larger f-stop gives more depth of field, bringing more
     * objects into focus. A smaller f-stop will narrow the focus around the focal length.
     */
    pub f_stop: f64,
    pub xform: matrix::Mat,
}

impl Camera {
    pub fn default() -> Camera {
        Self::new(5.0, HORIZONTAL_APERTURE_35MM, VERTICAL_APERTURE_35MM, 8.0,
                &quat::Quat::identity(), &vector::Vec::zero())
    }

    pub fn new(
        focal_length: f64,
        horizontal_aperture: f64,
        vertical_aperture: f64,
        f_stop: f64,
        rotate: &quat::Quat,
        translate: &vector::Vec) -> Camera
    {
        let translate_mat = matrix::Mat::translation(&translate);
        let rotate_mat = matrix::Mat::rotation(&rotate);
        let combined = &translate_mat * &rotate_mat;

        Camera {
            focal_length: focal_length,
            horizontal_aperture: horizontal_aperture,
            vertical_aperture: vertical_aperture,
            f_stop: f_stop,
            xform: combined,
        }
    }

    /**
     * The radius of the entrance pupil.
     * See <https://en.wikipedia.org/wiki/F-number> for derivation.
     */
    pub fn pupil_radius(&self) -> f64 {
        0.5 * (self.focal_length / self.f_stop)
    }

    /** Horizontal to vertical ratio. */
    pub fn aspect_ratio(&self) -> f64 {
        self.horizontal_aperture / self.vertical_aperture
    }

    pub fn window_max(&self) -> (f64, f64) {
        (self.horizontal_aperture / (self.focal_length * 2.0),
         self.vertical_aperture / (self.focal_length * 2.0))
    }

    /**
     * Computes the ray starting at the viewpoint and extending through the given window position.
     * The window position is defined in normalized coordinates in [-1, 1] where (0, 0) is the
     * center, (-1, 1) is the lower-left, and (1, 1) is the upper-right.
     */
    pub fn compute_ray(&self, s: f64, t: f64) -> ray::Ray {
        let window_max = self.window_max();
        let origin = vector::Vec::zero();
        let direction = vector::Vec::new(window_max.0 * s, window_max.1 * t, -1.0)
                .normalized();

        let world_origin = self.xform.transform(&origin);
        let world_direction = self.xform.transform_dir(&direction);

        ray::Ray::new(world_origin, world_direction)
    }
}
