use std::{
    f64,
    fmt::Debug,
    ops::{Add, Div, Mul, Neg, Range, Sub},
};

use crate::{color::Color, random::Rng};

pub type Point3 = Vec3;

type Value = f64;

#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: Value,
    pub y: Value,
    pub z: Value,
}

impl Vec3 {
    pub fn new(x: Value, y: Value, z: Value) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self::new(0., 0., 0.)
    }
    pub fn random_in_unit_sphere_reject(rng: &mut Rng) -> Self {
        loop {
            let candidate = Self::new(
                rng.next_f64_range(-1.0..1.0),
                rng.next_f64_range(-1.0..1.0),
                rng.next_f64_range(-1.0..1.0),
            );
            if candidate.length_squared() < 1.0 {
                return candidate;
            }
        }
    }
    pub fn random_in_unit_sphere(rng: &mut Rng) -> Self {
        let theta = rng.next_f64_range(0.0..2.0 * f64::consts::PI);
        let z = rng.next_f64_range(-1.0..1.0);
        let r = (1.0 - z.powi(2)).sqrt();
        Self::new(r * theta.cos(), r * theta.sin(), z)
    }
    pub fn random_on_unit_sphere(rng: &mut Rng) -> Self {
        Self::random_in_unit_sphere(rng).normalized()
    }
    pub fn random_in_unit_circle(rng: &mut Rng) -> Self {
        loop {
            let candidate = Self::new(
                rng.next_f64_range(-1.0..1.0),
                rng.next_f64_range(-1.0..1.0),
                0.0,
            );
            if candidate.length_squared() < 1.0 {
                return candidate;
            }
        }
    }
    pub fn random_on_hemisphere(rng: &mut Rng, normal: &Vec3) -> Self {
        let random = Self::random_on_unit_sphere(rng);
        random.dot(normal).signum() * random
    }
    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        *self - 2.0 * (*self).dot(normal) * *normal
    }
    pub fn dot(&self, rhs: &Self) -> Value {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    pub fn cross(&self, rhs: &Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
    pub fn length_squared(&self) -> Value {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }
    pub fn length(&self) -> Value {
        self.length_squared().sqrt()
    }
    pub fn unit_vector(&self) -> Self {
        *self / self.length()
    }
    pub fn normalized(&self) -> Self {
        *self / self.length()
    }
    pub fn near_zero(&self) -> bool {
        let threshold = 1e-9;
        self.x.abs() < threshold && self.y.abs() < threshold && self.z.abs() < threshold
    }
    pub fn distance(&self, other: &Self) -> f64 {
        (*self - *other).length()
    }
}

impl From<Color> for Vec3 {
    fn from(value: Color) -> Self {
        Self::new(value.r, value.g, value.b)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}

macro_rules! impl_vec3_ops {
    ($trait:ident, $op:ident, $type:ty) => {
        impl $trait for $type {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                Self::new(self.x.$op(rhs.x), self.y.$op(rhs.y), self.z.$op(rhs.z))
            }
        }

        impl $trait<Value> for $type {
            type Output = Self;
            fn $op(self, rhs: Value) -> Self::Output {
                Self::new(self.x.$op(rhs), self.y.$op(rhs), self.z.$op(rhs))
            }
        }

        impl $trait<$type> for Value {
            type Output = $type;
            fn $op(self, rhs: $type) -> Self::Output {
                Self::Output::new(self.$op(rhs.x), self.$op(rhs.y), self.$op(rhs.z))
            }
        }
    };
}

impl_vec3_ops!(Add, add, Vec3);
impl_vec3_ops!(Sub, sub, Vec3);
impl_vec3_ops!(Mul, mul, Vec3);
impl_vec3_ops!(Div, div, Vec3);

impl Debug for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec3(")?;
        write!(f, "{:.3}, ", &self.x)?;
        write!(f, "{:.3}, ", &self.y)?;
        write!(f, "{:.3}", &self.z)?;
        write!(f, ")")

        // f.debug_tuple("Vec3")
        //     .field(&format_args!("{:.3}", &self.x))
        //     .field(&format_args!("{:.3}", &self.y))
        //     .field(&format_args!("{:.3}", &self.z))
        //     // .field(&self.y)
        //     // .field(&self.z)
        //     .finish()
    }
}

extern crate test;

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use super::*;
    use crate::random::Rng;
    use test::Bencher;

    #[bench]
    fn bench_random_in_unit_sphere(b: &mut Bencher) {
        let mut rng = Rng::new();
        b.iter(|| {
            black_box(Vec3::random_in_unit_sphere(&mut rng));
        });
    }

    #[bench]
    fn bench_random_in_unit_sphere_reject(b: &mut Bencher) {
        let mut rng = Rng::new();
        b.iter(|| {
            black_box(Vec3::random_in_unit_sphere_reject(&mut rng));
        });
    }
}
