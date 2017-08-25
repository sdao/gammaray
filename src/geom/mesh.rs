use geom::prim;

use core;
use material;

use std;
use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use wavefront_obj;

struct Tri {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub an: usize,
    pub bn: usize,
    pub cn: usize,
}

impl Tri {
    pub fn new(a: usize, b: usize, c: usize, an: usize, bn: usize, cn: usize) -> Tri {
        Tri {a: a, b: b, c: c, an: an, bn: bn, cn: cn}
    }
}

pub struct Mesh {
    mat: material::Material,
    vertices: std::vec::Vec<core::Vec>,
    normals: std::vec::Vec<core::Vec>,
    tris: std::vec::Vec<Tri>,
}

impl Mesh {
    pub fn from_obj<P: AsRef<Path>>(material: material::Material, mat: core::Mat, path: P)
        -> Result<Mesh, String>
    {
        let mut file: File;
        match File::open(path) {
            Ok(f) => {
                file = f;
            },
            Err(reason) => {
                return Err(format!("Couldn't open OBJ file: {}", reason));
            }
        }

        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(reason) => {
                return Err(format!("Couldn't read OBJ file: {}", reason));
            }
        }

        let obj_set: wavefront_obj::obj::ObjSet;
        match wavefront_obj::obj::parse(contents) {
            Ok(os) => {
                obj_set = os;
            },
            Err(reason) => {
                return Err(format!("OBJ parse error at line {}: {}",
                        reason.line_number, reason.message));
            }
        }

        let mut vertices = std::vec::Vec::<core::Vec>::new();
        let mut normals = std::vec::Vec::<core::Vec>::new();
        let mut tris = std::vec::Vec::<Tri>::new();
        let xform = core::Xform::new(mat);
        for obj in obj_set.objects {
            // Copy all vertices.
            let offset = vertices.len();
            for v in obj.vertices {
                vertices.push(xform.transform(&core::Vec::new(v.x as f32, v.y as f32, v.z as f32)));
            }

            // Copy all normals.
            let noffset = normals.len();
            for n in obj.normals {
                normals.push(xform.transform_normal(
                        &core::Vec::new(n.x as f32, n.y as f32, n.z as f32)));
            }

            // Copy all triangles.
            for g in obj.geometry {
                for s in g.shapes {
                    match s.primitive {
                        wavefront_obj::obj::Primitive::Triangle(
                            (av, _, Some(an)), (bv, _, Some(bn)), (cv, _, Some(cn))) =>
                        {
                            // We're able to read the shading normal from the file.
                            tris.push(Tri::new(
                                    offset + av, offset + bv, offset + cv,
                                    noffset + an, noffset + bn, noffset + cn));
                        },
                        wavefront_obj::obj::Primitive::Triangle(
                            (av, _, _), (bv, _, _), (cv, _, _)) =>
                        {
                            // Compute the geometric normal, and use that in place of the shading
                            // normal.
                            let edge1 = &vertices[offset + bv] - &vertices[offset + av];
                            let edge2 = &vertices[offset + cv] - &vertices[offset + av];
                            let normal = edge1.cross(&edge2).normalized();
                            normals.push(normal);

                            tris.push(Tri::new(
                                    offset + av,
                                    offset + bv,
                                    offset + cv,
                                    normals.len() - 1,
                                    normals.len() - 1,
                                    normals.len() - 1));
                        },
                        _ => {}
                    }
                }
            }
        }

        let mesh = Mesh {
            mat: material,
            vertices: vertices,
            normals: normals,
            tris: tris
        };
        Ok(mesh)
    }
}

impl Display for Mesh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mesh({} vertices, {} triangles)", self.vertices.len(), self.tris.len())
    }
}

impl prim::Prim for Mesh {
    fn num_components(&self) -> usize {
        self.tris.len()
    }

    fn display_color(&self) -> &core::Vec {
        &self.mat.display_color()
    }

    fn material(&self) -> &material::Material {
        &self.mat
    }

    fn local_to_world_xform(&self) -> &core::Xform {
        &core::Xform::identity_ref()
    }

    /**
     * This is unimplemented for meshes, because meshes are always stored in world space.
     */
    fn bbox_local(&self, _: usize) -> core::BBox {
        unreachable!();
    }

    fn bbox_world(&self, component: usize) -> core::BBox {
        let tri = &self.tris[component];
        core::BBox::empty()
                .union_with(&self.vertices[tri.a])
                .union_with(&self.vertices[tri.b])
                .union_with(&self.vertices[tri.c])
    }

    /**
     * This is unimplemented for meshes, because meshes are always stored in world space.
     */
    fn intersect_local(&self, _: &core::Ray, _: usize) -> (f32, prim::SurfaceProperties) {
        unreachable!();
    }

    /**
     * Intersects the given ray in world space with the prim, and returns the distance along the
     * ray and the surface properties at the point of intersection.
     */
    fn intersect_world(&self, ray: &core::Ray, component: usize) -> (f32, prim::SurfaceProperties) {
        let tri = &self.tris[component];
        let a = &self.vertices[tri.a];
        let b = &self.vertices[tri.b];
        let c = &self.vertices[tri.c];
        let an = &self.normals[tri.an];
        let bn = &self.normals[tri.bn];
        let cn = &self.normals[tri.cn];

        // Uses the Moller-Trumbore intersection algorithm.
        // See <http://en.wikipedia.org/wiki/Moller-Trumbore_intersection_algorithm> for more info.
        let edge1 = b - a;
        let edge2 = c - a;

        let p = ray.direction.cross(&edge2);
        let det = edge1.dot(&p);
        if core::is_nearly_zero(det) {
            return (0.0, prim::SurfaceProperties::zero()); // No hit on plane.
        }

        let inv_det = 1.0 / det;
        let t = &ray.origin - &a;
        let u = &t.dot(&p) * inv_det;
        if u < 0.0 || u > 1.0 {
            return (0.0, prim::SurfaceProperties::zero()); // In plane but not triangle.
        }

        let q = t.cross(&edge1);
        let v = ray.direction.dot(&q) * inv_det;
        if v < 0.0 || (u + v) > 1.0 {
            return (0.0, prim::SurfaceProperties::zero()); // In plane but not triangle.
        }

        let dist = edge2.dot(&q) * inv_det;
        if !core::is_positive(dist) {
            return (0.0, prim::SurfaceProperties::zero()); // In triangle but behind us.
        }

        // XXX: Maybe use UV coordinates to setup tangent and binormal.
        // That way we'd get consistent tangents.
        let w = 1.0 - u - v;
        let normal = (&(&(w * an) + &(u * bn)) + &(v * cn)).normalized();
        let (tangent, binormal) = normal.coord_system();
        let surface_props = prim::SurfaceProperties::new(normal, tangent, binormal, normal);

        return (dist, surface_props);
    }
}