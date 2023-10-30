use std::sync::{mpsc::SyncSender, Arc};

use crate::{
    camera::{Camera, CameraBuilder},
    color::Color,
    hittable::{Dielectric, Hittable, HittableList, Lambertian, Metal, Sphere},
    vec3::{Point3, Vec3},
};

pub struct Scene<W> {
    pub camera: Camera,
    world: W,
}

impl Scene<Box<dyn Hittable>> {
    pub fn new(camera: Camera, world: Box<dyn Hittable>) -> Self {
        Self { camera, world }
    }
    pub fn render(&self, sender: SyncSender<((usize, usize), (usize, usize), Vec<Color>)>) {
        self.camera.render(&self.world, sender);
    }
}

pub fn composition(camera_builder: CameraBuilder) -> Scene<Box<dyn Hittable>> {
    let camera = camera_builder
        .field_of_view(55.0)
        .lookfrom(Point3::new(0.0, 0.5, 1.0) * 1.5)
        .lookat(Point3::new(0.0, 0.3, 0.0))
        .up_vector(Vec3::new(0.0, 1.0, 0.0))
        .defocus_angle(0.0)
        .build();
    let mut world = Box::new(HittableList::default());

    // Ground
    world.add(Box::new(Sphere::new(
        Point3::new(0., -40_000_000.5, 0.),
        40_000_000.,
        Arc::new(Lambertian::from(Color::new(0.05, 0.20, 0.07))),
    )));

    let blue_lamb = Arc::new(Lambertian::from(Color::new(0.1, 0.1, 0.8)));
    let red_lamb = Arc::new(Lambertian::from(Color::new(0.8, 0.1, 0.1)));

    // Ballz
    world.add(Box::new(Sphere::new(
        Point3::new(0., 0., -1.),
        0.5,
        red_lamb,
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(1.3, 0., -1.7),
        0.5,
        blue_lamb,
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-0.25 - 0.125, -0.25, -0.5),
        0.25,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-0.25 - 0.125, -0.25, -0.5),
        -0.20,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.6, 0.1, -0.4),
        0.3,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-1.0, 0., -1.0),
        0.5,
        Arc::new(Metal::new(Color::gray(0.7), 0.0)),
    )));
    return Scene::new(camera, world);
}
