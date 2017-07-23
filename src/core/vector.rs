use core::math;

use std::fmt;
use std::fmt::Display;
use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Clone)]
pub struct Vec {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec {
    pub fn new(x: f64, y: f64, z: f64) -> Vec {
        Vec {x: x, y: y, z: z}
    }

    pub fn zero() -> Vec { Self::new(0.0, 0.0, 0.0) }
    pub fn one() -> Vec { Self::new(1.0, 1.0, 1.0) }

    pub fn x_axis() -> Vec { Self::new(1.0, 0.0, 0.0) }
    pub fn y_axis() -> Vec { Self::new(0.0, 1.0, 0.0) }
    pub fn z_axis() -> Vec { Self::new(0.0, 0.0, 1.0) }

    pub fn red() -> Vec { Self::x_axis() }
    pub fn green() -> Vec{ Self::y_axis() }
    pub fn blue() -> Vec { Self::z_axis() }

    pub fn comp_mult(&self, other: &Vec) -> Vec {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    pub fn comp_div(&self, other: &Vec) -> Vec {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }

    pub fn cross(&self, other: &Vec) -> Vec {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x)
    }

    pub fn dot(&self, other: &Vec) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn is_exactly_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }
    pub fn magnitude(&self) -> f64 {
        f64::sqrt(self.dot(self))
    }

    pub fn normalized(&self) -> Vec {
        let length = self.magnitude();
        Self::new(self.x / length, self.y / length, self.z / length)
    }

    /**
     * Determines whether a vec's magnitude is zero, within a small epsilon.
     */
    pub fn is_nearly_zero(&self) -> bool {
        math::is_nearly_zero(self.dot(&self))
    }

    /**
     * Generates an orthonormal coordinate basis. The first vector must be given,
     * and the other two orthogonal vectors will be generated from it.
     * Taken from page 63 of Pharr & Humphreys' Physically-Based Rendering.
     */
    pub fn coord_system(&self) -> (Vec, Vec) {
        if &self.x.abs() > &self.y.abs() {
            let inv_len = 1.0 / f64::sqrt(self.x * self.x + self.z * self.z);
            let v2 = Self::new(-self.z * inv_len, 0.0, self.x * inv_len);
            let v3 = self.cross(&v2);
            (v2, v3)
        }
        else {
            let inv_len = 1.0 / f64::sqrt(self.y * self.y + self.z * self.z);
            let v2 = Self::new(0.0, self.z * inv_len, -self.y * inv_len);
            let v3 = self.cross(&v2);
            (v2, v3)
        }
    }

    /**
     * Converts a world-space vector to a local coordinate system defined by a vector basis.
     * The resulting coordinates are (x, y, z), where x is the weight of the
     * tangent, y is the weight of the binormal, and z is the weight of the
     * normal.
     */
    pub fn world_to_local(&self, tangent: &Vec, binormal: &Vec, normal: &Vec)
        -> Vec
    {
        Self::new(self.dot(&tangent), self.dot(&binormal), self.dot(&normal))
    }

    /**
     * Converts a local-space vector back to world-space. The local-space vector
     * should be (x, y, z), where x is the weight of the tangent, y is the weight
     * of the binormal, and z is the weight of the normal.
     */
    pub fn local_to_world(&self, tangent: &Vec, binormal: &Vec, normal: &Vec)
        -> Vec
    {
        Self::new(
            tangent.x * self.x + binormal.x * self.y + normal.x * self.z,
            tangent.y * self.x + binormal.y * self.y + normal.y * self.z,
            tangent.z * self.x + binormal.z * self.y + normal.z * self.z
        )
    }

    /**
     * Returns Cos[Theta] of a vector where Theta is the polar angle of the vector
     * in spherical coordinates.
     */
    pub fn cos_theta(&self) -> f64 { self.z }

    /**
     * Returns Abs[Cos[Theta]] of a vector where Theta is the polar angle of the
     * vector in spherical coordinates.
     */
    pub fn abs_cos_theta(&self) -> f64 { self.z.abs() }

    /**
     * Returns Sin[Theta]^2 of a vector where Theta is the polar angle of the
     * vector in spherical coordinates.
     */
    pub fn sin_theta2(&self) -> f64 {
        f64::max(0.0, 1.0 - self.cos_theta() * self.cos_theta())
    }

