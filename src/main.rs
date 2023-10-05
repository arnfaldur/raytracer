#![allow(unused)]

use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

use hittable::Hittable;

use crate::color::Color;
use crate::hittable::{HittableList, Sphere};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

mod color;
mod hittable;
mod ray;
mod vec3;

const ASPECT_RATIO: f64 = 16.0 / 10.0;

const IMAGE_WIDTH: usize = 500;
const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
const PIXEL_COUNT: usize = IMAGE_WIDTH * IMAGE_HEIGHT;
const ACTUAL_RATIO: f64 = IMAGE_WIDTH as f64 / IMAGE_HEIGHT as f64;

fn main() -> std::io::Result<()> {
    let start_time = Instant::now();

    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * ACTUAL_RATIO;
    let camera_center = Point3::new(0., 0., 0.);

    let viewport_u = Vec3::new(viewport_width, 0., 0.);
    let viewport_v = Vec3::new(0., -viewport_height, 0.);

    let pixel_delta_u = viewport_u / IMAGE_WIDTH as f64;
    let pixel_delta_v = viewport_v / IMAGE_HEIGHT as f64;

    let viewport_upper_left =
        camera_center - Vec3::new(0., 0., focal_length) - viewport_u / 2. - viewport_v / 2.;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    let mut world = Box::new(HittableList::default());

    world.add(Box::new(Sphere::new(Point3::new(0.,0.,-1.), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0., -100.5, -1.), 100.)));

    let world = world as Box<dyn Hittable>;

    let mut image_buffer = Vec::with_capacity(PIXEL_COUNT);

    let mut dragger = 0;
    for j in 0..IMAGE_HEIGHT {
        let nth_pixel = j * IMAGE_WIDTH;
        let boi = start_time.elapsed().as_millis();
        if boi - dragger > 5 {
            let progress = nth_pixel as f64 / (PIXEL_COUNT - 1) as f64;
            dragger = boi;
            println!("{:.2}%", progress * 100.0);
        }
        for i in 0..IMAGE_WIDTH {
            let pixel_center =
                pixel00_loc + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_direction);
            let color = ray_color(&ray, &world);

            image_buffer.push(color);
        }
    }

    write_buffer_to_file(&image_buffer)?;
    let elapsed = start_time.elapsed().as_secs_f64();
    println!("Done in {:.3} seconds", elapsed);
    Ok(())
}

fn write_buffer_to_file(image_buffer: &Vec<Color>) -> std::io::Result<()> {
    let file = File::create("image.ppm")?;
    let mut file_writer = BufWriter::new(file);
    file_writer.write_all(format!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT).as_bytes())?;
    for color in image_buffer.iter() {
        color.write_to_writer(&mut file_writer)?;
    }
    file_writer.flush()?;
    Ok(())
}

fn ray_color(ray: &Ray, world: &Box<dyn Hittable>) -> Color {
    let center = Point3::new(0., 0., -1.);
    if let Some(hit_record) = world.hit(ray, 0.0..f64::INFINITY) {
        return Color::from(hit_record.normal + 1.) / 2.;
    }
    let unit_direction = ray.direction.unit_vector();
    let a = 0.5 * (unit_direction.y + 1.0);
    (1.0 - a) * Color::new(1., 1., 1.) + a * Color::new(0.5, 0.7, 1.)
}

fn value_to_color(value: f64) -> Color {
    Color::new(
        if value < 0.0 { value } else { 0. },
        if value > 0.0 { value.fract() } else { 0. },
        value.abs() / 10.,
    )
}
