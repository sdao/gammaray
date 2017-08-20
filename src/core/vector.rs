use core::math;

use std::fmt;
use std::fmt::Display;
use std::ops::{Add, Sub, Mul, Div, Neg, Index, IndexMut};

#[derive(Clone, Copy)]
pub struct Vec {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec {
    pub fn new(x: f32, y: f32, z: f32) -> Vec {
        Vec {x: x, y: y, z: z}
    }

    /// Computes a vector from spherical coordinates, with radius 1, inclination theta, and
    /// azimuth phi.
    pub fn from_spherical(cos_theta: f32, phi: f32) -> Vec {
        let sin_theta = f32::sqrt(1.0 - cos_theta * cos_theta);
        Self::new(sin_theta * f32::cos(phi), sin_theta * f32::sin(phi), cos_theta)
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

    pub fn dot(&self, other: &Vec) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn is_exactly_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }
    pub fn magnitude(&self) -> f32 {
        f32::sqrt(self.dot(self))
    }

    pub fn normalized(&self) -> Vec {
        let length = self.magnitude();
        Self::new(self.x / length, self.y / length, self.z / length)
    }

    pub fn sqrt(&self) -> Vec {
        Self::new(f32::sqrt(self.x), f32::sqrt(self.y), f32::sqrt(self.z))
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
        if f32::abs(self.x) > f32::abs(self.y) {
            let inv_len = 1.0 / f32::sqrt(self.x * self.x + self.z * self.z);
            let v2 = Self::new(-self.z * inv_len, 0.0, self.x * inv_len);
            let v3 = self.cross(&v2);
            (v2, v3)
        }
        else {
            let inv_len = 1.0 / f32::sqrt(self.y * self.y + self.z * self.z);
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
    pub fn cos_theta(&self) -> f32 { self.z }

    pub fn cos2_theta(&self) -> f32 { self.z * self.z }

    /**
     * Returns Abs[Cos[Theta]] of a vector where Theta is the polar angle of the
     * vector in spherical coordinates.
     */
    pub fn abs_cos_theta(&self) -> f32 { f32::abs(self.z) }

    /**
     * Returns Sin[Theta]^2 of a vector where Theta is the polar angle of the
     * vector in spherical coordinates.
     */
    pub fn sin2_theta(&self) -> f32 {
        f32::max(0.0, 1.0 - self.cos2_theta())
    }

    /**
     * Returns Sin[Theta] of a vector where Theta is the polar angle of the vector
     * in spherical coordinates.
     */
    pub fn sin_theta(&self) -> f32 {
        f32::sqrt(self.sin2_theta())
    }

    pub fn tan_theta(&self) -> f32 {
        self.sin_theta() / self.cos_theta()
    }

    pub fn tan2_theta(&self) -> f32 {
        self.sin2_theta() / self.cos2_theta()
    }

    /**
     * Returns Cos[Phi] of a vector where Phi is the azimuthal angle of the vector
     * in spherical coordinates.
     */
    pub fn cos_phi(&self) -> f32 {
        let sin_t = self.sin_theta();
        if sin_t == 0.0 {
            1.0
        }
        else {
            math::clamp(self.x / sin_t, -1.0, 1.0)
        }
    }

    pub fn cos2_phi(&self) -> f32 {
        self.cos_phi() * self.cos_phi()
    }

    /**
     * Returns Sin[Phi] of a vector where Phi is the azimuthal angle of the vector
     * in spherical coordinates.
     */
    pub fn sin_phi(&self) -> f32 {
        let sin_t = self.sin_theta();
        if sin_t == 0.0 {
            0.0
        }
        else {
            math::clamp(self.y / sin_t, -1.0, 1.0)
        }
    }

    pub fn sin2_phi(self) -> f32 {
        self.sin_phi() * self.sin_phi()
    }

    /**
     * Determines if two vectors in the same local coordinate space are in the
     * same hemisphere.
     */
    pub fn is_local_same_hemisphere(&self, v: &Vec) -> bool {
        self.z * v.z > 0.0
    }

    /**
     * Luminance of an RGB color stored in a vec.
     */
    pub fn luminance(&self) -> f32 {
        0.21 * self.x + 0.71 * self.y + 0.08 * self.z
    }

    /**
     * Interprets this vector as a color; returns a version normalized by luminance to isolate hue
     * and saturation.
     */
    pub fn tint(&self) -> Vec {
        let lume = self.luminance();
        if lume > 0.0 {
            self / lume
        }
        else {
            Self::one()
        }
    }

    /**
     * Reflects a vector over a surface normal. The original and reflected vectors both
     * point away from the surface. (This produces the opposite result of GLSL reflect.)
     */
    pub fn reflect(&self, n: &Vec) -> Vec {
        let k = 2.0 * n.dot(self);
        Self::new(
            n.x * k - self.x,
            n.y * k - self.y,
            n.z * k - self.z)
    }

    /**
     * Refracts a vector over a surface with the given angle and eta (IOR). The original and
     * refracted vectors both point away from the surface. (This produces a different result
     * from GLSL refract.)
     */
    pub fn refract(&self, n: &Vec, eta: f32) -> Vec {
        let cos_theta_in = n.dot(self);
        let sin2_theta_in = f32::max(0.0, 1.0 - cos_theta_in * cos_theta_in);
        let sin2_theta_trans = eta * eta * sin2_theta_in;
        if sin2_theta_trans >= 1.0 {
            Self::zero()
        }
        else {
            let cos_theta_trans = f32::sqrt(1.0 - sin2_theta_trans);
            &(-eta * self) + &((eta * cos_theta_in - cos_theta_trans) * n)
        }
    }

    pub fn to_rgba8(&self) -> [u8; 4] {
        [
            (math::clamp_unit(self.x) * 255.99999) as u8,
            (math::clamp_unit(self.y) * 255.99999) as u8,
            (math::clamp_unit(self.z) * 255.99999) as u8,
            255u8
        ]
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn lerp(&self, other: &Vec, a: f32) -> Vec {
        Self::new(
            math::lerp(self.x, other.x, a),
            math::lerp(self.y, other.y, a),
            math::lerp(self.z, other.z, a))
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

impl<'a> Mul<f32> for &'a Vec {
    type Output = Vec;
    fn mul(self, _rhs: f32) -> Vec {
        Vec::new(self.x * _rhs, self.y * _rhs, self.z * _rhs)
    }
}

impl<'b> Mul<&'b Vec> for f32 {
    type Output = Vec;
    fn mul(self, _rhs: &'b Vec) -> Vec { _rhs * self }
}

impl<'a> Div<f32> for &'a Vec {
    type Output = Vec;
    fn div(self, _rhs: f32) -> Vec {
        Vec::new(self.x / _rhs, self.y / _rhs, self.z / _rhs)
    }
}

impl<'a> Neg for &'a Vec {
    type Output = Vec;
    fn neg(self) -> Vec {
        Vec::new(-self.x, -self.y, -self.z)
    }
}

impl Index<usize> for Vec {
    type Output = f32;

    fn index(&self, index: usize) -> &f32 {
        if index == 0 {
            &self.x
        }
        else if index == 1 {
            &self.y
        }
        else if index == 2 {
            &self.z
        }
        else {
            panic!("Vec index out of bounds");
        }
    }
}

impl IndexMut<usize> for Vec {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        if index == 0 {
            &mut self.x
        }
        else if index == 1 {
            &mut self.y
        }
        else if index == 2 {
            &mut self.z
        }
        else {
            panic!("Vec index out of bounds");
        }
    }
}
