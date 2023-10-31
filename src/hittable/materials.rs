use std::{
    fmt::Debug,
    ops::{Neg, Range},
    sync::Arc,
};

use super::{
    texture::{SolidColor, Texture},
    HitRecord,
};
use crate::{color::Color, random::Rng, ray::Ray, vec3::Vec3};

#[derive(Debug)]
pub struct Lambertian {
    pub albedo: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

pub trait Material: Sync + Send + Debug {
    fn scatter(&self, rng: &mut Rng, ray: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)>;
}

impl Material for Lambertian {
    fn scatter(&self, rng: &mut Rng, ray: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let scatter_direction = hit_record.normal + Vec3::random_on_unit_sphere(rng);
        let scatter_direction = if scatter_direction.near_zero() {
            hit_record.normal
        } else {
            scatter_direction
        };
        let scattered_ray = Ray::new(hit_record.point, scatter_direction, ray.time);
        return Some((
            self.albedo
                .value(hit_record.u, hit_record.v, &hit_record.point),
            scattered_ray,
        ));
    }
}

impl From<Color> for Lambertian {
    fn from(value: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::from(value)),
        }
    }
}
impl From<Arc<dyn Texture>> for Lambertian {
    fn from(value: Arc<dyn Texture>) -> Self {
        Self { albedo: value }
    }
}

#[derive(Debug)]
pub struct Metal {
    pub(crate) albedo: Color,
    pub(crate) fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl Material for Metal {
    fn scatter(&self, rng: &mut Rng, ray: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let reflected = ray.direction.reflect(&hit_record.normal);
        let scatter_direction = reflected + self.fuzz * Vec3::random_on_unit_sphere(rng);
        let scattered_ray = Ray::new(hit_record.point, scatter_direction, ray.time);
        return Some((self.albedo, scattered_ray));
    }
}

impl From<Color> for Metal {
    fn from(value: Color) -> Self {
        Self::new(value, 0.)
    }
}

#[derive(Debug)]
pub struct Dielectric {
    pub(crate) index_of_refraction: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Self {
        Self {
            index_of_refraction,
        }
    }
    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl Material for Dielectric {
    fn scatter(&self, rng: &mut Rng, ray: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray)> {
        let refraction_ratio = if hit_record.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let unit_direction = ray.direction.unit_vector();
        let cos_theta = (-unit_direction).dot(&hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction =
            if cannot_refract || reflectance(cos_theta, refraction_ratio) > rng.next_f64() {
                unit_direction.reflect(&hit_record.normal)
            } else {
                refract(&unit_direction, &hit_record.normal, refraction_ratio)
            };

        let scattered = Ray::new(hit_record.point, direction, ray.time);

        return Some((Color::white(), scattered));
    }
}

pub(crate) fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = (-(*uv)).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (*uv + cos_theta * *n);
    let r_out_parallel = (1.0 - r_out_perp.length_squared()).abs().sqrt().neg() * *n;
    return r_out_perp + r_out_parallel;
}

pub(crate) fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
    let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
    return r0 + (1.0 - r0) * (1.0 - cosine).powi(5);
}
