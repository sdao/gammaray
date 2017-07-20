use core::vector;

use std;
use rand;
use rand::{Rand, Rng, ThreadRng};
use rand::distributions::{IndependentSample, Sample};
use rand::distributions::normal::{Normal, StandardNormal};
use rand::distributions::range::Range;

/** The number of steradians in a sphere (4 * Pi). */
const STERADIANS_PER_SPHERE: f64 = std::f64::consts::PI * 4.0;

/**
 * Samples a unit disk, ensuring that the samples are uniformally distributed
 * throughout the area of the disk.
 *
 * Taken from Pharr & Humphreys' p. 667.
 */
pub struct AreaSampleDisk {
}

impl Sample<(f64, f64)> for AreaSampleDisk {
    fn sample<R>(&mut self, rng: &mut R) -> (f64, f64) where R: Rng {
        self.ind_sample(rng)
    }
}

impl IndependentSample<(f64, f64)> for AreaSampleDisk {
    fn ind_sample<R>(&self, rng: &mut R) -> (f64, f64) where R: Rng {
        let range = Range::new(-1.0, 1.0);
        let sx = range.ind_sample(rng);
        let sy = range.ind_sample(rng);

        // Handle degeneracy at the origin.
        if sx == 0.0 && sy == 0.0 {
            (0.0, 0.0)
        }
        else {
            let r: f64;
            let theta: f64;
            if sx >= -sy {
                if sx > sy {
                    // Region 1.
                    r = sx;
                    if sy > 0.0  {
                        theta = sy / r;
                    } else {
                        theta = 8.0 + sy / r;
                    }
                } else {
                    // Region 2.
                    r = sy;
                    theta = 2.0 - sx / r;
                }
            } else {
                if sx <= sy {
                    // Region 3.
                    r = -sx;
                    theta = 4.0 - sy / r;
                } else {
                    // Region 4.
                    r = -sy;
                    theta = 6.0 + sx / r;
                }
            }
            let theta_pi4 = theta * std::f64::consts::FRAC_PI_4;
            (r * theta_pi4.cos(), r * theta_pi4.sin())
        }
    }
}

/**
 * Samples a unit hemisphere with a cosine-weighted distribution.
 * Directions with a higher cosine value (more parallel to the normal) are
 * more likely to be chosen than those with a lower cosine value (more
 * perpendicular to the normal).
 *
 * Taken from Pharr & Humphreys p. 669.
 *
 * @param flipped whether to sample from the hemisphere on the negative
 *                Z-axis instead; false will sample from the positive
 *                hemisphere and true will sample from the negative hemisphere
 */
pub struct CosineSampleHemisphere {
    flipped: bool,
}

impl CosineSampleHemisphere {
    /**
     * Returns the probability that the given direction was sampled from a unit
     * hemisphere using a cosine-weighted distribution. (It does not matter
     * whether the hemisphere is on the positive or negative Z-axis.)
     */
    pub fn pdf(direction: &vector::Vec) -> f64 {
        direction.abs_cos_theta() * std::f64::consts::FRAC_1_PI
    }
}

impl Sample<vector::Vec> for CosineSampleHemisphere {
    fn sample<R>(&mut self, rng: &mut R) -> vector::Vec where R: Rng {
        self.ind_sample(rng)
    }
}

impl IndependentSample<vector::Vec> for CosineSampleHemisphere {
    fn ind_sample<R>(&self, rng: &mut R) -> vector::Vec where R: Rng {
        const AREA_SAMPLE_DISK: AreaSampleDisk = AreaSampleDisk {};
        let (x, y) = AREA_SAMPLE_DISK.ind_sample(rng);
        let z = f64::max(0.0, 1.0 - x * x - y * y);

        if self.flipped {
            vector::Vec::new(x, y, -1.0 * z)
        }
        else {
            vector::Vec::new(x, y, z)
        }
    }
}

pub struct UniformSampleSphere {
}

