use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, sync_channel, Sender, SyncSender};
use std::sync::Arc;
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

#[derive(Default, Debug)]
pub struct ImageSpecBuilder {
    width: Option<usize>,
    height: Option<usize>,
    aspect_ratio: Option<f64>,
}

impl ImageSpecBuilder {
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }
    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }
    pub fn aspect_ratio(mut self, aspect_ratio: f64) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }
    pub fn build(self) -> ImageSpec {
        match self {
            ImageSpecBuilder {
                width: Some(width),
                height: Some(height),
                aspect_ratio: None,
            } => ImageSpec {
                width,
                height,
                aspect_ratio: width as f64 / height as f64,
            },
            ImageSpecBuilder {
                width: Some(width),
                height: None,
                aspect_ratio: Some(aspect_ratio),
            } => ImageSpec {
                width,
                height: ((width as f64 / aspect_ratio) as usize).max(1),
                aspect_ratio,
            },
            ImageSpecBuilder {
                width: None,
                height: Some(height),
                aspect_ratio: Some(aspect_ratio),
            } => ImageSpec {
                width: ((aspect_ratio / height as f64) as usize).max(1),
                height,
                aspect_ratio,
            },
            _ => panic!("image spec must have exactly one missing field {:?}", self),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageSpec {
    pub width: usize,
    pub height: usize,
    pub aspect_ratio: f64,
}

#[derive(Default)]
pub struct CameraBuilder {
    image_spec: Option<ImageSpec>,

    pixel_sampler: Option<PixelSampler>,
    max_ray_depth: Option<usize>,

    field_of_view: Option<f64>,
    lookfrom: Option<Point3>,
    lookat: Option<Point3>,
    up_vector: Option<Vec3>,

    defocus_angle: Option<f64>,
    focus_distance: Option<f64>,
}

macro_rules! builder_field_mut {
    ($field:ident, $type:ty) =>
        (pub fn $field(mut self, $field: $type) -> Self {
            self.$field = Some($field);
            self
        })
}
macro_rules! builder_field {
    ($field:ident, $type:ty) =>
        (pub fn $field(mut self, $field: $type) -> Self {
            Self {
                $field: Some($field),
                ..self
            }
        })
}

impl CameraBuilder {
    builder_field!{image_spec, ImageSpec}
    builder_field!{max_ray_depth, usize}
    builder_field!{field_of_view, f64}
    builder_field!{lookfrom, Point3}
    builder_field!{lookat, Point3}
    builder_field!{up_vector, Vec3}
    builder_field!{defocus_angle, f64}
    builder_field!{focus_distance, f64}
    pub fn uniform_sampler(self, samples_per_pixel: usize) -> Self {
        Self {
            pixel_sampler: Some(PixelSampler::Uniform(samples_per_pixel)),
            ..self
        }
    }
    pub fn random_sampler(mut self, samples_per_pixel: usize) -> Self {
        Self {
            pixel_sampler: Some(PixelSampler::Random(samples_per_pixel)),
            ..self
        }
    }
    pub fn build(self) -> Camera {
        let image_spec = self
            .image_spec
            .expect("The image specifications must be set");
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
        let depth = self.max_ray_depth.expect("The depth must be set");

        let field_of_view = self.field_of_view.unwrap_or(90.0);
        let lookfrom = self.lookfrom.unwrap_or(Point3::new(0., 0., 0.));
        let lookat = self.lookat.unwrap_or(Point3::new(0., 0., -1.));
        let up_vector = self.up_vector.unwrap_or(Vec3::new(0., 1., 0.));

        let defocus_angle = self.defocus_angle.unwrap_or(0.0);
        let focus_distance = self.focus_distance.unwrap_or(lookfrom.distance(&lookat));

        // Actual initialization

        let center = lookfrom;

        //let focal_length = (lookfrom - lookat).length();
        let theta = field_of_view.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * focus_distance;
        let viewport_width = viewport_height * image_spec.width as f64 / image_spec.height as f64;

        let w = (lookfrom - lookat).unit_vector();
        let u = up_vector.cross(&w).unit_vector();
        let v = w.cross(&u);

        let viewport_u = (viewport_width * u);
        let viewport_v = viewport_height * -v;

        let pixel_delta_u = viewport_u / image_spec.width as f64;
        let pixel_delta_v = viewport_v / image_spec.height as f64;

        let viewport_upper_left = center - (focus_distance * w) - viewport_u / 2. - viewport_v / 2.;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius = focus_distance * (defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = defocus_radius * u;
        let defocus_disk_v = defocus_radius * v;
        Camera {
            aspect_ratio: image_spec.aspect_ratio,
            image_width: image_spec.width,
            pixel_sampler,
            depth,

            field_of_view,
            lookfrom,
            lookat,
            up_vector,

            defocus_angle,
            focus_distance,

            image_height: image_spec.height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            w,
            u,
            v,

            defocus_disk_u,
            defocus_disk_v,
        }
    }
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
        let progress_interval = 1000;
        let start_time = Instant::now();
        let pixel_count = self.image_width * self.image_height;
        let mut image_buffer = vec![Color::black(); pixel_count];

        let mut rng = Rng::from_seed([123, 128]);
        let mut rng = rng.short_jump();

        let mut threshold = 0;

        let threads: usize = thread::available_parallelism().unwrap().into();
        let rect = (64, 64);
        let rect = (32, 32);
        let rect_count = self.image_height.div_ceil(rect.0) * self.image_width.div_ceil(rect.1);
        thread::scope(|s| {
            let get_parameters = |index: usize| {
                let by = self.image_height.div_ceil(rect.0);
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
            for thread in 0..threads {
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
        mut rng: Rng,
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

    fn render_scanlines(
        &self,
        mut rng: Rng,
        j_offset: usize,
        scanlines: usize,
        world: &Box<dyn Hittable>,
        image_buffer: &mut [Color],
    ) {
        for j in 0..scanlines {
            for i in 0..self.image_width {
                let mut rng = rng.clone();
                let color = self.sample_pixel(&mut rng, j_offset + j, i, world);

                let gamma_corrected = color.gamma_corrected(2.2);

                let index = (j * self.image_width) + i;
                image_buffer[index] = gamma_corrected;
            }
        }
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
