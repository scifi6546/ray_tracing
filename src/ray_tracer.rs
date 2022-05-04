use super::{vec_near_zero, Image, RgbColor, RgbImage};

use crate::reflect;
use cgmath::{InnerSpace, Point3, Vector3};
use miniquad::rand;
use rand::prelude::*;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

trait Material {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray)>;
}
struct Lambertian {
    albedo: RgbColor,
}
impl Material for Lambertian {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray)> {
        let scatter_direction = record_in.normal + rand_unit_vec();

        Some((
            self.albedo,
            Ray {
                origin: record_in.position,
                direction: if !vec_near_zero(scatter_direction) {
                    scatter_direction
                } else {
                    record_in.normal
                },
            },
        ))
    }
}
struct Metal {
    albedo: RgbColor,
    fuzz: f32,
}
impl Material for Metal {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray)> {
        let reflected = reflect(ray_in.direction.normalize(), record_in.normal);
        if reflected.dot(record_in.normal) > 0.0 {
            Some((
                self.albedo,
                Ray {
                    origin: record_in.position,
                    direction: reflected + self.fuzz * rand_unit_vec(),
                },
            ))
        } else {
            None
        }
    }
}
pub fn rand_unit_vec() -> Vector3<f32> {
    loop {
        let v = 2.0 * (rand_vec() - Vector3::new(0.5, 0.5, 0.5));
        if v.dot(v) < 1.0 {
            return v;
        }
    }
}
/// generates random vec with all components in range [0,1)
pub fn rand_vec() -> Vector3<f32> {
    Vector3 {
        x: rand::random(),
        y: rand::random(),
        z: rand::random(),
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}
impl Ray {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction.normalize()
    }
}
fn ray_color(ray: Ray, world: &World, depth: u32) -> RgbColor {
    if depth == 0 {
        return RgbColor {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        };
    }
    if let Some(record) = world.nearest_hit(&ray) {
        let target = record.position + record.normal + rand_unit_vec();
        let new_ray = Ray {
            origin: record.position,
            direction: target - record.position,
        };
        if let Some((color, scattered_ray)) = record.material.borrow().scatter(ray, &record) {
            return color * ray_color(scattered_ray, world, depth - 1);
        } else {
            RgbColor {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            }
        }
    } else {
        let unit = ray.direction.normalize();
        let t = 0.5 * (unit.y + 1.0);
        (1.0 - t)
            * RgbColor {
                red: 1.0,
                blue: 1.0,
                green: 1.0,
            }
            + t * RgbColor {
                red: 0.5,
                green: 0.7,
                blue: 1.0,
            }
    }
}
#[derive(Clone)]
pub struct HitRecord {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub t: f32,
    front_face: bool,
    material: Rc<RefCell<dyn Material>>,
}

impl HitRecord {
    pub fn new(
        ray: &Ray,
        position: Point3<f32>,
        normal: Vector3<f32>,
        t: f32,
        material: Rc<RefCell<dyn Material>>,
    ) -> Self {
        let front_face = ray.direction.dot(normal) < 0.0;
        Self {
            position,
            normal: if front_face { normal } else { -1.0 * normal },
            t,
            front_face,
            material,
        }
    }
}
pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}
#[derive(Clone)]
struct Sphere {
    pub radius: f32,
    pub origin: Point3<f32>,
    pub material: Rc<RefCell<dyn Material>>,
}
impl Sphere {
    pub fn did_intercept(&self, ray: &Ray) -> bool {
        let rel_origin = ray.origin - self.origin;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * (rel_origin.dot(ray.direction));
        let c = rel_origin.dot(rel_origin) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant > 0.0 {
            true
        } else {
            false
        }
    }
    /// gets distance along ray where intercept occurred
    pub fn intercept_t(&self, ray: &Ray) -> Option<f32> {
        let rel_origin = ray.origin - self.origin;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * (rel_origin.dot(ray.direction));
        let c = rel_origin.dot(rel_origin) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            None
        } else {
            Some((-1.0 * b - discriminant.sqrt()) / (2.0 * a))
        }
    }
}
impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let rel_origin = ray.origin - self.origin;
        let a = ray.direction.dot(ray.direction);
        let half_b = rel_origin.dot(ray.direction);
        let c = rel_origin.dot(rel_origin) - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrt_d = discriminant.sqrt();
        let mut root = (-1.0 * half_b - sqrt_d) / a;
        if root < t_min || t_max < root {
            root = (-1.0 * half_b + sqrt_d) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }
        let position = ray.at(root);
        return Some(HitRecord::new(
            ray,
            position,
            (position - self.origin) / self.radius,
            root,
            self.material.clone(),
        ));
    }
}
#[derive(Clone)]
pub struct World {
    spheres: Vec<Sphere>,
}
impl World {
    pub fn nearest_hit(&self, ray: &Ray) -> Option<HitRecord> {
        self.spheres
            .iter()
            .filter_map(|s| s.hit(ray, 0.0, f32::MAX))
            .reduce(|acc, x| if acc.t < x.t { acc } else { x })
    }
}
#[derive(Clone, Debug)]
pub struct Camera {
    origin: Point3<f32>,
    world_width: f32,
    world_height: f32,
    focal_length: f32,
    image_width: u32,
    image_height: u32,
}
impl Camera {
    pub fn new(
        image_width: u32,
        image_height: u32,
        world_height: f32,
        focal_length: f32,
        origin: Point3<f32>,
    ) -> Self {
        let aspect_ratio = image_width as f32 / image_height as f32;
        Self {
            origin,
            world_width: aspect_ratio * world_height,
            world_height,
            focal_length,
            image_width,
            image_height,
        }
    }
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: self.origin,
            direction: Vector3 {
                x: (u - 0.5) * self.world_height,
                y: (v - 0.5) * self.world_width,
                z: -1.0 * self.focal_length,
            },
        }
    }
}
pub struct RayTracer {
    sender: Sender<Image>,
}

