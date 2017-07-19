use core::math;
use num::traits::{Float, Signed};
use std::fmt;
use std::fmt::Display;
use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Copy, Clone)]
pub struct Vec3<T> where T: Signed + Copy {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T> where T: Signed + Copy {
    pub fn new(x: T, y: T, z: T) -> Vec3<T> {
        Vec3 {x: x, y: y, z: z}
    }

    pub fn zero() -> Vec3<T> {
        Self::new(T::zero(), T::zero(), T::zero())
    }

    pub fn one() -> Vec3<T> {
        Self::new(T::one(), T::one(), T::one())
    }

    pub fn comp_mult(&self, other: &Vec3<T>) -> Vec3<T> {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }

    pub fn comp_div(&self, other: &Vec3<T>) -> Vec3<T> {
        Self::new(self.x / other.x, self.y / other.y, self.z / other.z)
    }

    pub fn cross(&self, other: &Vec3<T>) -> Vec3<T> {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x)
    }

    pub fn dot(&self, other: &Vec3<T>) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn is_exactly_zero(&self) -> bool {
        self.x == T::zero() && self.y == T::zero() && self.z == T::zero()
    }
}

impl<T> Display for Vec3<T> where T: Signed + Copy + Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl<T> Add for Vec3<T> where T: Signed + Copy {
    type Output = Vec3<T>;
    fn add(mut self, _rhs: Vec3<T>) -> Vec3<T> {
        self.x = self.x + _rhs.x;
        self.y = self.y + _rhs.y;
        self.z = self.z + _rhs.z;
        self
    }
}

impl<T> Sub for Vec3<T> where T: Signed + Copy {
    type Output = Vec3<T>;
    fn sub(mut self, _rhs: Vec3<T>) -> Vec3<T> {
        self.x = self.x - _rhs.x;
        self.y = self.y - _rhs.y;
        self.z = self.z - _rhs.z;
        self
    }
}

impl<T> Mul<T> for Vec3<T> where T: Signed + Copy {
    type Output = Vec3<T>;
    fn mul(mut self, _rhs: T) -> Vec3<T> {
        self.x = self.x - _rhs;
        self.y = self.y - _rhs;
        self.z = self.z - _rhs;
        self
    }
}

impl Mul<Vec3<f64>> for f64 {
    type Output = Vec3<f64>;
    fn mul(self, _rhs: Vec3<f64>) -> Vec3<f64> { _rhs * self }
}

impl Mul<Vec3<i32>> for i32 {
    type Output = Vec3<i32>;
    fn mul(self, _rhs: Vec3<i32>) -> Vec3<i32> { _rhs * self }
}

impl<T> Div<T> for Vec3<T> where T: Signed + Copy {
    type Output = Vec3<T>;
    fn div(mut self, _rhs: T) -> Vec3<T> {
        self.x = self.x / _rhs;
        self.y = self.y / _rhs;
        self.z = self.z / _rhs;
        self
    }
}

impl<T> Neg for Vec3<T> where T: Signed + Copy {
    type Output = Vec3<T>;
    fn neg(mut self) -> Vec3<T> {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }
}

impl<T> Vec3<T> where T: Float + Signed + Copy {
    pub fn magnitude(&self) -> T {
        T::sqrt(self.dot(self))
    }

    pub fn normalized(&self) -> Vec3<T> {
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
    pub fn coord_system(&self) -> (Vec3<T>, Vec3<T>) {
        if Signed::abs(&self.x) > Signed::abs(&self.y) {
            let inv_len = T::one() / T::sqrt(self.x * self.x + self.z * self.z);
            let v2 = Self::new(-self.z * inv_len, T::zero(), self.x * inv_len);
            let v3 = self.cross(&v2);
            (v2, v3)
        }
        else {
            let inv_len = T::one() / T::sqrt(self.y * self.y + self.z * self.z);
            let v2 = Self::new(T::zero(), self.z * inv_len, -self.y * inv_len);
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
    pub fn world_to_local(&self, tangent: &Vec3<T>, binormal: &Vec3<T>, normal: &Vec3<T>)
        -> Vec3<T>
    {
        Self::new(self.dot(&tangent), self.dot(&binormal), self.dot(&normal))
    }


    /**
     * Converts a local-space vector back to world-space. The local-space vector
     * should be (x, y, z), where x is the weight of the tangent, y is the weight
     * of the binormal, and z is the weight of the normal.
     */
    pub fn local_to_world(&self, tangent: &Vec3<T>, binormal: &Vec3<T>, normal: &Vec3<T>)
        -> Vec3<T>
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
    pub fn cos_theta(&self) -> T { self.z }

    /**
     * Returns Abs[Cos[Theta]] of a vector where Theta is the polar angle of the
     * vector in spherical coordinates.
     */
    pub fn abs_cos_theta(&self) -> T { Signed::abs(&self.z) }

    /**
     * Returns Sin[Theta]^2 of a vector where Theta is the polar angle of the
     * vector in spherical coordinates.
     */
    pub fn sin_theta2(&self) -> T {
        T::max(T::zero(), T::one() - self.cos_theta() * self.cos_theta())
    }

    /**
     * Returns Sin[Theta] of a vector where Theta is the polar angle of the vector
     * in spherical coordinates.
     */
    pub fn sin_theta(&self) -> T {
        T::sqrt(self.sin_theta2())
    }

    /**
     * Returns Cos[Phi] of a vector where Phi is the azimuthal angle of the vector
     * in spherical coordinates.
     */
    pub fn cos_phi(&self) -> T {
        let sin_t = self.sin_theta();
        if sin_t == T::zero() {
            T::one()
        }
        else {
            math::clamp(self.x / sin_t, -T::one(), T::one())
        }
    }

    /**
     * Returns Sin[Phi] of a vector where Phi is the azimuthal angle of the vector
     * in spherical coordinates.
     */
    pub fn sin_phi(&self) -> T {
        let sin_t = self.sin_theta();
        if sin_t == T::zero() {
            T::zero()
        }
        else {
            math::clamp(self.y / sin_t, -T::one(), T::one())
        }
    }

    /**
     * Determines if two vectors in the same local coordinate space are in the
     * same hemisphere.
     */
    pub fn is_local_same_hemisphere(&self, v: &Vec3<T>) -> bool {
        self.z * v.z >= T::zero()
    }
}

impl Vec3<f64> {
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
    pub fn reflect(&self, n: &Vec3<f64>) -> Vec3<f64> {
        *self - (*n * (2.0 * n.dot(self)))
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
    pub fn refract(&self, n: &Vec3<f64>, eta: f64) -> Vec3<f64> {
      let d = n.dot(self);
      let k = 1.0 - eta * eta * (1.0 - d * d);
      if k < 0.0 {
          Self::zero()
      } else {
          (*self * eta) - *n * ((eta * d + k.sqrt()))
      }
    }
}
