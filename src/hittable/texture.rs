use std::{
    fmt::Debug,
    ops::{Div, Rem, Shl},
};

use image::{ImageBuffer, RgbaImage};

use crate::{color::Color, random::Rng, vec3::Point3};

pub trait Texture: Send + Sync + Debug {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color;
}

#[derive(Debug)]
pub struct SolidColor {
    color: Color,
}

impl SolidColor {}

impl From<Color> for SolidColor {
    fn from(color: Color) -> Self {
        Self { color }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _point: &Point3) -> Color {
        self.color
    }
}

#[derive(Debug)]
pub struct CheckerTexture {
    inv_scale: f64,
    odd: Box<dyn Texture>,
    even: Box<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, odd: Box<dyn Texture>, even: Box<dyn Texture>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            odd,
            even,
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color {
        let x = (point.x * self.inv_scale).floor() as i32;
        let y = (point.y * self.inv_scale).floor() as i32;
        let z = (point.z * self.inv_scale).floor() as i32;
        if ((x + y + z).rem_euclid(2) == 0) {
            self.odd.value(u, v, point)
        } else {
            self.even.value(u, v, point)
        }
    }
}

#[derive(Debug)]
pub struct ImageTexture {
    image: RgbaImage,
}

impl ImageTexture {
    pub fn new(image: RgbaImage) -> Self {
        Self { image }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, point: &Point3) -> Color {
        if self.image.width() == 0 || self.image.height() == 0 {
            return Color::cyan();
        }

        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let i = (u * self.image.width() as f64) as u32;
        let j = (v * self.image.height() as f64) as u32;
        let pixel = self.image.get_pixel(i, j);
        Color::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64) / 255.0
    }
}

#[derive(Debug)]
pub struct NoiseTexture {
    inv_scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        Self {
            inv_scale: 1.0 / scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        // let x = (point.x * self.inv_scale).floor() as i32;
        // let y = (point.y * self.inv_scale).floor() as i32;
        // let z = (point.z * self.inv_scale).floor() as i32;
        // let a = x as u64;
        // let b = (y as u64).wrapping_add((z as u64).wrapping_shl(32));
        // let mut rng = Rng::from_seed([a, b]);
        // rng.short_jump();
        let x = (point.x * self.inv_scale).floor() as i32;
        let y = (point.y * self.inv_scale).floor() as i32;
        let z = (point.z * self.inv_scale).floor() as i32;
        let a = x as u64;
        let b = (y as u64).wrapping_add((z as u64).wrapping_shl(32));
        let mut rng = Rng::from_seed([a, b]);
        rng.short_jump();
        Color::gray(rng.next_f64())
    }
}
