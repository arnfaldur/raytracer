use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::{Instant, UNIX_EPOCH};
use std::{default, thread, time};

use crate::random::Rng;
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
    depth: Option<usize>,
    field_of_view: Option<f64>,
    lookfrom: Option<Point3>,
    lookat: Option<Point3>,
    up_vector: Option<Vec3>,
}

impl CameraBuilder {
    pub fn new() -> Self {
        Self {
            aspect_ratio: None,
            image_width: None,
            pixel_sampler: None,
            depth: None,
            field_of_view: None,
            lookfrom: None,
            lookat: None,
            up_vector: None,
        }
    }
    pub fn aspect_ratio(mut self, aspect_ratio: f64) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }
    pub fn image_width(mut self, image_width: usize) -> Self {
        self.image_width = Some(image_width);
        self
    }
    pub fn uniform_sampler(mut self, samples_per_pixel: usize) -> Self {
        self.pixel_sampler = Some(PixelSampler::Uniform(samples_per_pixel));
        self
    }
    pub fn random_sampler(mut self, samples_per_pixel: usize) -> Self {
        self.pixel_sampler = Some(PixelSampler::Random(samples_per_pixel));
        self
    }
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }
    pub fn field_of_view(mut self, field_of_view: f64) -> Self {
        self.field_of_view = Some(field_of_view);
        self
    }
    pub fn lookfrom(mut self, lookfrom: Point3) -> Self {
        self.lookfrom = Some(lookfrom);
        self
    }
    pub fn lookat(mut self, lookat: Point3) -> Self {
        self.lookat = Some(lookat);
        self
    }
    pub fn up_vector(mut self, up_vector: Vec3) -> Self {
        self.up_vector = Some(up_vector);
        self
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
        let depth = self.depth.expect("The depth must be set");

        let field_of_view = self.field_of_view.unwrap_or(90.0);
        let lookfrom = self.lookfrom.unwrap_or(Point3::new(0., 0., 0.));
        let lookat = self.lookat.unwrap_or(Point3::new(0., 0., -1.));
        let up_vector = self.up_vector.unwrap_or(Vec3::new(0., 1., 0.));

        let image_height = ((image_width as f64 / aspect_ratio) as usize).max(1);

        let center = lookfrom;

        let focal_length = (lookfrom - lookat).length();
        let theta = field_of_view.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focal_length;
        let viewport_width = viewport_height * image_width as f64 / image_height as f64;

        let w = (lookfrom - lookat).unit_vector();
        let u = up_vector.cross(&w).unit_vector();
        let v = w.cross(&u);

        let viewport_u = (viewport_width * u);
        let viewport_v = viewport_height * -v;

        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left = center - (focal_length * w) - viewport_u / 2. - viewport_v / 2.;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);
        Camera {
            aspect_ratio,
            image_width,
            pixel_sampler,
            depth,

            field_of_view,
            lookfrom,
            lookat,
            up_vector,

            image_height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            w,
            u,
            v,
        }
    }
}
pub struct Camera {
    aspect_ratio: f64,
    image_width: usize,
    pixel_sampler: PixelSampler,
    depth: usize,

    field_of_view: f64,
    lookfrom: Point3,
    lookat: Point3,
    up_vector: Vec3,

    image_height: usize,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Camera {
    pub fn render(&self, world: &Box<dyn Hittable>) {
        let progress_interval = 1000;
        let start_time = Instant::now();
        let pixel_count = self.image_width * self.image_height;
        let mut image_buffer = vec![Color::black(); pixel_count];

        let mut rng = Rng::from_seed([123, 123]);

        let mut threshold = 0;
        // for (j, chunk) in image_buffer.chunks_mut(self.image_width).enumerate() {
        //     let nth_pixel = j * self.image_width;
        //     let elapsed = start_time.elapsed().as_millis();
        //     if elapsed - threshold > progress_interval {
        //         let progress = nth_pixel as f64 / (pixel_count - 1) as f64;
        //         threshold = elapsed;
        //         println!("{:.2}%", progress * 100.0);
        //     }
        //     self.render_scanline(rng.short_jump().clone(), j, world, chunk);
        // }

        thread::scope(|s| {
            for (j, chunk) in image_buffer.chunks_mut(self.image_width).enumerate() {
                let mut rng = Rng::from_seed([
                    j as u64,
                    time::SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64,
                ]);
                // }
                // for j in 0..self.image_height {
                // let nth_pixel = j * self.image_width;
                // let elapsed = start_time.elapsed().as_millis();
                // if elapsed - threshold > progress_interval {
                //     let progress = nth_pixel as f64 / (pixel_count - 1) as f64;
                //     threshold = elapsed;
                //     println!("{:.2}%", progress * 100.0);
                // }

                s.spawn(move || self.render_scanline(rng, j, world, chunk));
            }
        });
        self.write_buffer_to_file(&image_buffer).unwrap();
    }

    fn render_scanline(
        &self,
        mut rng: Rng,
        j: usize,
        world: &Box<dyn Hittable>,
        image_buffer: &mut [Color],
    ) {
        for i in 0..self.image_width {
            let color = self.sample_pixel(&mut rng, j, i, world);

            let gamma_corrected = color.gamma_corrected(2.2);

            let index = (j * self.image_width) + i;
            image_buffer[i] = gamma_corrected;
        }
    }
    fn sample_pixel(&self, rng: &mut Rng, j: usize, i: usize, world: &Box<dyn Hittable>) -> Color {
        let mut accumulator = Color::black();

        match self.pixel_sampler {
            PixelSampler::Uniform(samples_sqrt) => {
                for yi in 0..samples_sqrt {
                    for xi in 0..samples_sqrt {
                        let subpixel_interval = 1.0 / samples_sqrt as f64;
                        let subpixel_offset = subpixel_interval / 2.0 + 0.5;

                        let dy = j as f64 + yi as f64 * subpixel_interval - subpixel_offset;
                        let dx = i as f64 + xi as f64 * subpixel_interval - subpixel_offset;

                        accumulator += self.sample_at(rng, dx, dy, world);
                    }
                }
                accumulator / samples_sqrt.pow(2) as f64
            }
            PixelSampler::Random(samples) => {
                for _ in 0..samples {
                    let dy = j as f64 + rng.next_f64() - 0.5;
                    let dx = i as f64 + rng.next_f64() - 0.5;

                    accumulator += self.sample_at(rng, dx, dy, world);
                }
                accumulator / samples as f64
            }
        }
    }

    fn sample_at(&self, rng: &mut Rng, dx: f64, dy: f64, world: &Box<dyn Hittable>) -> Color {
        let pixel_center = self.pixel00_loc + (dx * self.pixel_delta_u) + (dy * self.pixel_delta_v);
        let ray_direction = pixel_center - self.center;
        let ray = Ray::new(self.center, ray_direction);
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