impl RayTracer {
    const SAMPLES_PER_PIXEL: usize = 80;
    pub fn new() -> Receiver<Image> {
        let (sender, recvier) = channel();
        let s = Self { sender };
        thread::spawn(move || s.start_tracing());
        recvier
    }
    pub fn start_tracing(&self) {
        self.sender
            .send(Image::from_fn(|x, y| [0, 0, 0, 0xff], 1000, 1000))
            .expect("failed to send");

        const IMAGE_HEIGHT: u32 = 1000;
        const IMAGE_WIDTH: u32 = 1000;
        const FOCAL_LENGTH: f32 = 1.0;

        let image_world_height = 2.0;
        let camera = Camera::new(
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            image_world_height,
            FOCAL_LENGTH,
            Point3 {
                x: 0.0f32,
                y: 0.0f32,
                z: 0.0f32,
            },
        );
        let world = World {
            spheres: vec![
                Sphere {
                    radius: 0.3,
                    origin: Point3 {
                        x: 0.0,
                        y: -0.1,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Lambertian {
                        albedo: RgbColor {
                            red: 0.5,
                            green: 0.5,
                            blue: 0.5,
                        },
                    })),
                },
                Sphere {
                    radius: 0.3,
                    origin: Point3 {
                        x: -0.7,
                        y: -0.1,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Metal {
                        albedo: RgbColor {
                            red: 0.8,
                            green: 0.6,
                            blue: 0.2,
                        },
                        fuzz: 0.1,
                    })),
                },
                Sphere {
                    radius: 0.3,
                    origin: Point3 {
                        x: 0.7,
                        y: -0.1,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Metal {
                        albedo: RgbColor {
                            red: 0.9,
                            green: 0.6,
                            blue: 0.2,
                        },
                        fuzz: 0.6,
                    })),
                },
                Sphere {
                    radius: 100.0,
                    origin: Point3 {
                        x: 0.0,
                        y: -100.5,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Lambertian {
                        albedo: RgbColor {
                            red: 0.1,
                            green: 0.5,
                            blue: 0.1,
                        },
                    })),
                },
            ],
        };

        let mut rgb_img = RgbImage::new_black(1000, 1000);
        for num_s in 0..Self::SAMPLES_PER_PIXEL {
            for x in 0..IMAGE_WIDTH {
                for y in 0..IMAGE_WIDTH {
                    let u = (x as f32 + rand::random::<f32>()) / (IMAGE_WIDTH as f32 - 1.0);
                    let v = (y as f32 + rand::random::<f32>()) / (IMAGE_HEIGHT as f32 - 1.0);
                    let r = camera.get_ray(u, v);
                    let c = ray_color(r, &world, 5);
                    rgb_img.add_xy(x, y, ray_color(r, &world, 20));
                }
            }

            self.sender
                .send(Image::from_rgb_image(&(rgb_img.clone() / num_s as f32)));
        }
    }
}
