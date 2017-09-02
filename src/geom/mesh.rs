use geom::prim;

use core;
use material;

use std;
use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use rand;
use rand::distributions::IndependentSample;
use wavefront_obj;

struct Tri {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub an: usize,
    pub bn: usize,
    pub cn: usize,
    pub at: usize,
    pub bt: usize,
    pub ct: usize,
}

impl Tri {
    pub fn new(
        a: usize, b: usize, c: usize,
        an: usize, bn: usize, cn: usize,
        at: usize, bt: usize, ct: usize) -> Tri
    {
        Tri {a: a, b: b, c: c, an: an, bn: bn, cn: cn, at: at, bt: bt, ct: ct}
    }

    fn area(&self, vertices: &std::vec::Vec<core::Vec>) -> f32 {
        // A triangle is half of a corresponding parallelogram.
        // The area of a parallelogram is given by the length of the cross product of the two sides.
        // Thus the area of a triangle is half the length of the cross product of two sides.
        let a = &vertices[self.a];
        let b = &vertices[self.b];
        let c = &vertices[self.c];
        let edge1 = a - c;
        let edge2 = b - c;
        0.5 * edge1.cross(&edge2).magnitude()
    }
}

pub struct Mesh {
    mat: material::Material,
    vertices: std::vec::Vec<core::Vec>,
    normals: std::vec::Vec<core::Vec>,
    uvs: std::vec::Vec<core::Vec>, // XXX: This is probably wasteful since we only need xy-coords.
    tris: std::vec::Vec<Tri>,
    area_dist: core::CumulativeDistribution,
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
        let mut uvs = std::vec::Vec::<core::Vec>::new();
        let mut tris = std::vec::Vec::<Tri>::new();
        let xform = core::Xform::new(mat);
        for obj in obj_set.objects {
            // Copy all vertices.
            let offset = vertices.len();
            for v in obj.vertices {
                vertices.push(xform.transform(&core::Vec::new(v.x as f32, v.y as f32, v.z as f32)));
            }

            // Copy all normals.
            // Note: we do not assign any meaning to non-unit normals, and the rest of the rendering
            // system relies on there being unit-length normals, so normalize here.
            let noffset = normals.len();
            for n in obj.normals {
                let n2 = core::Vec::new(n.x as f32, n.y as f32, n.z as f32);
                if n2.is_nearly_zero() {
                    normals.push(n2);
                }
                else {
                    normals.push(xform.transform_normal(&n2).normalized());
                }
            }

            let toffset = uvs.len();
            for t in obj.tex_vertices {
                uvs.push(core::Vec::new(t.u as f32, t.v as f32, 0.0));
            }

            // Copy all triangles.
            for g in obj.geometry {
                for s in g.shapes {
                    if let wavefront_obj::obj::Primitive::Triangle(a, b, c) = s.primitive {
                        let (av, bv, cv) = (offset + a.0, offset + b.0, offset + c.0);
                        let (at, bt, ct) = match (a.1, b.1, c.1) {
                            (Some(at), Some(bt), Some(ct)) => {
                                (toffset + at, toffset + bt, toffset + ct)
                            },
                            _ => {
                                uvs.push(core::Vec::zero());
                                (uvs.len() - 1, uvs.len() - 1, uvs.len() - 1)
                            }
                        };
                        let (an, bn, cn) = match (a.2, b.2, c.2) {
                            (Some(an), Some(bn), Some(cn)) if
                                !normals[noffset + an].is_nearly_zero() &&
                                !normals[noffset + bn].is_nearly_zero() &&
                                !normals[noffset + cn].is_nearly_zero() =>
                            {
                                // We're able to read the shading normal from the file, and the
                                // normals are non-zero.
                                (noffset + an, noffset + bn, noffset + cn)
                            },
                            _ => {
                                // Either we're missing a shading normal, or at least one of the
                                // normals is degenerate. Use usize::MAX as a sentinel to indicate
                                // that we should use the geometric normal instead.
                                (std::usize::MAX, std::usize::MAX, std::usize::MAX)
                            }
                        };
                        tris.push(Tri::new(av, bv, cv, an, bn, cn, at, bt, ct));
                    }
                }
            }
        }

