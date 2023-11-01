use std::{
    fmt::Debug,
    ops::{Div, Rem, Shl},
    result,
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
        let x = point.x * self.inv_scale;
        let y = point.y * self.inv_scale;
        let z = point.z * self.inv_scale;
        let ix = x.floor() as i32;
        let iy = y.floor() as i32;
        let iz = z.floor() as i32;

        let linear_to_piecewise_quadratic = |x: f64| {
            if x < 0.5 {
                2. * x.powi(2)
            } else {
                1.0 - 2.0 * (x - 1.0).powi(2)
            }
        };
        let linear_to_hermite_cubic = |x: f64| x.powi(2) * (3.0 - 2.0 * x);

        let x_blend = linear_to_hermite_cubic(x.rem_euclid(1.0));
        let y_blend = linear_to_hermite_cubic(y.rem_euclid(1.0));
        let z_blend = linear_to_hermite_cubic(z.rem_euclid(1.0));

        let m00 =
            noise_at(ix + 0, iy + 0, iz + 0).blend(&noise_at(ix + 1, iy + 0, iz + 0), x_blend);
        let m01 =
            noise_at(ix + 0, iy + 0, iz + 1).blend(&noise_at(ix + 1, iy + 0, iz + 1), x_blend);
        let m10 =
            noise_at(ix + 0, iy + 1, iz + 0).blend(&noise_at(ix + 1, iy + 1, iz + 0), x_blend);
        let m11 =
            noise_at(ix + 0, iy + 1, iz + 1).blend(&noise_at(ix + 1, iy + 1, iz + 1), x_blend);

        let o0 = m00.blend(&m10, y_blend);
        let o1 = m01.blend(&m11, y_blend);

        let result = o0.blend(&o1, z_blend);

        result
    }
}

fn noise_at(x: i32, y: i32, z: i32) -> Color {
    let a = x as u64;
    let b = (y as u64).wrapping_add((z as u64).wrapping_shl(32));
    let mut rng = Rng::from_seed([a, b]);
    rng.short_jump();
    Color::gray(rng.next_f64())
}
