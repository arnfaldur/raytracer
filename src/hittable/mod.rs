use std::{
    ops::{Neg, Range},
    sync::Arc,
};

use crate::{
    color::Color,
    random::Rng,
    range::Membership,
    ray::Ray,
    vec3::{Point3, Vec3},
};

use self::materials::Material;

pub mod materials;

pub trait Hittable: Sync {
    fn hit(&self, ray: &Ray, ray_trange: Range<f64>) -> Option<HitRecord>;
}

pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub front_face: bool,
    pub t: f64,
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

pub struct Sphere {
    center: Point3,
    radius: f64,
    material: Arc<dyn Material>,
}
impl Sphere {
    pub fn new(center: Point3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }

    fn calculate_hit(
        &self,
        ray: &Ray,
        ray_trange: Range<f64>,
        center: Point3,
    ) -> Option<HitRecord> {
        let sphere_to_ray = ray.origin - center;
        let squared_raydir_magnitude = ray.direction.length_squared();
        let alignment = sphere_to_ray.dot(&ray.direction);
        let surface_dist = sphere_to_ray.length_squared() - self.radius.powi(2);
        let discriminant = alignment.powi(2) - squared_raydir_magnitude * surface_dist;
        if discriminant < 0. {
            return (None);
        }
        let sqrtd = discriminant.sqrt();
        let mut root = (-alignment - sqrtd) / squared_raydir_magnitude;
        if !ray_trange.exclusive(root) {
            root = (-alignment + sqrtd) / squared_raydir_magnitude;
            if !ray_trange.exclusive(root) {
                return (None);
            }
        }
        let intersection_point = ray.at(root);
        let outward_normal = (intersection_point - center) / self.radius;
        let front_face = ray.direction.dot(&outward_normal) < 0.;
        return (Some(HitRecord {
            point: intersection_point,
            normal: if front_face { 1. } else { -1. } * outward_normal,
            material: self.material.clone(),
            t: root,
            front_face,
        }));
    }
}
impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, ray_trange: Range<f64>) -> Option<HitRecord> {
        return self.calculate_hit(ray, ray_trange, self.center);
    }
}

pub struct MovingSphere {
    sphere: Sphere,
    destination: Point3,
}
impl MovingSphere {
    pub fn new(sphere: Sphere, destination: Point3) -> Self {
        Self {
            sphere,
            destination,
        }
    }
}
impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, ray_trange: Range<f64>) -> Option<HitRecord> {
        return self.sphere.calculate_hit(
            ray,
            ray_trange,
            self.sphere.center * (1. - ray.time) + self.destination * ray.time,
        );
    }
}

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, ray_trange: Range<f64>) -> Option<HitRecord> {
        let mut closest_so_far = ray_trange.end;
        let mut result = None;

        for object in self.objects.iter() {
            if let Some(hit_record) = object.hit(ray, ray_trange.start..closest_so_far) {
                closest_so_far = hit_record.t;
                result = Some(hit_record);
            }
        }
        return result;
    }
}
