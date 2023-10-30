use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;
use std::time::{Instant};
use std::{thread};

use crate::random::Rng;
use crate::{
    color::Color,
    hittable::Hittable,
    ray::Ray,
    vec3::{Point3, Vec3},
};

pub mod builder;
pub mod image;

pub enum PixelSampler {
    Uniform(usize),
    Random(usize),
}

pub struct Camera {
    aspect_ratio: f64,
    pub image_width: usize,
    pixel_sampler: PixelSampler,
    depth: usize,

    field_of_view: f64,
    lookfrom: Point3,
    lookat: Point3,
    up_vector: Vec3,

    defocus_angle: f64,
    focus_distance: f64,

    pub image_height: usize,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,

    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn render(
        &self,
        world: &Box<dyn Hittable>,
        sender: SyncSender<((usize, usize), (usize, usize), Vec<Color>)>,
    ) {
        let start_time = Instant::now();
        let pixel_count = self.image_width * self.image_height;
        let mut image_buffer = vec![Color::black(); pixel_count];

        let mut rng = Rng::from_seed([123, 128]);
        let mut rng = rng.short_jump();

        let threads = usize::from(thread::available_parallelism().unwrap());
        let rect = (32, 32);
        let rect_count = self.image_height.div_ceil(rect.0) * self.image_width.div_ceil(rect.1);
        thread::scope(|s| {
            let get_parameters = |index: usize| {
                let bx = self.image_width.div_ceil(rect.1);
                let rect_y = index / bx;
                let rect_x = index % bx;
                let top_left = (rect_y * rect.0, rect_x * rect.1);
                let rect = (
                    rect.0.min(self.image_height - top_left.0),
                    rect.1.min(self.image_width - top_left.1),
                );

                return (rng.clone(), top_left, rect, world);
            };

            let shared_index = Arc::new(AtomicUsize::new(0));

            let (worker_sender, delegator_receiver) = sync_channel(1);
            for _thread in 0..threads {
                let shared_index = shared_index.clone();
                let worker_sender = worker_sender.clone();
                s.spawn(move || {
                    loop {
                        let idx = shared_index.fetch_add(1, Ordering::SeqCst);
                        if idx >= rect_count {
                            break;
                        }
                        let (rng, top_left, rect, world) = get_parameters(idx);
                        let result = self.render_rect(rng, top_left, rect, world);
                        if let Err(_) = worker_sender.send((top_left, rect, result)) {
                            break;
                        }
                    }
                    drop(worker_sender);
                });
            }

            for i in 0..rect_count {
                let (top_left, rect, result) = delegator_receiver.recv().unwrap();
                for dy in 0..rect.0 {
                    for dx in 0..rect.1 {
                        let index = ((top_left.0 + dy) * self.image_width) + (top_left.1 + dx);
                        image_buffer[index] = result[(dy * rect.1) + dx];
                    }
                }
                if let Err(_) = sender.send((top_left, rect, result)) {
                    println!("cancelled");
                    return;
                }
            }
            self.write_buffer_to_file(&image_buffer).unwrap();
        });
    }

    fn render_rect(
        &self,
        rng: Rng,
        top_left: (usize, usize),
        rect: (usize, usize),
        world: &Box<dyn Hittable>,
    ) -> Vec<Color> {
        let (height, width) = rect;
        let mut result = vec![Color::black(); rect.0 * rect.1];
        for j in 0..height {
            for i in 0..width {
                let mut rng = rng.clone();
                let color = self.sample_pixel(&mut rng, top_left.0 + j, top_left.1 + i, world);

                let gamma_corrected = color.gamma_corrected(2.2);

                let index = (j * width) + i;
                result[index] = gamma_corrected;
            }
        }
        return result;
    }

    fn sample_pixel(&self, rng: &mut Rng, j: usize, i: usize, world: &Box<dyn Hittable>) -> Color {
        let mut accumulator = Color::black();

        match self.pixel_sampler {
            PixelSampler::Uniform(samples_sqrt) => {
                // let mut rng = rng.clone();
                for yi in 0..samples_sqrt {
                    for xi in 0..samples_sqrt {
                        let subpixel_interval = 1.0 / samples_sqrt as f64;
                        let subpixel_offset = subpixel_interval / 2.0 + 0.5;

                        let dy = j as f64 + yi as f64 * subpixel_interval - subpixel_offset;
                        let dx = i as f64 + xi as f64 * subpixel_interval - subpixel_offset;

                        // rng.short_jump();
                        // let mut rng = rng.clone();
                        accumulator += self.sample_at(rng, dx, dy, world);
                    }
                }
                accumulator / samples_sqrt.pow(2) as f64
            }
            PixelSampler::Random(samples) => {
                //let mut rng = rng.clone();
                for _ in 0..samples {
                    //rng.short_jump();
                    //let mut rng = rng.clone();
                    let dy = j as f64 + rng.next_f64_range(-0.5..0.5);
                    let dx = i as f64 + rng.next_f64_range(-0.5..0.5);

                    accumulator += self.sample_at(rng, dx, dy, world);
                }
                accumulator / samples as f64
            }
        }
    }

    fn sample_at(&self, rng: &mut Rng, dx: f64, dy: f64, world: &Box<dyn Hittable>) -> Color {
        let pixel_center = self.pixel00_loc + (dx * self.pixel_delta_u) + (dy * self.pixel_delta_v);
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample(rng)
        };
        let ray_direction = pixel_center - ray_origin;
        let ray = Ray::new(ray_origin, ray_direction);
        self.ray_color(rng, &ray, world)
    }
    fn ray_color(&self, rng: &mut Rng, ray: &Ray, world: &Box<dyn Hittable>) -> Color {
        fn ray_color_inner(
            rng: &mut Rng,
            depth: usize,
            limit: usize,
            ray: &Ray,
            world: &Box<dyn Hittable>,
        ) -> Color {
            if depth >= limit {
                return Color::black();
            }
            if let Some(hit_record) = world.hit(ray, 0.000001..f64::INFINITY) {
                if let Some((attenuation, scattered)) =
                    hit_record.material.scatter(rng, ray, &hit_record)
                {
                    return attenuation * ray_color_inner(rng, depth + 1, limit, &scattered, world);
                }
            }
            let unit_direction = ray.direction.unit_vector();
            let a = 0.5 * (unit_direction.y + 1.0);
            return (1. - a) * Color::new(1., 1., 1.) + a * Color::new(0.5, 0.7, 1.);
        }
        return ray_color_inner(rng, 0, self.depth, ray, world);
    }
    fn defocus_disk_sample(&self, rng: &mut Rng) -> Vec3 {
        let random = Vec3::random_in_unit_circle(rng);
        self.center + self.defocus_disk_u * random.x + self.defocus_disk_v * random.y
    }
    // I would prefer this not be a method of the camera class but it's own thing
    fn write_buffer_to_file(&self, image_buffer: &Vec<Color>) -> std::io::Result<()> {
        let file = File::create("image.ppm")?;
        let mut file_writer = BufWriter::new(file);
        file_writer.write_all(
            format!("P3\n{} {}\n255\n", self.image_width, self.image_height).as_bytes(),
        )?;
        for color in image_buffer.iter() {
            color.write_to_writer(&mut file_writer)?;
        }
        file_writer.flush()?;
        Ok(())
    }
}
