#![allow(unused)]
#![feature(float_next_up_down)]

use std::sync::Arc;
use std::time::Instant;

use crate::camera::{Camera, CameraBuilder};
use crate::color::Color;
use crate::hittable::{HittableList, Lambertian, Sphere, Metal};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};
use hittable::Hittable;

mod camera;
mod color;
mod hittable;
mod range;
mod ray;
mod vec3;

fn main() -> std::io::Result<()> {
    let start_time = Instant::now();

    fastrand::seed(123);

    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        // .aspect_ratio(1.0)
        .image_width(900)
        .uniform_sampler(4_usize.pow(2))
        .depth(10)
        // .random_sampler(1)
        .build();

    let mut world = Box::new(HittableList::default());

    // Ground
    world.add(Box::new(Sphere::new(
        Point3::new(0., -40_000_000.5, -1.),
        40_000_000.,
        Arc::new(Lambertian::from(Color::new(0.2, 0.5, 0.3))),
    )));

    // // Ballz
    world.add(Box::new(Sphere::new(
        Point3::new(0., 0., -2.),
        0.5,
        Arc::new(Lambertian::from(Color::new(0.7, 0.3, 0.3))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(1.8, 0., -3.7),
        0.5,
        Arc::new(Lambertian::from(Color::new(0.7, 0.3, 0.3))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-1.6, 0., -2.7),
        0.5,
        Arc::new(Metal),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-0.6, 1., -2.7),
        0.5,
        Arc::new(Metal),
    )));

    let world = world as Box<dyn Hittable>;

    camera.render(&world);

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("Done in {:.3} seconds", elapsed);
    Ok(())
}

fn value_to_color(value: f64) -> Color {
    Color::new(
        if value < 0.0 { value } else { 0. },
        if value > 0.0 { value.fract() } else { 0. },
        value.abs() / 10.,
    )
}
