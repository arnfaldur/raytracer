use std::ops::{Neg, Add, Sub, Mul, Div};


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

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y, -self.z)
    }
}
impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl Mul for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}
impl Mul<Value> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Value) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}
impl Mul<Vec3> for Value {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::Output::new(rhs.x * self, rhs.y * self, rhs.z * self)
    }
}
impl Div for Vec3 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}
impl Div<Value> for Vec3 {
    type Output = Self;
    fn div(self, rhs: Value) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}
impl Div<Vec3> for Value {
    type Output = Vec3;
    fn div(self, rhs: Vec3) -> Self::Output {
        Self::Output::new(rhs.x / self, rhs.y / self, rhs.z / self)
    }
}
