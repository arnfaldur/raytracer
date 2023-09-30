use std::io::Write;

type Value = f64;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: Value,
    g: Value,
    b: Value,
}

impl Color {
    pub fn new(r: Value, g: Value, b: Value) -> Self {
        Self { r, g, b }
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
    pub fn write_to_file(&self, file: &mut std::fs::File) -> std::io::Result<()> {
        let ir = (255.999 * self.r) as u8;
        let ig = (255.999 * self.g) as u8;
        let ib = (255.999 * self.b) as u8;

        file.write_all(format!("{} {} {}\n", ir, ig, ib).as_bytes())?;
        Ok(())
    }
}

impl std::ops::Neg for Color {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.r, -self.g, -self.b)
    }
}
impl std::ops::Add for Color {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}
impl std::ops::Sub for Color {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b)
    }
}
impl std::ops::Mul for Color {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.r * rhs.r, self.g * rhs.g, self.b * rhs.b)
    }
}
impl std::ops::Mul<Value> for Color {
    type Output = Self;
    fn mul(self, rhs: Value) -> Self::Output {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}
impl std::ops::Div for Color {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.r / rhs.r, self.g / rhs.g, self.b / rhs.b)
    }
}
impl std::ops::Div<Value> for Color {
    type Output = Self;
    fn div(self, rhs: Value) -> Self::Output {
        Self::new(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}
