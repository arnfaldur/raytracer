use crate::{
    ray::{Point3, Ray},
    vec3::Vec3,
};

trait Hittable {
    fn hit(&self, ray: Ray, ray_tmin: f64, ray_tmax: f64, rec: &HitRecord) -> Option<HitRecord>;
}

struct HitRecord {
    point: Point3,
    normal: Vec3,
    t: f64,
    front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vec3) {
        self.front_face = ray.direction.dot(&outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        }
    }
}

struct Sphere {
    center: Point3,
    radius: f64,
}
impl Hittable for Sphere {
    fn hit(&self, ray: Ray, ray_tmin: f64, ray_tmax: f64, rec: &HitRecord) -> Option<HitRecord> {
        let sphere_to_ray = ray.origin - self.center;
        let squared_raydir_magnitude = ray.direction.length_squared();
        let alignment = sphere_to_ray.dot(&ray.direction);
        let surface_dist = sphere_to_ray.length_squared() - self.radius.powi(2);

        let discriminant = alignment.powi(2) - squared_raydir_magnitude * surface_dist;
        if discriminant < 0. {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (-alignment - sqrtd) / squared_raydir_magnitude;

        if (root <= ray_tmin || ray_tmax <= root) {
            root = (-alignment + sqrtd) / squared_raydir_magnitude;
            if (root <= ray_tmin || ray_tmax <= root) {
                return None;
            }
        }

        let intersection_point = ray.at(root);
        let outward_normal = (intersection_point - self.center) / self.radius;
        let front_face = ray.direction.dot(&outward_normal) < 0.;

        return Some(HitRecord {
            point: intersection_point,
            normal: if front_face { 1. } else { -1. } * outward_normal,
            t: root,
            front_face,
        });
    }
}
