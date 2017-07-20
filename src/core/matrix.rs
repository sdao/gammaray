use core::math;
use core::quat;
use core::ray;
use core::vector;

use std;
use std::fmt;
use std::fmt::Display;
use std::ops::{Mul, Index, IndexMut};

/** A 4x4 matrix in row-major order. */
#[derive(Clone)]
pub struct Mat {
    storage: [[f64; 4]; 4],
}

impl Mat {
    pub fn new(data: [[f64; 4]; 4]) -> Mat {
        Self {storage: data}
    }

    pub fn zero() -> Mat {
        Self::new([[0.0; 4]; 4])
    }

    pub fn identity() -> Mat {
        Self::diagonal(1.0)
    }

    pub fn diagonal(k: f64) -> Mat {
        let mut output = Self::zero();
        output[0][0] = k;
        output[0][1] = 0.0;
        output[0][2] = 0.0;
        output[0][2] = 0.0;
        output[1][0] = 0.0;
        output[1][1] = k;
        output[1][2] = 0.0;
        output[1][3] = 0.0;
        output[2][0] = 0.0;
        output[2][1] = 0.0;
        output[2][2] = k;
        output[2][3] = 0.0;
        output[3][0] = 0.0;
        output[3][1] = 0.0;
        output[3][2] = 0.0;
        output[3][3] = k;

        output
    }

    pub fn scale(k: f64) -> Mat {
        let mut output = Self::zero();
        output[0][0] = k;
        output[0][1] = 0.0;
        output[0][2] = 0.0;
        output[0][2] = 0.0;
        output[1][0] = 0.0;
        output[1][1] = k;
        output[1][2] = 0.0;
        output[1][3] = 0.0;
        output[2][0] = 0.0;
        output[2][1] = 0.0;
        output[2][2] = k;
        output[2][3] = 0.0;
        output[3][0] = 0.0;
        output[3][1] = 0.0;
        output[3][2] = 0.0;
        output[3][3] = 1.0;

        output
    }

    pub fn translation(translate: &vector::Vec) -> Mat {
        let mut output = Self::zero();
        output[0][0] = 1.0;
        output[0][1] = 0.0;
        output[0][2] = 0.0;
        output[0][2] = 0.0;
        output[1][0] = 0.0;
        output[1][1] = 1.0;
        output[1][2] = 0.0;
        output[1][3] = 0.0;
        output[2][0] = 0.0;
        output[2][1] = 0.0;
        output[2][2] = 1.0;
        output[2][3] = 0.0;
        output[3][0] = translate.x;
        output[3][1] = translate.y;
        output[3][2] = translate.z;
        output[3][3] = 1.0;

        output
    }

    pub fn rotation(rotate: &quat::Quat) -> Mat {
        let r = &rotate.real;
        let i = &rotate.imaginary;

        let mut output = Self::zero();
        output[0][0] = 1.0 - 2.0 * (i.y * i.y + i.z * i.z);
        output[0][1] =       2.0 * (i.x * i.y + i.z *   r);
        output[0][2] =       2.0 * (i.z * i.x - i.y *   r);
        output[0][3] = 0.0;

        output[1][0] =       2.0 * (i.x * i.y - i.z *   r);
        output[1][1] = 1.0 - 2.0 * (i.z * i.z + i.x * i.x);
        output[1][2] =       2.0 * (i.y * i.z + i.x *   r);
        output[1][3] = 0.0;

        output[2][0] =       2.0 * (i.z * i.x + i.y *   r);
        output[2][1] =       2.0 * (i.y * i.z - i.x *   r);
        output[2][2] = 1.0 - 2.0 * (i.y * i.y + i.x * i.x);
        output[2][3] = 0.0;

        output[3][0] = 0.0;
        output[3][1] = 0.0;
        output[3][2] = 0.0;
        output[3][3] = 1.0;

        output
    }

    pub fn transposed(&self) -> Mat {
        let mut output = Mat::zero();
        for row in 0..4 {
            for col in 0..4 {
                output[row][col] = output[col][row];
            }
        }
        output
    }

