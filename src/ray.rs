use crate::{
    color::Color,
    vec3::{Point3, Vec3},
};

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub time: f64,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3, time: f64) -> Self {
        Self { origin, direction, time }
    }
    pub fn at(&self, t: f64) -> Point3 {
        self.origin + t * self.direction
    }
    pub fn color(&self) -> Color {
        let unit_direction = self.direction.unit_vector();
        let a = 0.5 * (unit_direction.y + 1.0);
        (1.0 - a) * Color::new(1., 1., 1.) + a * Color::new(0.5, 0.7, 1.)
    }
}
