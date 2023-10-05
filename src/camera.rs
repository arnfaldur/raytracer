use crate::{color::Color, hittable::Hittable, ray::Ray};

struct Camera {}

impl Camera {
    pub fn render(world: Box<dyn Hittable>) {}
    fn initialize() {}
    fn ray_color(ray: &Ray, world: Box<dyn Hittable>) -> Color {}
}
