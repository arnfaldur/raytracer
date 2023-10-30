use std::sync::mpsc::SyncSender;

use crate::{
    camera::{Camera, CameraBuilder},
    hittable::Hittable, color::Color,
};

struct Scene {
    camera: Camera,
    world: Box<dyn Hittable>,
}

impl Scene {
    fn new(camera: Camera, world: Box<dyn Hittable>) -> Self {
        Self { camera, world }
    }
    fn render(&self, sender: SyncSender<((usize, usize), (usize, usize), Vec<Color>)>) {
        self.camera.render(&self.world, sender);
    }
}
