use core::vector;

use std::ops::{Div, Mul, Neg};

#[derive(Clone)]
pub struct Quat {
    pub real: f32,
    pub imaginary: vector::Vec,
}

impl Quat {
    pub fn identity() -> Quat {
        Quat {real: 1.0, imaginary: vector::Vec::zero()}
    }
    pub fn length_squared(&self) -> f32 {
        self.real * self.real + self.imaginary.dot(&self.imaginary)
    }
}

impl Neg for Quat {
    type Output = Quat;
    fn neg(mut self) -> Quat {
        let lsq = self.length_squared();
        self.real = self.real / lsq;
        self.imaginary = &(-&self.imaginary) / lsq;
        self
    }
}

impl Mul for Quat {
    type Output = Quat;
    fn mul(mut self, _rhs: Quat) -> Quat {
        let r: f32;
        let i: vector::Vec;
        {
            let r1 = self.real;
            let r2 = _rhs.real;

            let i1 = &self.imaginary;
            let i2 = &_rhs.imaginary;

            r = r1 * r2 - i1.dot(i2);
            i = vector::Vec::new(
                r1 * i2.x + r2 * i1.x + (i1.y * i2.z - i1.z * i2.y),
    	        r1 * i2.y + r2 * i1.y + (i1.z * i2.x - i1.x * i2.z),
    	        r1 * i2.z + r2 * i1.z + (i1.x * i2.y - i1.y * i2.x));
        }

        self.real = r;
        self.imaginary = i;
        self
    }
}

impl Mul<f32> for Quat {
    type Output = Quat;
    fn mul(mut self, _rhs: f32) -> Quat {
        self.real = self.real * _rhs;
        self.imaginary = &self.imaginary * _rhs;
        self
    }
}

impl Div<f32> for Quat {
    type Output = Quat;
    fn div(mut self, _rhs: f32) -> Quat {
        self.real = self.real / _rhs;
        self.imaginary = &self.imaginary / _rhs;
        self
    }
}