/**
 * Uniformly samples from a unit sphere, with respect to the sphere's
 * surface area.
 */
impl UniformSampleSphere {
    /**
     * Returns the probability that any solid angle was sampled uniformly
     * from a unit sphere.
     */
    pub fn pdf() -> f64 {
        1.0 / STERADIANS_PER_SPHERE
    }
}

impl Sample<vector::Vec> for UniformSampleSphere {
    fn sample<R>(&mut self, rng: &mut R) -> vector::Vec where R: Rng {
        self.ind_sample(rng)
    }
}

impl IndependentSample<vector::Vec> for UniformSampleSphere {
    fn ind_sample<R>(&self, rng: &mut R) -> vector::Vec where R: Rng {
        // See MathWorld <http://mathworld.wolfram.com/SpherePointPicking.html>.
        let StandardNormal(x) = rng.gen();
        let StandardNormal(y) = rng.gen();
        let StandardNormal(z) = rng.gen();
        let a = 1.0 / (x * x + y * y + z * z).sqrt();

        vector::Vec::new(a * x, a * y, a * z)
    }
}

/**
 * Generates a random ray in a cone around the positive z-axis, uniformly
 * with respect to solid angle.
 *
 * Handy Mathematica code for checking that this works:
 * \code
 * R[a_] := (h = Cos[Pi/2];
 *   z = RandomReal[{h, 1}];
 *   t = RandomReal[{0, 2*Pi}];
 *   r = Sqrt[1 - z^2];
 *   x = r*Cos[t];
 *   y = r*Sin[t];
 *   {x, y, z})
 *
 * ListPointPlot3D[Map[R, Range[1000]], BoxRatios -> Automatic]
 * \endcode
 *
 * @param half_angle the half-angle of the cone's opening; must be between 0
 *                   and Pi/2 and in radians
 */
pub struct UniformSampleCone {
    half_angle: f64
}

impl UniformSampleCone {
    /**
     * Returns the probability that any solid angle already inside the given
     * cone was sampled uniformly from the cone. The cone is defined by the
     * half-angle of the subtended (apex) angle.
     *
     * @param halfAngle the half-angle of the cone
     * @returns         the probability that the angle was sampled
     */
    pub fn pdf_internal(half_angle: f64) -> f64 {
        let solid_angle = std::f64::consts::PI * 2.0 * (1.0 - half_angle.cos());
        1.0 / solid_angle
    }

    /**
     * Returns the proabability that the given solid angle was sampled
     * uniformly from the given cone. The cone is defined by the half-angle of
     * the subtended (apex) angle. The probability is uniform if the direction
     * is actually in the cone, and zero if it is outside the cone.
     *
     * @param halfAngle the half-angle of the cone
     * @param direction the direction of the sampled vector
     * @returns         the probability that the angle was sampled
     */
    pub fn pdf(half_angle: f64, direction: &vector::Vec) -> f64{
      let cos_half_angle = half_angle.cos();
      let solid_angle = std::f64::consts::PI * 2.0 * (1.0 - cos_half_angle);
      if direction.cos_theta() > cos_half_angle {
          // Within the sampling cone.
          1.0 / solid_angle
      } else {
          // Outside the sampling cone.
          0.0
      }
    }
}

impl Sample<vector::Vec> for UniformSampleCone {
    fn sample<R>(&mut self, rng: &mut R) -> vector::Vec where R: Rng {
        self.ind_sample(rng)
    }
}

impl IndependentSample<vector::Vec> for UniformSampleCone {
    fn ind_sample<R>(&self, rng: &mut R) -> vector::Vec where R: Rng {
        let h = self.half_angle.cos();
        let z = Range::new(h, 1.0).ind_sample(rng);
        let t = Range::new(0.0, std::f64::consts::PI * 2.0).ind_sample(rng);
        let r = (1.0 - (z * z)).sqrt();
        let x = r * t.cos();
        let y = r * t.sin();

        vector::Vec::new(x, y, z)
    }
}