    fn get_determinant3(&self, r1: usize, r2: usize, r3: usize, c1: usize, c2: usize, c3: usize)
        -> f64
    {
        (  self[r1][c1] * self[r2][c2] * self[r3][c3]
	     + self[r1][c2] * self[r2][c3] * self[r3][c1]
	     + self[r1][c3] * self[r2][c1] * self[r3][c2]
	     - self[r1][c1] * self[r2][c3] * self[r3][c2]
	     - self[r1][c2] * self[r2][c1] * self[r3][c3]
	     - self[r1][c3] * self[r2][c2] * self[r3][c1])
    }

    pub fn get_determinant(&self) -> f64 {
        (- self[0][3] * self.get_determinant3(1, 2, 3, 0, 1, 2)
         + self[1][3] * self.get_determinant3(0, 2, 3, 0, 1, 2)
         - self[2][3] * self.get_determinant3(0, 1, 3, 0, 1, 2)
         + self[3][3] * self.get_determinant3(0, 1, 2, 0, 1, 2))
    }

    pub fn transform(&self, v: &vector::Vec) -> vector::Vec {
        let x = v.x * self[0][0] + v.y * self[1][0] + v.z * self[2][0] + self[3][0];
        let y = v.x * self[0][1] + v.y * self[1][1] + v.z * self[2][1] + self[3][1];
        let z = v.x * self[0][2] + v.y * self[1][2] + v.z * self[2][2] + self[3][2];
        let w = v.x * self[0][3] + v.y * self[1][3] + v.z * self[2][3] + self[3][3];
        vector::Vec::new(x / w, y / w, z / w)
    }

    pub fn transform_dir(&self, v: &vector::Vec) -> vector::Vec {
        vector::Vec::new(
            v.x * self[0][0] + v.y * self[1][0] + v.z * self[2][0],
            v.x * self[0][1] + v.y * self[1][1] + v.z * self[2][1],
            v.x * self[0][2] + v.y * self[1][2] + v.z * self[2][2])
    }

    pub fn transform_ray(&self, r: &ray::Ray) -> ray::Ray {
        ray::Ray {
            origin: self.transform(&r.origin),
            direction: self.transform_dir(&r.direction)
        }
    }