        // Compute CDF over area so we can sample uniformly over area.
        let mut area_cdf = std::vec::Vec::<f32>::with_capacity(tris.len());
        let mut total_area = 0.0;
        for i in 0..tris.len() {
            total_area += tris[i].area(&vertices);
            area_cdf.push(total_area);
        }
        for i in 0..tris.len() {
            area_cdf[i] = area_cdf[i] / total_area;
        }

        vertices.shrink_to_fit();
        normals.shrink_to_fit();
        uvs.shrink_to_fit();
        tris.shrink_to_fit();
        area_cdf.shrink_to_fit();

        let mesh = Mesh {
            mat: material,
            vertices: vertices,
            normals: normals,
            uvs: uvs,
            tris: tris,
            area_dist: core::CumulativeDistribution::new(area_cdf)
        };
        Ok(mesh)
    }

    fn compute_surface_props(&self, tri: &Tri, u: f32, v: f32, w: f32) -> prim::SurfaceProperties {
        let a = &self.vertices[tri.a];
        let b = &self.vertices[tri.b];
        let c = &self.vertices[tri.c];
        let edge1 = a - c;
        let edge2 = b - c;

        let at = &self.uvs[tri.at];
        let bt = &self.uvs[tri.bt];
        let ct = &self.uvs[tri.ct];
        let uv1 = at - ct;
        let uv2 = bt - ct;

        // Geometric normal.
        let geom_normal = edge1.cross(&edge2).normalized();

        // Shading normal.
        let normal = if tri.an == std::usize::MAX {
            // No shading normals. Use geometric normal instead.
            geom_normal
        }
        else {
            // Compute the shading normal from barycentric coordintes.
            let an = &self.normals[tri.an];
            let bn = &self.normals[tri.bn];
            let cn = &self.normals[tri.cn];
            (&(&(u * an) + &(v * bn)) + &(w * cn)).normalized()
        };

        // Compute the derivative dpos/du. See PBRT 3e p. 158.
        // pos = pos_0 + u * dpos/du + v * dpos/dv
        // Note: I'm not computing dpdv here because there's no need for it yet.
        let uv_det = uv1.x * uv2.y + uv1.y * uv2.x;
        let (tangent, binormal) = if uv_det == 0.0 {
            // Just use an arbitrary coordinate system for tangent and binormal if we can't
            // compute analytically.
            normal.coord_system()
        }
        else {
            // Compute tangent and binormal analytically.
            // i × j = k, k × i = j
            // normal × dpdu = binormal, binormal × normal = tangent
            let inv_uv_det = 1.0 / uv_det;
            let dpdu = &(&(uv2.y * &uv1) - &(uv1.y * &uv2)) * inv_uv_det;
            let binormal = normal.cross(&dpdu).normalized();
            let tangent = binormal.cross(&normal);
            (tangent, binormal)
        };

        prim::SurfaceProperties::new(normal, tangent, binormal, geom_normal)
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

        // Uses the Moller-Trumbore intersection algorithm.
        // See <http://en.wikipedia.org/wiki/Moller-Trumbore_intersection_algorithm> for more info.
        let edge1 = a - c;
        let edge2 = b - c;

        let p = ray.direction.cross(&edge2);
        let det = edge1.dot(&p);
        if core::is_nearly_zero(det) {
            return (0.0, prim::SurfaceProperties::zero()); // No hit on plane.
        }

        let inv_det = 1.0 / det;
        let t = &ray.origin - &c;
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

        let w = 1.0 - u - v;
        let surface_props = self.compute_surface_props(tri, u, v, w);
        return (dist, surface_props);
    }

    fn sample(&self, rng: &mut rand::XorShiftRng) -> (core::Vec, prim::SurfaceProperties) {
        let tri_index = self.area_dist.ind_sample(rng);
        let tri = &self.tris[tri_index];
        let a = &self.vertices[tri.a];
        let b = &self.vertices[tri.b];
        let c = &self.vertices[tri.c];

        let uniform_sample_barycentric = core::UniformSampleBarycentric {};
        let (u, v) = uniform_sample_barycentric.ind_sample(rng);
        let w = 1.0 - u - v;
        let pt = &(&(u * a) + &(v * b)) + &(w * c);

        let surface_props = self.compute_surface_props(tri, u, v, w);
        (pt, surface_props)
    }
}