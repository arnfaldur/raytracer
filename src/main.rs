#![allow(unused)]
#![feature(test)]

use std::sync::mpsc::{SyncSender};
use std::sync::Arc;
use std::time::Instant;

use camera::{builder::CameraBuilder, image::ImageSpecBuilder};
use color::Color;
use hittable::{Dielectric, Hittable, HittableList, Lambertian, Material, Metal, Sphere};
use random::Rng;
use scene::{composition, Scene, book_cover};
use vec3::{Point3, Vec3};

mod camera;
mod color;
mod hittable;
mod random;
mod range;
mod ray;
mod scene;
mod ui;
mod vec3;

fn main() {
    let image_spec = ImageSpecBuilder::default()
        .width(3840 / 3)
        .aspect_ratio((16.0 / 3.) / (9.0 / 2.))
        .build();

    let camera = CameraBuilder::default()
        .image_spec(image_spec.clone())
        .uniform_sampler(9_usize.pow(2))
        .max_ray_depth(10)
        //.random_sampler(9_usize.pow(2))
        .defocus_angle(0.0)
        //.focus_distance(6.5)
     ;

    std::thread::scope(|s| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(64);
        s.spawn(move || {
            ui::sdl_thread(image_spec.width, image_spec.height, receiver);
        });
        s.spawn(move || {
            let scene = book_cover(camera);
            render_thread(scene, sender);
        });
    });
}

fn render_thread(
    scene: Scene<Box<dyn Hittable>>,
    // camera: Camera,
    sender: SyncSender<((usize, usize), (usize, usize), Vec<Color>)>,
) {
    let start_time = Instant::now();

    scene.render(sender);

    let elapsed = start_time.elapsed().as_secs_f64();
    println!("Done in {:.3} seconds", elapsed);
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