    pub fn inverted(&self) -> Mat {
        let mut x00: f64;
        let mut x01: f64;
        let x02: f64;
        let x03: f64;
        let mut x10: f64;
        let mut x11: f64;
        let x12: f64;
        let x13: f64;
        let mut x20: f64;
        let mut x21: f64;
        let x22: f64;
        let x23: f64;
        let mut x30: f64;
        let mut x31: f64;
        let x32: f64;
        let x33: f64;
        let mut y01: f64;
        let mut y02: f64;
        let mut y03: f64;
        let mut y12: f64;
        let mut y13: f64;
        let mut y23: f64;
        let z00: f64;
        let z01: f64;
        let z02: f64;
        let z03: f64;
        let z10: f64;
        let z11: f64;
        let z12: f64;
        let z13: f64;
        let z20: f64;
        let z21: f64;
        let z22: f64;
        let z23: f64;
        let z30: f64;
        let z31: f64;
        let z32: f64;
        let z33: f64;

        // Pickle 1st two columns of matrix into registers.
        x00 = self[0][0];
        x01 = self[0][1];
        x10 = self[1][0];
        x11 = self[1][1];
        x20 = self[2][0];
        x21 = self[2][1];
        x30 = self[3][0];
        x31 = self[3][1];

        // Compute all six 2x2 determinants of 1st two columns.
        y01 = (x00 * x11) - (x10 * x01);
        y02 = (x00 * x21) - (x20 * x01);
        y03 = (x00 * x31) - (x30 * x01);
        y12 = (x10 * x21) - (x20 * x11);
        y13 = (x10 * x31) - (x30 * x11);
        y23 = (x20 * x31) - (x30 * x21);

        // Pickle 2nd two columns of matrix into registers.
        x02 = self[0][2];
        x03 = self[0][3];
        x12 = self[1][2];
        x13 = self[1][3];
        x22 = self[2][2];
        x23 = self[2][3];
        x32 = self[3][2];
        x33 = self[3][3];

        // Compute all 3x3 cofactors for 2nd two columns.
        z33 = (x02 * y12) - (x12 * y02) + (x22 * y01);
        z23 = (x12 * y03) - (x32 * y01) - (x02 * y13);
        z13 = (x02 * y23) - (x22 * y03) + (x32 * y02);
        z03 = (x22 * y13) - (x32 * y12) - (x12 * y23);
        z32 = (x13 * y02) - (x23 * y01) - (x03 * y12);
        z22 = (x03 * y13) - (x13 * y03) + (x33 * y01);
        z12 = (x23 * y03) - (x33 * y02) - (x03 * y23);
        z02 = (x13 * y23) - (x23 * y13) + (x33 * y12);

        // Compute all six 2x2 determinants of 2nd two columns.
        y01 = (x02 * x13) - (x12 * x03);
        y02 = (x02 * x23) - (x22 * x03);
        y03 = (x02 * x33) - (x32 * x03);
        y12 = (x12 * x23) - (x22 * x13);
        y13 = (x12 * x33) - (x32 * x13);
        y23 = (x22 * x33) - (x32 * x23);

        // Pickle 1st two columns of matrix into registers.
        x00 = self[0][0];
        x01 = self[0][1];
        x10 = self[1][0];
        x11 = self[1][1];
        x20 = self[2][0];
        x21 = self[2][1];
        x30 = self[3][0];
        x31 = self[3][1];

        // Compute all 3x3 cofactors for 1st two columns.
        z30 = (x11 * y02) - (x21 * y01) - (x01 * y12);
        z20 = (x01 * y13) - (x11 * y03) + (x31 * y01);
        z10 = (x21 * y03) - (x31 * y02) - (x01 * y23);
        z00 = (x11 * y23) - (x21 * y13) + (x31 * y12);
        z31 = (x00 * y12) - (x10 * y02) + (x20 * y01);
        z21 = (x10 * y03) - (x30 * y01) - (x00 * y13);
        z11 = (x00 * y23) - (x20 * y03) + (x30 * y02);
        z01 = (x20 * y13) - (x30 * y12) - (x10 * y23);

        // Compute 4x4 determinant & its reciprocal.
        let det = (x30 * z30) + (x20 * z20) + (x10 * z10) + (x00 * z00);

        if math::is_positive(det) {
            let rcp = 1.0 / det;
            // Multiply all 3x3 cofactors by reciprocal & transpose.
            let mut output = Mat::zero();
            output[0][0] = z00 * rcp;
            output[0][1] = z10 * rcp;
            output[1][0] = z01 * rcp;
            output[0][2] = z20 * rcp;
            output[2][0] = z02 * rcp;
            output[0][3] = z30 * rcp;
            output[3][0] = z03 * rcp;
            output[1][1] = z11 * rcp;
            output[1][2] = z21 * rcp;
            output[2][1] = z12 * rcp;
            output[1][3] = z31 * rcp;
            output[3][1] = z13 * rcp;
            output[2][2] = z22 * rcp;
            output[2][3] = z32 * rcp;
            output[3][2] = z23 * rcp;
            output[3][3] = z33 * rcp;

            output
        }
        else {
    	    Self::scale(std::f64::MAX)
        }
    }
}

impl Display for Mat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = String::new();
        output.push_str("(");
        for row in 0..4 {
            output.push_str("(");
            for col in 0..4 {
                output.push_str(&self[row][col].to_string());
                if col != 3 {
                    output.push_str(", ");
                }
            }
            output.push_str(")");
            if row != 3 {
                output.push_str(", ");
            }
        }
        output.push_str(")");
        write!(f, "{}", output)
    }
}

