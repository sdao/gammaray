use core::bbox;
use core::matrix;
use core::ray;
use core::vector;

pub struct Xform {
    mat: matrix::Mat,
    inv_mat: matrix::Mat,
}

static IDENTITY: Xform = Xform {
    mat: matrix::Mat {
        storage: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]]
    },
    inv_mat: matrix::Mat {
        storage: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]]
    },
};

impl Xform {
    pub fn new(mat: matrix::Mat) -> Xform {
        let inv_mat = mat.inverted();
        Xform {
            mat: mat,
            inv_mat: inv_mat,
        }
    }

    pub fn identity_ref() -> &'static Xform {
        &IDENTITY
    }

    fn _transform(mat: &matrix::Mat, v: &vector::Vec) -> vector::Vec {
        let x = v.x * mat[0][0] + v.y * mat[1][0] + v.z * mat[2][0] + mat[3][0];
        let y = v.x * mat[0][1] + v.y * mat[1][1] + v.z * mat[2][1] + mat[3][1];
        let z = v.x * mat[0][2] + v.y * mat[1][2] + v.z * mat[2][2] + mat[3][2];
        let w = v.x * mat[0][3] + v.y * mat[1][3] + v.z * mat[2][3] + mat[3][3];
        vector::Vec::new(x / w, y / w, z / w)
    }

    fn _transform_dir(mat: &matrix::Mat, v: &vector::Vec) -> vector::Vec {
        vector::Vec::new(
                v.x * mat[0][0] + v.y * mat[1][0] + v.z * mat[2][0],
                v.x * mat[0][1] + v.y * mat[1][1] + v.z * mat[2][1],
                v.x * mat[0][2] + v.y * mat[1][2] + v.z * mat[2][2])
    }

    fn _transform_normal(inv_mat: &matrix::Mat, v: &vector::Vec) -> vector::Vec {
        vector::Vec::new(
                v.x * inv_mat[0][0] + v.y * inv_mat[0][1] + v.z * inv_mat[0][2],
                v.x * inv_mat[1][0] + v.y * inv_mat[1][1] + v.z * inv_mat[1][2],
                v.x * inv_mat[2][0] + v.y * inv_mat[2][1] + v.z * inv_mat[2][2])
    }

    fn _transform_ray(mat: &matrix::Mat, r: &ray::Ray) -> ray::Ray {
        ray::Ray {
            origin: Self::_transform(mat, &r.origin),
            direction: Self::_transform_dir(mat, &r.direction)
        }
    }

    fn _transform_bbox(mat: &matrix::Mat, b: &bbox::BBox) -> bbox::BBox {
        bbox::BBox::empty()
                .union_with(&Self::_transform(mat, &vector::Vec::new(b.min.x, b.min.y, b.min.z)))
                .union_with(&Self::_transform(mat, &vector::Vec::new(b.max.x, b.min.y, b.min.z)))
                .union_with(&Self::_transform(mat, &vector::Vec::new(b.min.x, b.max.y, b.min.z)))
                .union_with(&Self::_transform(mat, &vector::Vec::new(b.min.x, b.min.y, b.max.z)))
                .union_with(&Self::_transform(mat, &vector::Vec::new(b.min.x, b.max.y, b.max.z)))
                .union_with(&Self::_transform(mat, &vector::Vec::new(b.max.x, b.min.y, b.max.z)))
                .union_with(&Self::_transform(mat, &vector::Vec::new(b.max.x, b.max.y, b.min.z)))
                .union_with(&Self::_transform(mat, &vector::Vec::new(b.max.x, b.max.y, b.max.z)))
    }

    pub fn transform(&self, v: &vector::Vec) -> vector::Vec {
        Self::_transform(&self.mat, v)
    }

    pub fn untransform(&self, v: &vector::Vec) -> vector::Vec {
        Self::_transform(&self.inv_mat, v)
    }

    pub fn transform_dir(&self, v: &vector::Vec) -> vector::Vec {
        Self::_transform_dir(&self.mat, v)
    }

    pub fn untransform_dir(&self, v: &vector::Vec) -> vector::Vec {
        Self::_transform_dir(&self.inv_mat, v)
    }
    
    pub fn transform_normal(&self, v: &vector::Vec) -> vector::Vec {
        // This is right. _transform_normal takes the inverse mat because normals are transformed
        // by the transposed inverted matrix.
        Self::_transform_normal(&self.inv_mat, v)
    }

    pub fn untransform_normal(&self, v: &vector::Vec) -> vector::Vec {
        Self::_transform_normal(&self.mat, v)
    }

    pub fn transform_ray(&self, r: &ray::Ray) -> ray::Ray {
        Self::_transform_ray(&self.mat, r)
    }

    pub fn untransform_ray(&self, r: &ray::Ray) -> ray::Ray {
        Self::_transform_ray(&self.inv_mat, r)
    }

    pub fn transform_bbox(&self, b: &bbox::BBox) -> bbox::BBox {
        Self::_transform_bbox(&self.mat, b)
    }

    pub fn untransform_bbox(&self, b: &bbox::BBox) -> bbox::BBox {
        Self::_transform_bbox(&self.inv_mat, b)
    }
}