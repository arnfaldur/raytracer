use std::{
    cmp::Ordering,
    ops::{Neg, Range},
    sync::Arc, slice::IterMut,
};

use crate::{
    color::Color,
    random::Rng,
    range::Membership,
    ray::Ray,
    vec3::{Point3, Vec3},
};

use self::{aabb::AABB, materials::Material};

pub mod aabb;
pub mod materials;

pub trait Hittable: Sync {
    fn hit(&self, ray: &Ray, ray_trange: &Range<f64>) -> Option<HitRecord>;
    fn bounding_box(&self) -> &AABB;
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
    bounding_box: AABB,
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

    fn calculate_hit(
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
        return Some(HitRecord {
            point: intersection_point,
            normal: if front_face { 1. } else { -1. } * outward_normal,
            material: self.material.clone(),
            t: root,
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

pub struct MovingSphere {
    sphere: Sphere,
    destination: Point3,
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

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
    bounding_box: AABB,
}

impl HittableList {
    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.bounding_box = AABB::from_boxes(&self.bounding_box, object.bounding_box());
        self.objects.push(object);
    }
    pub fn into_bvh(mut self) -> Box<dyn Hittable> {
        let mut rng = Rng::new();
        return BVHNode::from_vec(&mut self.objects, 0, &mut rng);
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, ray_trange: &Range<f64>) -> Option<HitRecord> {
        let mut closest_so_far = ray_trange.end;
        let mut result = None;

        for object in self.objects.iter() {
            if let Some(hit_record) = object.hit(ray, &(ray_trange.start..closest_so_far)) {
                closest_so_far = hit_record.t;
                result = Some(hit_record);
            }
        }
        return result;
    }

    fn bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}

pub struct BVHNode {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
    bounding_box: AABB,
}

impl BVHNode {
    fn from_vec(mut objects: &mut Vec<Box<dyn Hittable>>, start: usize, rng: &mut Rng) -> Box<dyn Hittable> {
        let axis = ((rng.next_f64() * 3.0) as usize).min(2);

        let comparator = if axis == 0 {
            BVHNode::box_x_compare
        } else if axis == 1 {
            BVHNode::box_y_compare
        } else if axis == 2 {
            BVHNode::box_z_compare
        } else {
            panic!("invalid axis")
        };

        let length = objects.len() - start;

        return if length == 1 {
            objects.pop().unwrap()
            //*objects.take_first_mut().unwrap()
        } else if length == 2 {
            let a = objects.pop().unwrap();
            let b = objects.pop().unwrap();
            let bounding_box = AABB::from_boxes(a.bounding_box(), b.bounding_box());
            Box::new(if comparator(&a, &b).is_lt() {
                BVHNode {
                    left: a,
                    right: b,
                    bounding_box,
                }
            } else {
                BVHNode {
                    left: b,
                    right: a,
                    bounding_box,
                }
            })
        } else {
            objects.split_at_mut(start).1.sort_by(comparator);
            let right = BVHNode::from_vec(objects, start + length / 2, rng);
            let left = BVHNode::from_vec(objects, start, rng);
            let bounding_box = AABB::from_boxes(left.bounding_box(), right.bounding_box());
            Box::new(BVHNode {
                left,
                right,
                bounding_box,
            })
        };
    }

    // fn box_compare(axis_index: usize) -> dyn Fn(dyn Hittable,  dyn Hittable) -> Ordering {
    //     |a: dyn Hittable, b: dyn Hittable| a.bounding_box().axis(axis_index).start.total_cmp(&b.bounding_box().axis(axis_index).end)
    // }
    fn box_x_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        a.bounding_box()
            .axis(0)
            .start
            .total_cmp(&b.bounding_box().axis(0).end)
    }
    fn box_y_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        a.bounding_box()
            .axis(1)
            .start
            .total_cmp(&b.bounding_box().axis(1).end)
    }
    fn box_z_compare(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
        a.bounding_box()
            .axis(2)
            .start
            .total_cmp(&b.bounding_box().axis(2).end)
    }
}

impl Hittable for BVHNode {
    fn hit(&self, ray: &Ray, ray_trange: &Range<f64>) -> Option<HitRecord> {
        if self.bounding_box.hit(ray).is_none() {
            return None;
        }

        if let Some(record) = self.left.hit(ray, ray_trange) {
            if let Some(record) = self.right.hit(ray, &(ray_trange.start..record.t)) {
                Some(record)
            } else {
                Some(record)
            }
        } else {
            self.right.hit(ray, ray_trange)
        }
    }

    fn bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}