impl<'a, 'b> Mul<&'b Mat> for &'a Mat {
    type Output = Mat;
    fn mul(self, _rhs: &'b Mat) -> Mat {
        let mut output = Mat::zero();
        output[0][0] = self[0][0] * _rhs[0][0] +
                       self[0][1] * _rhs[1][0] +
                       self[0][2] * _rhs[2][0] +
                       self[0][3] * _rhs[3][0];

        output[0][1] = self[0][0] * _rhs[0][1] +
                       self[0][1] * _rhs[1][1] +
                       self[0][2] * _rhs[2][1] +
                       self[0][3] * _rhs[3][1];

        output[0][2] = self[0][0] * _rhs[0][2] +
                       self[0][1] * _rhs[1][2] +
                       self[0][2] * _rhs[2][2] +
                       self[0][3] * _rhs[3][2];

        output[0][3] = self[0][0] * _rhs[0][3] +
                       self[0][1] * _rhs[1][3] +
                       self[0][2] * _rhs[2][3] +
                       self[0][3] * _rhs[3][3];

        output[1][0] = self[1][0] * _rhs[0][0] +
                       self[1][1] * _rhs[1][0] +
                       self[1][2] * _rhs[2][0] +
                       self[1][3] * _rhs[3][0];

        output[1][1] = self[1][0] * _rhs[0][1] +
                       self[1][1] * _rhs[1][1] +
                       self[1][2] * _rhs[2][1] +
                       self[1][3] * _rhs[3][1];

        output[1][2] = self[1][0] * _rhs[0][2] +
                       self[1][1] * _rhs[1][2] +
                       self[1][2] * _rhs[2][2] +
                       self[1][3] * _rhs[3][2];

        output[1][3] = self[1][0] * _rhs[0][3] +
                       self[1][1] * _rhs[1][3] +
                       self[1][2] * _rhs[2][3] +
                       self[1][3] * _rhs[3][3];

        output[2][0] = self[2][0] * _rhs[0][0] +
                       self[2][1] * _rhs[1][0] +
                       self[2][2] * _rhs[2][0] +
                       self[2][3] * _rhs[3][0];

        output[2][1] = self[2][0] * _rhs[0][1] +
                       self[2][1] * _rhs[1][1] +
                       self[2][2] * _rhs[2][1] +
                       self[2][3] * _rhs[3][1];

        output[2][2] = self[2][0] * _rhs[0][2] +
                       self[2][1] * _rhs[1][2] +
                       self[2][2] * _rhs[2][2] +
                       self[2][3] * _rhs[3][2];

        output[2][3] = self[2][0] * _rhs[0][3] +
                       self[2][1] * _rhs[1][3] +
                       self[2][2] * _rhs[2][3] +
                       self[2][3] * _rhs[3][3];

        output[3][0] = self[3][0] * _rhs[0][0] +
                       self[3][1] * _rhs[1][0] +
                       self[3][2] * _rhs[2][0] +
                       self[3][3] * _rhs[3][0];

        output[3][1] = self[3][0] * _rhs[0][1] +
                       self[3][1] * _rhs[1][1] +
                       self[3][2] * _rhs[2][1] +
                       self[3][3] * _rhs[3][1];

        output[3][2] = self[3][0] * _rhs[0][2] +
                       self[3][1] * _rhs[1][2] +
                       self[3][2] * _rhs[2][2] +
                       self[3][3] * _rhs[3][2];

        output[3][3] = self[3][0] * _rhs[0][3] +
                       self[3][1] * _rhs[1][3] +
                       self[3][2] * _rhs[2][3] +
                       self[3][3] * _rhs[3][3];

        output
    }
}

impl Index<usize> for Mat {
    type Output = [f64; 4];

    fn index(&self, index: usize) -> &[f64; 4] {
        &self.storage[index]
    }
}

impl IndexMut<usize> for Mat {
    fn index_mut(&mut self, index: usize) -> &mut [f64; 4] {
        &mut self.storage[index]
    }
}
