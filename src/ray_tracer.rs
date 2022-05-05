mod camera;
use super::{prelude::*, vec_near_zero, Image, RgbColor, RgbImage};
use camera::Camera;

use crate::reflect;
use cgmath::{InnerSpace, Point3, Vector3};

use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};
const IMAGE_HEIGHT: u32 = 1000;
const IMAGE_WIDTH: u32 = 1000;
pub trait Material {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray)>;
}
struct Lambertian {
    albedo: RgbColor,
}
impl Material for Lambertian {
    fn scatter(&self, _ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray)> {
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
struct Dielectric {
    pub index_refraction: f32,
}
impl Dielectric {
    fn refract(uv: Vector3<f32>, n: Vector3<f32>, etai_over_etat: f32) -> Vector3<f32> {
        let cos_theta = n.dot(-1.0 * uv).min(1.0);

        let r_out_perp = etai_over_etat * (uv + cos_theta * n);
        let r_out_parallel = -1.0 * n * (1.0 - (r_out_perp.dot(r_out_perp))).abs().sqrt();
        r_out_perp + r_out_parallel
    }
    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * ((1.0 - cosine).powi(5))
    }
}
impl Material for Dielectric {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray)> {
        let refraction_ratio = if record_in.front_face {
            1.0 / self.index_refraction
        } else {
            self.index_refraction
        };
        let unit_direction = ray_in.direction.normalize();
        let cos_theta = record_in.normal.dot(-1.0 * unit_direction).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let can_not_refract = (refraction_ratio * sin_theta) > 1.0;
        let direction = if can_not_refract
            || Self::reflectance(cos_theta, refraction_ratio) > rand::random::<f32>()
        {
            reflect(unit_direction, record_in.normal)
        } else {
            Self::refract(unit_direction, record_in.normal, refraction_ratio)
        };

        Some((
            RgbColor {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
            },
            Ray {
                origin: record_in.position,
                direction,
            },
        ))
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
pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}
#[derive(Clone)]
struct Sphere {
    pub radius: f32,
    pub origin: Point3<f32>,
    pub material: Rc<RefCell<dyn Material>>,
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
        Some(HitRecord::new(
            ray,
            position,
            (position - self.origin) / self.radius,
            root,
            self.material.clone(),
        ))
    }
}
#[derive(Clone)]
pub struct World {
    spheres: Vec<Sphere>,
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
    let spheres = (-11..11)
        .flat_map(|a| {
            (-11..11).filter_map(move |b| {
                let choose_mat = rand::random::<f32>();
                let center = Point3::new(
                    a as f32 + 0.9 * rand::random::<f32>(),
                    0.2,
                    b as f32 + 0.9 * rand::random::<f32>(),
                );
                let check = center - Point3::new(4.0, 0.2, 0.0);
                if check.dot(check).sqrt() > 0.9 {
                    if choose_mat < 0.8 {
                        Some(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Lambertian {
                                albedo: RgbColor::random(),
                            })),
                        })
                    } else if choose_mat < 0.95 {
                        Some(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Metal {
                                albedo: RgbColor::random(),
                                fuzz: rand::random::<f32>() * 0.5 + 0.5,
                            })),
                        })
                    } else {
                        Some(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Dielectric {
                                index_refraction: 1.5,
                            })),
                        })
                    }
                } else {
                    None
                }
            })
        })
        .chain([
            Sphere {
                radius: 1000.0,
                origin: Point3::new(0.0, -1000.0, 1000.0),
                material: Rc::new(RefCell::new(Lambertian {
                    albedo: RgbColor {
                        red: 0.5,
                        green: 0.5,
                        blue: 0.5,
                    },
                })),
            },
            Sphere {
                radius: 1.0,
                origin: Point3::new(0.0, 1.0, 0.0),
                material: Rc::new(RefCell::new(Dielectric {
                    index_refraction: 1.5,
                })),
            },
            Sphere {
                radius: 1.0,
                origin: Point3::new(-4.0, 1.0, 0.0),
                material: Rc::new(RefCell::new(Lambertian {
                    albedo: RgbColor::new(0.4, 0.2, 0.1),
                })),
            },
            Sphere {
                radius: 1.0,
                origin: Point3::new(4.0, 1.0, 0.0),
                material: Rc::new(RefCell::new(Metal {
                    albedo: RgbColor::new(0.4, 0.2, 0.1),
                    fuzz: 0.0,
                })),
            },
        ])
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
    println!("focus distance: {}", focus_distance);
    (
        World {
            spheres: vec![
                Sphere {
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
                },
                Sphere {
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
                },
                Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: -1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Dielectric {
                        index_refraction: 1.5,
                    })),
                },
                Sphere {
                    radius: -0.45,
                    origin: Point3 {
                        x: -1.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    material: Rc::new(RefCell::new(Dielectric {
                        index_refraction: 1.5,
                    })),
                },
                Sphere {
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
                },
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