    /**
     * Returns Sin[Theta] of a vector where Theta is the polar angle of the vector
     * in spherical coordinates.
     */
    pub fn sin_theta(&self) -> f64 {
        f64::sqrt(self.sin_theta2())
    }

    /**
     * Returns Cos[Phi] of a vector where Phi is the azimuthal angle of the vector
     * in spherical coordinates.
     */
    pub fn cos_phi(&self) -> f64 {
        let sin_t = self.sin_theta();
        if sin_t == 0.0 {
            1.0
        }
        else {
            math::clamp(self.x / sin_t, -1.0, 1.0)
        }
    }

    /**
     * Returns Sin[Phi] of a vector where Phi is the azimuthal angle of the vector
     * in spherical coordinates.
     */
    pub fn sin_phi(&self) -> f64 {
        let sin_t = self.sin_theta();
        if sin_t == 0.0 {
            0.0
        }
        else {
            math::clamp(self.y / sin_t, -1.0, 1.0)
        }
    }

    /**
     * Determines if two vectors in the same local coordinate space are in the
     * same hemisphere.
     */
    pub fn is_local_same_hemisphere(&self, v: &Vec) -> bool {
        self.z * v.z >= 0.0
    }

    /**
     * Luminance of an RGB color stored in a vec.
     */
    pub fn luminance(&self) -> f64 {
        0.21 * self.x + 0.71 * self.y + 0.08 * self.z
    }

    /**
     * Same as GLSL reflect.
     * See <https://www.opengl.org/sdk/docs/man4/html/reflect.xhtml>.
     *
     * @param I the incoming vector to reflect
     * @param N the normal at the surface over which to reflect
     * @returns the outgoing reflection vector
     */
    pub fn reflect(&self, n: &Vec) -> Vec {
        let k = 2.0 * n.dot(self);
        Self::new(
            self.x - n.x * k,
            self.y - n.y * k,
            self.z - n.z * k)
    }

    /**
     * Same as GLSL refract.
     * See <https://www.opengl.org/sdk/docs/man4/html/refract.xhtml>.
     *
     * @param I   the incoming vector to refract
     * @param N   the normal at the surface to refract across;
     *            the normal points from the transmitting medium towards the
     *            incident medium
     * @param eta the ratio of the incoming IOR over the transmitting IOR
     * @returns   the outgoing refraction vector
     */
    pub fn refract(&self, n: &Vec, eta: f64) -> Vec {
      let d = n.dot(self);
      let k = 1.0 - eta * eta * (1.0 - d * d);
      if k < 0.0 {
          Self::zero()
      } else {
          let k = eta * d + k.sqrt();
          Self::new(
              self.x * eta - n.x * k,
              self.y * eta - n.y * k,
              self.z * eta - n.z * k)
      }
    }

    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            math::clamp((self.x * 255.99999) as u8, 0u8, 255u8),
            math::clamp((self.y * 255.99999) as u8, 0u8, 255u8),
            math::clamp((self.z * 255.99999) as u8, 0u8, 255u8),
            255u8
        ]
    }
}

impl Display for Vec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl<'a, 'b> Add<&'b Vec> for &'a Vec {
    type Output = Vec;
    fn add(self, _rhs: &'b Vec) -> Vec {
        Vec::new(self.x + _rhs.x, self.y + _rhs.y, self.z + _rhs.z)
    }
}

impl<'a, 'b> Sub<&'b Vec> for &'a Vec {
    type Output = Vec;
    fn sub(self, _rhs: &'b Vec) -> Vec {
        Vec::new(self.x - _rhs.x, self.y - _rhs.y, self.z - _rhs.z)
    }
}

impl<'a> Mul<f64> for &'a Vec {
    type Output = Vec;
    fn mul(self, _rhs: f64) -> Vec {
        Vec::new(self.x * _rhs, self.y * _rhs, self.z * _rhs)
    }
}

impl<'b> Mul<&'b Vec> for f64 {
    type Output = Vec;
    fn mul(self, _rhs: &'b Vec) -> Vec { _rhs * self }
}

impl<'a> Div<f64> for &'a Vec {
    type Output = Vec;
    fn div(self, _rhs: f64) -> Vec {
        Vec::new(self.x / _rhs, self.y / _rhs, self.z / _rhs)
    }
}

impl<'a> Neg for &'a Vec {
    type Output = Vec;
    fn neg(self) -> Vec {
        Vec::new(-self.x, -self.y, -self.z)
    }
}
