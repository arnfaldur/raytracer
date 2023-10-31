use std::{
    cmp::Ordering,
    f64::{},
    fmt::Debug,
    ops::{Neg, Range},
    slice::IterMut,
    sync::Arc,
};

use crate::{
    color::Color,
    random::Rng,
    range::{Membership, RangeExtensions},
    ray::Ray,
    vec3::{Point3, Vec3},
};

use self::{aabb::AABB, materials::Material};

pub mod aabb;
pub mod containers;
pub mod materials;
pub mod geometry;
pub mod texture;

pub trait Hittable: Sync + Debug {
    fn hit(&self, ray: &Ray, ray_trange: &Range<f64>) -> Option<HitRecord>;
    fn bounding_box(&self) -> &AABB;
}

pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
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
