use std::sync::{mpsc::SyncSender, Arc};

use crate::{
    camera::{builder::CameraBuilder, Camera},
    color::Color,
    hittable::{
        materials::Dielectric, materials::Lambertian, materials::Material, materials::Metal,
        Hittable, HittableList, MovingSphere, Sphere,
    },
    random::Rng,
    vec3::{Point3, Vec3},
};

pub struct Scene<W> {
    pub camera: Camera,
    world: W,
}

impl Scene<Box<dyn Hittable>> {
    pub fn new(camera: Camera, world: Box<dyn Hittable>) -> Self {
        Self { camera, world }
    }
    pub fn render(&self, sender: SyncSender<((usize, usize), (usize, usize), Vec<Color>)>) {
        self.camera.render(&self.world, sender);
    }
}

pub fn composition(camera_builder: CameraBuilder) -> Scene<Box<dyn Hittable>> {
    let camera = camera_builder
        .field_of_view(55.0)
        .lookfrom(Point3::new(0.0, 0.5, 1.0) * 1.5)
        .lookat(Point3::new(0.0, 0.3, 0.0))
        .up_vector(Vec3::new(0.0, 1.0, 0.0))
        .defocus_angle(0.0)
        .build();
    let mut world = Box::new(HittableList::default());

    // Ground
    world.add(Box::new(Sphere::new(
        Point3::new(0., -40_000_000.5, 0.),
        40_000_000.,
        Arc::new(Lambertian::from(Color::new(0.05, 0.20, 0.07))),
    )));

    let blue_lamb = Arc::new(Lambertian::from(Color::new(0.1, 0.1, 0.8)));
    let red_lamb = Arc::new(Lambertian::from(Color::new(0.8, 0.1, 0.1)));

    // Ballz
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
    return Scene { camera, world };
}

pub fn book_cover(camera_builder: CameraBuilder) -> Scene<Box<dyn Hittable>> {
    let camera = camera_builder
        .field_of_view(20.0)
        .lookfrom(Point3::new(13.0, 2.0, 3.0))
        .lookat(Point3::new(0.0, 0.0, 0.0))
        .up_vector(Vec3::new(0.0, 1.0, 0.0))
        .defocus_angle(0.3)
        .focus_distance(10.0)
        .build();

    let mut rng = Rng::from_seed([42, 1337]);
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
                // world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                world.add(Box::new(MovingSphere::new(
                    Sphere::new(center, 0.2, sphere_material),
                    center + Point3::new(0.0, 0.5 * (1. - choose_mat), 0.0),
                )));
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
        Arc::new(Lambertian::from(Color::from_hex(0xffca3a))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Arc::new(Metal::new(Color::new(0.8, 0.8, 0.8), 0.0)),
    )));
    return Scene {
        camera,
        world: world.into_bvh(),
    };
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
