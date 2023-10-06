#![allow(unused)]
#![feature(float_next_up_down)]

use std::time::Instant;

use crate::camera::{Camera, CameraBuilder};
use crate::color::Color;
use crate::hittable::{HittableList, Sphere};
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

    let mut world = Box::new(HittableList::default());

    world.add(Box::new(Sphere::new(Point3::new(0., 0., -2.), 0.5)));
    //world.add(Box::new(Sphere::new(Point3::new(0., -100.5, -1.), 100.)));
    world.add(Box::new(Sphere::new(Point3::new(0., -3., -2.), 2.)));
    world.add(Box::new(Sphere::new(Point3::new(0., 3., -2.), 2.)));
    world.add(Box::new(Sphere::new(Point3::new(-3., 0., -2.), 2.)));
    world.add(Box::new(Sphere::new(Point3::new(3., 0., -2.), 2.)));

    let world = world as Box<dyn Hittable>;

    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 10.0)
        .aspect_ratio(1.0)
        .image_width(900)
        .uniform_sampler(25)
        //.random_sampler(25)
        .build();

    camera.render(world);

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
