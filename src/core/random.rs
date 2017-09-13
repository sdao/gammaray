use core::vector;

use std;
use rand;
use rand::{Rng, SeedableRng};
use rand::distributions::{IndependentSample, Sample};
use rand::distributions::normal::StandardNormal;
use rand::distributions::range::Range;

/** The number of steradians in a sphere (4 * Pi). */
const STERADIANS_PER_SPHERE: f32 = std::f32::consts::PI * 4.0;

pub fn new_xor_shift_rng() -> rand::XorShiftRng {
    let mut thread_rng = rand::thread_rng();
    rand::XorShiftRng::from_seed([
            thread_rng.next_u32(),
            thread_rng.next_u32(),
            thread_rng.next_u32(),
            thread_rng.next_u32()])
}

/**
 * Samples a unit disk, ensuring that the samples are uniformally distributed
 * throughout the area of the disk.
 *
 * Taken from Pharr & Humphreys' p. 667.
 */
pub struct AreaSampleDisk {
}

impl Sample<(f32, f32)> for AreaSampleDisk {
    fn sample<R>(&mut self, rng: &mut R) -> (f32, f32) where R: Rng {
        self.ind_sample(rng)
    }
}

impl IndependentSample<(f32, f32)> for AreaSampleDisk {
    fn ind_sample<R>(&self, rng: &mut R) -> (f32, f32) where R: Rng {
        let range = Range::new(-1.0, 1.0);
        let sx: f32 = range.ind_sample(rng);
        let sy: f32 = range.ind_sample(rng);

        // Handle degeneracy at the origin.
        if sx == 0.0 && sy == 0.0 {
            (0.0, 0.0)
        }
        else {
            let (r, theta) = if f32::abs(sx) > f32::abs(sy) {
                (sx, std::f32::consts::FRAC_PI_4 * (sy / sx))
            }
            else {
                (sy, std::f32::consts::FRAC_PI_2 - std::f32::consts::FRAC_PI_4 * (sx / sy))
            };

            (r * f32::cos(theta), r * f32::sin(theta))
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
    pub flipped: bool,
}

impl CosineSampleHemisphere {
    /**
     * Returns the probability that the given direction was sampled from a unit
     * hemisphere using a cosine-weighted distribution. (It does not matter
     * whether the hemisphere is on the positive or negative Z-axis.)
     */
    pub fn pdf(direction: &vector::Vec) -> f32 {
        direction.abs_cos_theta() * std::f32::consts::FRAC_1_PI
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
        let z = f32::sqrt(f32::max(0.0, 1.0 - x * x - y * y));

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
    pub fn pdf() -> f32 {
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
        let x = {
            let StandardNormal(x) = rng.gen();
            x as f32
        };
        let y = {
            let StandardNormal(y) = rng.gen();
            y as f32
        };
        let z = {
            let StandardNormal(z) = rng.gen();
            z as f32
        };
        let a = 1.0 / f32::sqrt(x * x + y * y + z * z);

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
    half_angle: f32
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
    pub fn pdf_internal(half_angle: f32) -> f32 {
        let solid_angle = std::f32::consts::PI * 2.0 * (1.0 - f32::cos(half_angle));
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
    pub fn pdf(half_angle: f32, direction: &vector::Vec) -> f32{
      let cos_half_angle = f32::cos(half_angle);
      let solid_angle = std::f32::consts::PI * 2.0 * (1.0 - cos_half_angle);
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
        let h = f32::cos(self.half_angle);
        let z = Range::new(h, 1.0).ind_sample(rng);
        let t = Range::new(0.0, std::f32::consts::PI * 2.0).ind_sample(rng);
        let r = f32::sqrt(1.0 - (z * z));
        let x = r * f32::cos(t);
        let y = r * f32::sin(t);

        vector::Vec::new(x, y, z)
    }
}

pub struct CumulativeDistribution {
    cdf: std::vec::Vec<f32>,
}

impl CumulativeDistribution {
    pub fn new(cdf: std::vec::Vec<f32>) -> CumulativeDistribution {
        CumulativeDistribution {cdf: cdf}
    }
}

impl Sample<usize> for CumulativeDistribution {
    fn sample<R>(&mut self, rng: &mut R) -> usize where R: Rng {
        self.ind_sample(rng)
    }
}

impl IndependentSample<usize> for CumulativeDistribution {
    fn ind_sample<R>(&self, rng: &mut R) -> usize where R: Rng {
        let target = rng.next_f32();
        match self.cdf.binary_search_by(|x| x.partial_cmp(&target).unwrap()) {
            Ok(x) => x,
            Err(x) => x
        }
    }
}

/// Uniformly samples barycentric coordinates for a triangle.
pub struct UniformSampleBarycentric {
}

impl Sample<(f32, f32)> for UniformSampleBarycentric {
    fn sample<R>(&mut self, rng: &mut R) -> (f32, f32) where R: Rng {
        self.ind_sample(rng)
    }
}

impl IndependentSample<(f32, f32)> for UniformSampleBarycentric {
    fn ind_sample<R>(&self, rng: &mut R) -> (f32, f32) where R: Rng {
        let (a, b) = (rng.next_f32(), rng.next_f32());
        let sqrt_a = f32::sqrt(a);
        (1.0 - sqrt_a, b * sqrt_a)
    }
}
