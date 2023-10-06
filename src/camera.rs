use std::default;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

use crate::{
    color::Color,
    hittable::Hittable,
    ray::Ray,
    vec3::{Point3, Vec3},
};

enum PixelSampler {
    Uniform(usize),
    Random(usize),
}

pub struct CameraBuilder {
    aspect_ratio: Option<f64>,
    image_width: Option<usize>,
    pixel_sampler: Option<PixelSampler>,
}

impl CameraBuilder {
    pub fn new() -> Self {
        Self {
            aspect_ratio: None,
            image_width: None,
            pixel_sampler: None,
        }
    }
    pub fn aspect_ratio(self, aspect_ratio: f64) -> Self {
        Self {
            aspect_ratio: Some(aspect_ratio),
            ..self
        }
    }
    pub fn image_width(self, image_width: usize) -> Self {
        Self {
            image_width: Some(image_width),
            ..self
        }
    }
    pub fn uniform_sampler(self, samples_per_pixel: usize) -> Self {
        Self {
            pixel_sampler: Some(PixelSampler::Uniform(samples_per_pixel)),
            ..self
        }
    }
    pub fn random_sampler(self, samples_per_pixel: usize) -> Self {
        Self {
            pixel_sampler: Some(PixelSampler::Random(samples_per_pixel)),
            ..self
        }
    }
    pub fn build(self) -> Camera {
        let aspect_ratio = self.aspect_ratio.expect("The aspect ratio must be set");
        let image_width = self.image_width.expect("The image width must be set");
        let pixel_sampler = match self
            .pixel_sampler
            .expect("The samples per pixel must be set")
        {
            PixelSampler::Uniform(samples_per_pixel) => {
                let samples_sqrt = (samples_per_pixel as f64).sqrt();
                if samples_sqrt.fract() != 0.0 {
                    panic!("samples_per_pixel in the grid sampler must be a square number, current value: {}", samples_per_pixel);
                }
                PixelSampler::Uniform(samples_sqrt as usize)
            }
            PixelSampler::Random(samples_per_pixel) => PixelSampler::Random(samples_per_pixel),
        };

        let image_height = ((image_width as f64 / aspect_ratio) as usize).max(1);

        let center = Point3::zero();

        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * image_width as f64 / image_height as f64;

        let viewport_u = Vec3::new(viewport_width, 0., 0.);
        let viewport_v = Vec3::new(0., -viewport_height, 0.);

        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left =
            center - Vec3::new(0., 0., focal_length) - viewport_u / 2. - viewport_v / 2.;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);
        Camera {
            aspect_ratio,
            image_width,
            pixel_sampler,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
        }
    }
}
pub struct Camera {
    aspect_ratio: f64,
    image_width: usize,
    pixel_sampler: PixelSampler,

    image_height: usize,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    pub fn render(&self, world: Box<dyn Hittable>) {
        let progress_interval = 1000;
        let start_time = Instant::now();
        let pixel_count = self.image_width * self.image_height;
        let mut image_buffer = Vec::with_capacity(pixel_count);

        let mut threshold = 0;
        for j in 0..self.image_height {
            let nth_pixel = j * self.image_width;
            let elapsed = start_time.elapsed().as_millis();
            if elapsed - threshold > progress_interval {
                let progress = nth_pixel as f64 / (pixel_count - 1) as f64;
                threshold = elapsed;
                println!("{:.2}%", progress * 100.0);
            }
            for i in 0..self.image_width {
                let color = match self.pixel_sampler {
                    PixelSampler::Uniform(samples) => self.sample_uniformly(i, j, samples, &world),
                    PixelSampler::Random(samples) => self.sample_randomly(i, j, samples, &world),
                };

                image_buffer.push(color);
            }
        }
        self.write_buffer_to_file(&image_buffer).unwrap();
    }

    fn sample_uniformly(
        &self,
        i: usize,
        j: usize,
        samples: usize,
        world: &Box<dyn Hittable>,
    ) -> Color {
        let mut accumulator = Color::black();

        for yi in 0..samples {
            for xi in 0..samples {
                let subpixel_interval = 1.0 / samples as f64;
                let subpixel_offset = subpixel_interval / 2.0 + 0.5;

                let dy = yi as f64 * subpixel_interval - subpixel_offset;
                let dx = xi as f64 * subpixel_interval - subpixel_offset;

                let pixel_center = self.pixel00_loc
                    + ((i as f64 + dy) * self.pixel_delta_u)
                    + ((j as f64 + dx) * self.pixel_delta_v);
                let ray_direction = pixel_center - self.center;
                let ray = Ray::new(self.center, ray_direction);
                let result = self.ray_color(&ray, world);
                accumulator += result;
            }
        }
        return accumulator / samples.pow(2) as f64;
    }
    fn sample_randomly(
        &self,
        i: usize,
        j: usize,
        samples: usize,
        world: &Box<dyn Hittable>,
    ) -> Color {
        let mut accumulator = Color::black();

        for _ in 0..samples {
            let dy = fastrand::f64() - 0.5;
            let dx = fastrand::f64() - 0.5;

            let pixel_center = self.pixel00_loc
                + ((i as f64 + dy) * self.pixel_delta_u)
                + ((j as f64 + dx) * self.pixel_delta_v);
            let ray_direction = pixel_center - self.center;
            let ray = Ray::new(self.center, ray_direction);
            let result = self.ray_color(&ray, world);
            accumulator += result;
        }
        return accumulator / samples as f64;
    }
    fn ray_color(&self, ray: &Ray, world: &Box<dyn Hittable>) -> Color {
        if let Some(hit_record) = world.hit(ray, 0.0..f64::INFINITY) {
            return Color::from(hit_record.normal + 1.) / 2.;
        }
        let unit_direction = ray.direction.unit_vector();
        let a = 0.5 * (unit_direction.y + 1.0);
        return (1. - a) * Color::new(1., 1., 1.) + a * Color::new(0.5, 0.7, 1.);
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
