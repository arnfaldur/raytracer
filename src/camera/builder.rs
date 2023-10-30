use super::Camera;
use super::PixelSampler;
use super::image::ImageSpec;
use crate::vec3::Point3;
use crate::vec3::Vec3;

#[derive(Default)]
pub struct CameraBuilder {
    pub image_spec: Option<ImageSpec>,

    pub pixel_sampler: Option<PixelSampler>,
    pub max_ray_depth: Option<usize>,

    pub field_of_view: Option<f64>,
    pub lookfrom: Option<Point3>,
    pub lookat: Option<Point3>,
    pub up_vector: Option<Vec3>,

    pub defocus_angle: Option<f64>,
    pub focus_distance: Option<f64>,
}

macro_rules! builder_field {
    ($field:ident, $type:ty) => {
        pub fn $field(self, $field: $type) -> Self {
            Self {
                $field: Some($field),
                ..self
            }
        }
    };
}

impl CameraBuilder {
    builder_field! {image_spec, ImageSpec}
    builder_field! {max_ray_depth, usize}
    builder_field! {field_of_view, f64}
    builder_field! {lookfrom, Point3}
    builder_field! {lookat, Point3}
    builder_field! {up_vector, Vec3}
    builder_field! {defocus_angle, f64}
    builder_field! {focus_distance, f64}
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

        let viewport_u = viewport_width * u;
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
