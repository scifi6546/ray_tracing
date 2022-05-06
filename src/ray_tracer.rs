mod bvh;
mod camera;
mod hittable;
mod material;

use super::{prelude::*, vec_near_zero, Image, RgbColor, RgbImage};
use crate::reflect;
use bvh::AABB;
use camera::Camera;
use cgmath::{InnerSpace, Point3, Vector3};
use hittable::{Hittable, MovingSphere, Sphere};
use material::{Dielectric, Lambertian, Material, Metal};

use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};
const IMAGE_HEIGHT: u32 = 1000;
const IMAGE_WIDTH: u32 = 1000;

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
    pub time: f32,
}
impl Ray {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction
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
    if let Some(record) = world.nearest_hit(&ray, 0.001, f32::MAX) {
        if let Some((color, scattered_ray)) = record.material.borrow().scatter(ray, &record) {
            color * ray_color(scattered_ray, world, depth - 1)
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

pub struct World {
    spheres: Vec<Box<dyn Hittable>>,
}
impl World {
    pub fn nearest_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.spheres
            .iter()
            .filter_map(|s| s.hit(ray, t_min, t_max))
            .reduce(|acc, x| if acc.t < x.t { acc } else { x })
    }
}
#[allow(dead_code)]
fn random_scene() -> (World, Camera) {
    let big: [Box<dyn Hittable>; 4] = [
        Box::new(Sphere {
            radius: 1000.0,
            origin: Point3::new(0.0, -1000.0, 1000.0),
            material: Rc::new(RefCell::new(Lambertian {
                albedo: RgbColor {
                    red: 0.5,
                    green: 0.5,
                    blue: 0.5,
                },
            })),
        }),
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Dielectric {
                index_refraction: 1.5,
            })),
        }),
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(-4.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Lambertian {
                albedo: RgbColor::new(0.4, 0.2, 0.1),
            })),
        }),
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(4.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Metal {
                albedo: RgbColor::new(0.4, 0.2, 0.1),
                fuzz: 0.0,
            })),
        }),
    ];
    let spheres = (-11..11)
        .flat_map(|a| {
            (-11..11).filter_map::<Box<dyn Hittable>, _>(move |b| {
                let choose_mat = rand::random::<f32>();
                let center = Point3::new(
                    a as f32 + 0.9 * rand::random::<f32>(),
                    0.2,
                    b as f32 + 0.9 * rand::random::<f32>(),
                );
                let check = center - Point3::new(4.0, 0.2, 0.0);
                if check.dot(check).sqrt() > 0.9 {
                    if choose_mat < 0.8 {
                        Some(Box::new(MovingSphere {
                            radius: 0.2,
                            center_0: center,
                            center_1: center + Vector3::new(0.0, rand_f32(0.0, 0.5), 0.0),
                            time_0: 0.0,
                            time_1: 1.0,
                            material: Rc::new(RefCell::new(Lambertian {
                                albedo: RgbColor::random(),
                            })),
                        }))
                    } else if choose_mat < 0.95 {
                        Some(Box::new(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Metal {
                                albedo: RgbColor::random(),
                                fuzz: rand::random::<f32>() * 0.5 + 0.5,
                            })),
                        }))
                    } else {
                        Some(Box::new(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Dielectric {
                                index_refraction: 1.5,
                            })),
                        }))
                    }
                } else {
                    None
                }
            })
        })
        .chain(big)
        .collect();
    (
        World { spheres },
        Camera::new(
            IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32,
            20.0,
            Point3::new(13.0, 2.0, 3.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            0.0005,
            10.0,
            0.0,
            1.0,
        ),
    )
}
#[allow(dead_code)]
fn easy_scene() -> (World, Camera) {
    let look_at = Point3::new(0.0f32, 0.0, -1.0);
    let origin = Point3::new(3.0f32, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    (
        World {
            spheres: vec![
                Box::new(Sphere {
                    radius: 100.0,
                    origin: Point3 {
                        x: 0.0,
                        y: -100.5,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Lambertian {
                        albedo: RgbColor {
                            red: 0.8,
                            green: 0.8,
                            blue: 0.0,
                        },
                    })),
                }),
                Box::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Lambertian {
                        albedo: RgbColor {
                            red: 0.1,
                            green: 0.2,
                            blue: 0.5,
                        },
                    })),
                }),
                Box::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: -1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Dielectric {
                        index_refraction: 1.5,
                    })),
                }),
                Box::new(Sphere {
                    radius: -0.45,
                    origin: Point3 {
                        x: -1.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    material: Rc::new(RefCell::new(Dielectric {
                        index_refraction: 1.5,
                    })),
                }),
                Box::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Metal {
                        albedo: RgbColor::new(0.8, 0.6, 0.2),
                        fuzz: 0.0,
                    })),
                }),
            ],
        },
        Camera::new(
            IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32,
            20.0,
            origin,
            look_at,
            Vector3::new(0.0, 1.0, 0.0),
            0.00001,
            focus_distance,
            0.0,
            0.0,
        ),
    )
}
pub struct RayTracer {
    sender: Sender<Image>,
}

impl RayTracer {
    const SAMPLES_PER_PIXEL: usize = 500;
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Receiver<Image> {
        let (sender, recvier) = channel();
        let s = Self { sender };
        thread::spawn(move || s.start_tracing());
        recvier
    }
    pub fn start_tracing(&self) {
        self.sender
            .send(Image::from_fn(
                |_x, _y| [0, 0, 0, 0xff],
                IMAGE_WIDTH,
                IMAGE_HEIGHT,
            ))
            .expect("failed to send");

        let (world, camera) = random_scene();

        let mut rgb_img = RgbImage::new_black(1000, 1000);
        for num_s in 0..Self::SAMPLES_PER_PIXEL {
            for x in 0..IMAGE_WIDTH {
                for y in 0..IMAGE_WIDTH {
                    let u = (x as f32 + rand_f32(0.0, 1.0)) / (IMAGE_WIDTH as f32 - 1.0);
                    let v = (y as f32 + rand_f32(0.0, 1.0)) / (IMAGE_HEIGHT as f32 - 1.0);
                    let r = camera.get_ray(u, v);

                    rgb_img.add_xy(x, y, ray_color(r, &world, 50));
                }
            }

            self.sender
                .send(Image::from_rgb_image(&(rgb_img.clone() / num_s as f32)))
                .expect("channel failed");
        }
    }
}
