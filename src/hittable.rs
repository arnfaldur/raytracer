use std::{ops::{Range, Neg}, sync::Arc};

use crate::{
    color::Color,
    range::Membership,
    ray::Ray,
    vec3::{Point3, Vec3},
};

pub trait Hittable {
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
}
impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, ray_trange: Range<f64>) -> Option<HitRecord> {
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

        if !ray_trange.exclusive(root) {
            root = (-alignment + sqrtd) / squared_raydir_magnitude;
            if (!ray_trange.exclusive(root)) {
                return None;
            }
        }

        let intersection_point = ray.at(root);
        let outward_normal = (intersection_point - self.center) / self.radius;
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
        let mut hit_anything = false;
        let mut closest_so_far = ray_trange.end;
        let mut result = None;
        let boi = 0.0..=1.0;

        for object in self.objects.iter() {
            if let Some(hit_record) = object.hit(ray, ray_trange.start..closest_so_far) {
                hit_anything = true;
                closest_so_far = hit_record.t;
                result = Some(hit_record);
            }
        }
        return result;
    }
}

pub struct Lambertian {
    albedo: Color,
}

pub trait Material {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)>;
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let scatter_direction = hit_record.normal + Vec3::random_on_unit_sphere();
        let scatter_direction = if scatter_direction.near_zero() {
            hit_record.normal
        } else {
            scatter_direction
        };
        let scattered_ray = Ray::new(hit_record.point, scatter_direction);
        return Some((self.albedo, scattered_ray));
    }
}

impl From<Color> for Lambertian {
    fn from(value: Color) -> Self {
        Self { albedo: value }
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = ray.direction.reflect(&hit_record.normal);
        let scatter_direction = reflected + self.fuzz * Vec3::random_on_unit_sphere();
        let scattered_ray = Ray::new(hit_record.point, scatter_direction);
        return Some((self.albedo, scattered_ray));
    }
}

impl From<Color> for Metal {
    fn from(value: Color) -> Self {
        Self::new(value, 0.)
    }
}

pub struct Dielectric {
    index_of_refraction: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Self {
        Self { index_of_refraction }
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let refraction_ratio = if hit_record.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let unit_direction = ray.direction.unit_vector();
        let cos_theta = (-unit_direction).dot(&hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction = if cannot_refract {
            unit_direction.reflect(&hit_record.normal)
        } else {
            refract(&unit_direction, &hit_record.normal, refraction_ratio)
        };

        let scattered = Ray::new(hit_record.point, direction);

        return Some((Color::white(), scattered));
    }
}

fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = (-(*uv)).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (*uv + cos_theta * *n);
    let r_out_parallel = (1.0 - r_out_perp.length_squared()).abs().sqrt().neg() * *n;
    return r_out_perp + r_out_parallel;
}
