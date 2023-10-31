use std::{f64::consts::PI, ops::Range, sync::Arc};

use crate::{
    range::Membership,
    ray::Ray,
    vec3::{Point3, Vec3},
};

use super::{aabb::AABB, materials::Material, HitRecord, Hittable};

#[derive(Debug)]
pub struct Sphere {
    pub(crate) center: Point3,
    pub(crate) radius: f64,
    pub(crate) material: Arc<dyn Material>,
    pub(crate) bounding_box: AABB,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, material: Arc<dyn Material>) -> Self {
        let radius_vec = Vec3::new(radius, radius, radius);
        Self {
            center,
            radius,
            material,
            bounding_box: AABB::from_vecs(center - radius_vec, center + radius_vec),
        }
    }

    fn get_sphere_uv(p: &Point3) -> (f64, f64) {
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;
        (phi / (2. * PI), theta / PI)
    }

    pub(crate) fn calculate_hit(
        &self,
        ray: &Ray,
        ray_trange: &Range<f64>,
        center: Point3,
    ) -> Option<HitRecord> {
        let sphere_to_ray = ray.origin - center;
        let squared_raydir_magnitude = ray.direction.length_squared();
        let alignment = sphere_to_ray.dot(&ray.direction);
        let surface_dist = sphere_to_ray.length_squared() - self.radius.powi(2);
        let discriminant = alignment.powi(2) - squared_raydir_magnitude * surface_dist;
        if discriminant < 0. {
            return None;
        }
        let sqrtd = discriminant.sqrt();
        let mut root = (-alignment - sqrtd) / squared_raydir_magnitude;
        if !ray_trange.exclusive(root) {
            root = (-alignment + sqrtd) / squared_raydir_magnitude;
            if !ray_trange.exclusive(root) {
                return None;
            }
        }
        let intersection_point = ray.at(root);
        let outward_normal = (intersection_point - center) / self.radius;
        let front_face = ray.direction.dot(&outward_normal) < 0.;
        let (u, v) = Sphere::get_sphere_uv(&outward_normal);
        return Some(HitRecord {
            point: intersection_point,
            normal: if front_face { 1. } else { -1. } * outward_normal,
            material: self.material.clone(),
            t: root,
            u,
            v,
            front_face,
        });
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, ray_trange: &Range<f64>) -> Option<HitRecord> {
        return self.calculate_hit(ray, ray_trange, self.center);
    }
    fn bounding_box(&self) -> &AABB {
        return &self.bounding_box;
    }
}

#[derive(Debug)]
pub struct MovingSphere {
    pub(crate) sphere: Sphere,
    pub(crate) destination: Point3,
}

impl MovingSphere {
    pub fn new(sphere: Sphere, destination: Point3) -> Self {
        let radius_vec = Vec3::new(sphere.radius, sphere.radius, sphere.radius);
        let dest_bb = AABB::from_vecs(destination - radius_vec, destination + radius_vec);
        Self {
            sphere: Sphere {
                bounding_box: AABB::from_boxes(&sphere.bounding_box, &dest_bb),
                ..sphere
            },
            destination,
        }
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, ray_trange: &Range<f64>) -> Option<HitRecord> {
        return self.sphere.calculate_hit(
            ray,
            ray_trange,
            self.sphere.center * (1. - ray.time) + self.destination * ray.time,
        );
    }
    fn bounding_box(&self) -> &AABB {
        &self.sphere.bounding_box
    }
}
