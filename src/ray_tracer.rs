use super::{Image, RgbColor};
use cgmath::{InnerSpace, Point3, Vector3};
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Duration,
};
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}
impl Ray {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction.normalize()
    }
}
fn ray_color(ray: &Ray, world: &World) -> RgbColor {
    for s in world.spheres.iter() {
        if s.did_intercept(ray) {
            let t = s.intercept_t(ray).unwrap();
            let normal = ray.at(t)
                - Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                };
            return RgbColor {
                red: 0.5 * (normal.x + 1.0),
                green: 0.5 * (normal.y + 1.0),
                blue: 0.5 * (normal.z + 1.0),
            };
        }
    }

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
#[derive(Clone, Copy, Debug)]
pub struct HitRecord {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub t: f32,
}
pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}
#[derive(Clone, Copy, Debug)]
struct Sphere {
    pub radius: f32,
    pub origin: Point3<f32>,
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

        let discriminant = half_b * half_b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }
        todo!()
    }
}
#[derive(Clone, Debug)]
struct World {
    spheres: Vec<Sphere>,
}
pub struct RayTracer {
    sender: Sender<Image>,
}

impl RayTracer {
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
        const ASPECT_RATIO: f32 = IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32;
        let camera_origin = Point3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 0.0f32,
        };

        let world = World {
            spheres: vec![
                Sphere {
                    radius: 1.0,
                    origin: Point3 {
                        x: 0.0,
                        y: 0.0,
                        z: -2.0,
                    },
                },
                Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 0.6,
                        y: 0.8,
                        z: -2.0,
                    },
                },
            ],
        };
        let image_world_height = 2.0;
        let image_world_width = ASPECT_RATIO * image_world_height;
        let mut img = Image::from_fn(|_, _| [0, 0, 0, 0xff], 1000, 1000);
        for x in 0..IMAGE_WIDTH {
            for y in 0..IMAGE_WIDTH {
                let u = x as f32 / (IMAGE_WIDTH as f32 - 1.0);
                let v = y as f32 / (IMAGE_HEIGHT as f32 - 1.0);
                let r = Ray {
                    origin: camera_origin,
                    direction: Vector3 {
                        x: (u - 0.5) * image_world_height,
                        y: (v - 0.5) * image_world_width,
                        z: -1.0 * FOCAL_LENGTH,
                    },
                };
                img.set_xy_color(x, y, ray_color(&r, &world));
            }
        }

        self.sender.send(img).expect("failed to send");
    }
}
