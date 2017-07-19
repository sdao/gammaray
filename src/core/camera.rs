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
    /** Position of camera. */
    pub position: vector::Vec3<f64>,
    /**
     * Rotation of the camera. By default, the camera is Z-up, aiming towards the negative X-axis.
     * Any rotation is a rotation from this default position.
     */
    pub rotation: quat::Quat,
    /**
     * The f-number or focal ratio. A larger f-stop gives more depth of field, bringing more
     * objects into focus. A smaller f-stop will narrow the focus around the focal length.
     */
    pub f_stop: f64,
}

impl Camera {
    pub fn default() -> Camera {
        Camera {
            focal_length: 5.0,
            horizontal_aperture: HORIZONTAL_APERTURE_35MM,
            vertical_aperture: VERTICAL_APERTURE_35MM,
            position: vector::Vec3::<f64>::zero(),
            rotation: quat::Quat::identity(),
            f_stop: 8.0,
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

    pub fn view_matrix(&self) -> matrix::Mat4<f64> {
        let mut m = matrix::Mat4::<f64>::zero();
        m.set_look_at(self.position, self.rotation);
        m
    }

    /**
     * Computes the ray starting at the viewpoint and extending through the given window position.
     * The window position is defined in normalized coordinates in [-1, 1] where (0, 0) is the
     * center, (-1, 1) is the lower-left, and (1, 1) is the upper-right.
     */
    pub fn compute_ray(&self, x: f64, y: f64) -> ray::Ray {
        let window_max = self.window_max();
        let origin = vector::Vec3::<f64>::zero();
        let direction = vector::Vec3::<f64>::new(window_max.0 * x, window_max.1 * y, -1.0)
                .normalized();

        let view_inverse = self.view_matrix().inverted();
        let world_origin = view_inverse.transform(origin);
        let world_direction = view_inverse.transform_dir(direction);

        ray::Ray::new(world_origin, world_direction)
    }
}
