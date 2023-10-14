#![allow(unused)]
#![feature(float_next_up_down)]

use std::sync::Arc;
use std::time::Instant;

use crate::camera::{Camera, CameraBuilder};
use crate::color::Color;
use crate::hittable::{Dielectric, HittableList, Lambertian, Metal, Sphere};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};
use hittable::{Hittable, Material};
use random::Rng;

mod camera;
mod color;
mod hittable;
mod random;
mod range;
mod ray;
mod vec3;

fn main() -> std::io::Result<()> {
    let start_time = Instant::now();

    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        // .aspect_ratio(1.0)
        .image_width(900)
        .field_of_view(30.0)
        //.image_width(3840)
        .uniform_sampler(8_usize.pow(2))
        .max_ray_depth(20)
        //.random_sampler(4_usize.pow(2))
        .lookfrom(Point3::new(-2.0, 2.0, 1.0))
        .lookat(Point3::new(0.0, 0.0, -1.0))
        .up_vector(Vec3::new(0.0, 1.0, 0.0))
        .defocus_angle(10.0)
        .focus_distance(3.4)

        .build();

    let world = ordered();

    let world = world as Box<dyn Hittable>;

    camera.render(&world);

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("Done in {:.3} seconds", elapsed);
    Ok(())
}

fn book_cover() -> Box<HittableList> {
    let mut rng = Rng::from_seed([1,2]);
    let mut world = Box::new(HittableList::default());
    let ground_material = Arc::new(Lambertian::from(Color::new(0.5, 0.5, 0.5)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rng.next_f64();
            let center = Point3::new(
                a as f64 + 0.9 * rng.next_f64(),
                0.2,
                b as f64 + 0.9 * rng.next_f64(),
            );
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere_material: Arc<dyn Material> = if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random(&mut rng) * Color::random(&mut rng);
                    Arc::new(Lambertian::from(albedo))
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random(&mut rng) / 2.0 + 0.5;
                    let fuzz = rng.next_f64_range(0.0..0.5);
                    Arc::new(Metal::new(albedo, fuzz))
                } else {
                    // glass
                    Arc::new(Dielectric::new(1.5))
                };
                world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
            }
        }
    }
    return world;
}

fn ordered() -> Box<HittableList> {
    let mut world = Box::new(HittableList::default());
    let mat_ground = Arc::new(Lambertian::from(Color::new(0.8, 0.8, 0.0)));
    let mat_center = Arc::new(Lambertian::from(Color::new(0.1, 0.2, 0.5)));
    let mat_left = Arc::new(Dielectric::new(1.5));
    let mat_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.), 100.0, mat_ground)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.), 0.5, mat_center)));
    world.add(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.), 0.5, mat_left.clone())));
    world.add(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.), -0.4, mat_left.clone())));
    world.add(Box::new(Sphere::new(Point3::new(1.0, 0.0, -1.), 0.5, mat_right)));
    return world;
}

fn fov_test() -> Box<HittableList> {
    let mut world = Box::new(HittableList::default());
    let r = (std::f64::consts::PI / 4.0).cos();
    world.add(Box::new(Sphere::new(
        Point3::new(r, 0., -1.),
        r,
        Arc::new(Lambertian::from(Color::new(1.0, 0.0, 0.0))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-r, 0., -1.),
        r,
        Arc::new(Lambertian::from(Color::new(0.0, 0.0, 1.0))),
    )));
    return world;
}
fn composition() -> Box<HittableList> {
    let mut world = Box::new(HittableList::default());

    // Ground
    world.add(Box::new(Sphere::new(
        Point3::new(0., -40_000_000.5, -1.),
        40_000_000.,
        Arc::new(Lambertian::from(Color::new(0.05, 0.20, 0.07))),
    )));

    // // Ballz
    world.add(Box::new(Sphere::new(
        Point3::new(0., 0., -2.),
        0.5,
        Arc::new(Lambertian::from(Color::new(0.8, 0.1, 0.1))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(1.8, 0., -3.7),
        0.5,
        Arc::new(Lambertian::from(Color::new(0.1, 0.1, 0.8))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-0.25 - 0.125, -0.25, -1.5),
        0.25,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-0.25 - 0.125, -0.25, -1.5),
        -0.1,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-0.6, 1., -2.7),
        0.5,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-1.0, 0., -2.0),
        0.5,
        Arc::new(Metal::new(Color::gray(0.7), 0.3)),
    )));
    return world;
}

fn value_to_color(value: f64) -> Color {
    Color::new(
        if value < 0.0 { value } else { 0. },
        if value > 0.0 { value.fract() } else { 0. },
        value.abs() / 10.,
    )
}
