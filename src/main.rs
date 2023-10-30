#![allow(unused)]
#![feature(test)]

use std::sync::mpsc::{Sender, SyncSender};
use std::sync::Arc;
use std::thread::Scope;
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
mod ui;
mod random;
mod range;
mod ray;
mod vec3;
mod scene;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use std::time::Duration;

fn main() {
    let camera = CameraBuilder::new()
        .aspect_ratio(16.0 / 9.0)
        .aspect_ratio((16.0 / 3.) / (9.0 / 2.))
        // .aspect_ratio(1.0)
        .field_of_view(55.0)
        .image_width(768)
        .image_width(1920)
        .image_width(3840 / 6)
        //.image_width(3840)
        .uniform_sampler(3_usize.pow(2))
        .max_ray_depth(10)
        //.random_sampler(9_usize.pow(2))
        .lookfrom(Point3::new(0.0, 0.5, 1.0) * 1.5)
        .lookat(Point3::new(0.0, 0.3, 0.0))
        .up_vector(Vec3::new(0.0, 1.0, 0.0))
        .defocus_angle(0.0)
        //.focus_distance(6.5)
        .build();

    std::thread::scope(|s| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(64);
        s.spawn(move || {
            ui::sdl_thread(camera.image_width, camera.image_height, receiver);
        });
        s.spawn(move || {
            render_thread(camera, sender);
        });
    });
}

fn render_thread(
    camera: Camera,
    sender: SyncSender<((usize, usize), (usize, usize), Vec<Color>)>,
) -> std::io::Result<()> {
    let start_time = Instant::now();

    let world = composition();
    // let camera = CameraBuilder::new()
    //     .aspect_ratio(16.0 / 9.0)
    //     // .aspect_ratio(1.0)
    //     .image_width(1200)
    //     .field_of_view(20.0)
    //     //.image_width(3840)
    //     .uniform_sampler(25_usize.pow(2))
    //     .max_ray_depth(50)
    //     //.random_sampler(4_usize.pow(2))
    //     .lookfrom(Point3::new(13.0, 2.0, 3.0))
    //     .lookat(Point3::new(0.0, 0.0, 0.0))
    //     .up_vector(Vec3::new(0.0, 1.0, 0.0))
    //     .defocus_angle(0.6)
    //     .focus_distance(10.0)
    //     .build();

    // let world = book_cover();

    let world = world as Box<dyn Hittable>;

    camera.render(&world, sender);

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("Done in {:.3} seconds", elapsed);
    Ok(())
}

fn book_cover() -> Box<HittableList> {
    let mut rng = Rng::from_seed([42, 1337]);
    let mut world = Box::new(HittableList::default());
    let ground_material = Arc::new(Lambertian::from(Color::new(0.5, 0.5, 0.5)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    for a in -0..7 {
        for b in -0..3 {
            let choose_mat = rng.next_f64();
            let center = Point3::new(
                a as f64 + 0.9 * rng.next_f64(),
                0.2,
                b as f64 + 0.9 * rng.next_f64(),
            );
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere_material: Arc<dyn Material> = if choose_mat < 0.7 {
                    // diffuse
                    let albedo = Color::random(&mut rng) * Color::random(&mut rng);
                    Arc::new(Lambertian::from(albedo))
                } else if choose_mat < 0.9 {
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

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Arc::new(Dielectric::new(1.5)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Arc::new(Lambertian::from(Color::new(0.4, 0.2, 0.1))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0)),
    )));
    return world;
}
fn ordered() -> Box<HittableList> {
    let mut world = Box::new(HittableList::default());
    let mat_ground = Arc::new(Lambertian::from(Color::new(0.8, 0.8, 0.0)));
    let mat_center = Arc::new(Lambertian::from(Color::new(0.1, 0.2, 0.5)));
    let mat_left = Arc::new(Dielectric::new(1.5));
    let mat_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -100.5, -1.),
        100.0,
        mat_ground,
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 0.0, -1.),
        0.5,
        mat_center,
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.),
        0.5,
        mat_left.clone(),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-1.0, 0.0, -1.),
        -0.4,
        mat_left.clone(),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(1.0, 0.0, -1.),
        0.5,
        mat_right,
    )));
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
        Point3::new(0., -40_000_000.5, 0.),
        40_000_000.,
        Arc::new(Lambertian::from(Color::new(0.05, 0.20, 0.07))),
    )));

    let blue_lamb = Arc::new(Lambertian::from(Color::new(0.1, 0.1, 0.8)));
    let red_lamb = Arc::new(Lambertian::from(Color::new(0.8, 0.1, 0.1)));

    // // Ballz
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
    return world;
}

extern crate test;

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use super::*;
    use crate::random::Rng;
    use test::Bencher;

    #[bench]
    fn bench_random_in_unit_sphere(b: &mut Bencher) {
        let mut rng = Rng::new();
        b.iter(|| {
            black_box(Vec3::random_in_unit_sphere(&mut rng));
        });
    }

    #[bench]
    fn bench_random_in_unit_sphere_reject(b: &mut Bencher) {
        let mut rng = Rng::new();
        b.iter(|| {
            black_box(Vec3::random_in_unit_sphere_reject(&mut rng));
        });
    }
}
