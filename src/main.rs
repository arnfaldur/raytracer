#![allow(unused)]
#![feature(test)]

use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::time::Instant;

use camera::{builder::CameraBuilder, image::ImageSpecBuilder};
use color::Color;
use hittable::Hittable;
use random::Rng;
use scene::{book_cover, composition, Scene};
use vec3::{Point3, Vec3};

use crate::scene::{two_spheres, earth, something_blocky};

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
        //.width(3840 / 2)
        .aspect_ratio((16.0 / 3.) / (9.0 / 2.))
        .aspect_ratio(16.0 / 9.0)
        .build();

    let camera = CameraBuilder::default()
        .image_spec(image_spec.clone())
        .uniform_sampler(10_usize.pow(2))
        .uniform_sampler(6_usize.pow(2))
        .max_ray_depth(16)
        //.random_sampler(9_usize.pow(2))
        .defocus_angle(0.2)
        //.focus_distance(10.0)
     ;

    std::thread::scope(|s| {
        let (sender, receiver) = std::sync::mpsc::sync_channel(64);
        s.spawn(move || {
            ui::sdl_thread(image_spec.width, image_spec.height, receiver);
        });
        s.spawn(move || {
            let start_time = Instant::now();

            // slet scene = two_spheres(camera);
            let scene = something_blocky(camera);
            render_thread(scene, sender);
            let elapsed = start_time.elapsed().as_secs_f64();
            println!("Done in {:.3} seconds", elapsed);
        });
    });
}

fn render_thread(
    scene: Scene<Box<dyn Hittable>>,
    sender: SyncSender<((usize, usize), (usize, usize), Vec<Color>)>,
) {
    scene.render(sender);
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
