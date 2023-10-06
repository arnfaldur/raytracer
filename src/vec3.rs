use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::color::Color;

pub type Point3 = Vec3;

type Value = f64;

#[derive(Debug, Clone, Copy)]
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
                Self::Output::new(rhs.x.$op(self), rhs.y.$op(self), rhs.z.$op(self))
            }
        }
    };
}

impl_vec3_ops!(Add, add, Vec3);
impl_vec3_ops!(Sub, sub, Vec3);
impl_vec3_ops!(Mul, mul, Vec3);
impl_vec3_ops!(Div, div, Vec3);
