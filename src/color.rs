use std::fs::File;
use std::io::{BufWriter, Result, Write};
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::vec3::Vec3;

type Value = f64;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: Value,
    pub g: Value,
    pub b: Value,
}

impl Color {
    pub fn new(r: Value, g: Value, b: Value) -> Self {
        Self { r, g, b }
    }
    pub fn gray(value: Value) -> Self {
        Self::new(value, value, value)
    }
    pub fn white() -> Self {
        Self::new(1., 1., 1.)
    }
    pub fn black() -> Self {
        Self::new(0., 0., 0.)
    }
    pub fn dot(&self, rhs: &Self) -> Value {
        self.r * rhs.r + self.g * rhs.g + self.b * rhs.b
    }
    pub fn cross(&self, rhs: &Self) -> Self {
        Self::new(
            self.g * rhs.b - self.b * rhs.g,
            self.b * rhs.r - self.r * rhs.b,
            self.r * rhs.g - self.g * rhs.r,
        )
    }
    pub fn length_squared(&self) -> Value {
        self.r.powi(2) + self.g.powi(2) + self.b.powi(2)
    }
    pub fn length(&self) -> Value {
        self.length_squared().sqrt()
    }
    pub fn unit_vector(&self) -> Self {
        *self / self.length()
    }
    pub fn write_to_writer(&self, writer: &mut BufWriter<File>) -> Result<()> {
        let ir = (255.999 * self.r) as u8;
        let ig = (255.999 * self.g) as u8;
        let ib = (255.999 * self.b) as u8;

        writer.write_all(format!("{} {} {}\n", ir, ig, ib).as_bytes())?;
        Ok(())
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl Neg for Color {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.r, -self.g, -self.b)
    }
}
macro_rules! impl_color_ops {
    ($trait:ident, $op:ident, $type:ty) => {
        impl $trait for $type {
            type Output = Self;
            fn $op(self, rhs: Self) -> Self::Output {
                Self::new(self.r.$op(rhs.r), self.g.$op(rhs.g), self.b.$op(rhs.b))
            }
        }

        impl $trait<Value> for $type {
            type Output = Self;
            fn $op(self, rhs: Value) -> Self::Output {
                Self::new(self.r.$op(rhs), self.g.$op(rhs), self.b.$op(rhs))
            }
        }

        impl $trait<$type> for Value {
            type Output = $type;
            fn $op(self, rhs: $type) -> Self::Output {
                Self::Output::new(rhs.r.$op(self), rhs.g.$op(self), rhs.b.$op(self))
            }
        }
    };
}

impl_color_ops!(Add, add, Color);
impl_color_ops!(Sub, sub, Color);
impl_color_ops!(Mul, mul, Color);
impl_color_ops!(Div, div, Color);
