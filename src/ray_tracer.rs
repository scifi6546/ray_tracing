mod background;
mod bvh;
mod camera;
mod hittable;
mod material;
mod pdf;
mod texture;

use super::{prelude::*, vec_near_zero, Image};
use crate::reflect;

use background::{Background, ConstantColor, Sky};
use bvh::AABB;
use camera::Camera;
use cgmath::{InnerSpace, Point3, Vector3};
use hittable::{
    ConstantMedium, FlipNormals, HitRecord, Hittable, Light, MovingSphere, RenderBox, RotateY,
    Sphere, Translate, XYRect,
};
use material::{Dielectric, DiffuseLight, Isotropic, Lambertian, Material, Metal};
use pdf::CosinePdf;
use texture::{CheckerTexture, DebugV, ImageTexture, Perlin, SolidColor, Texture};

use crate::ray_tracer::hittable::{XZRect, YZRect};
use crate::ray_tracer::pdf::PDF;
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

fn ray_color(ray: Ray, world: &World, depth: u32) -> RgbColor {
    if depth == 0 {
        return RgbColor {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        };
    }
    if let Some(record) = world.nearest_hit(&ray, 0.001, f32::MAX) {
        let emitted = record.material.borrow().emmit(&record);
        if let Some((color, scattered_ray, pdf)) = record.material.borrow().scatter(ray, &record) {
            let pdf = CosinePdf {
                uvw: OrthoNormalBasis::build_from_w(record.normal),
            };
            let (pdf_direction, value) = pdf.generate(world);

            emitted
                + color
                    * ray_color(
                        Ray {
                            origin: scattered_ray.origin,
                            direction: pdf_direction,
                            time: scattered_ray.time,
                        },
                        world,
                        depth - 1,
                    )
                    * record
                        .material
                        .borrow()
                        .scattering_pdf(ray, &record, scattered_ray)
                    / pdf.value(&scattered_ray.direction)
        } else {
            emitted
        }
    } else {
        world.background.color(ray)
    }
}

pub struct World {
    spheres: Vec<Rc<dyn Hittable>>,
    lights: Vec<Rc<dyn Light>>,
    background: Box<dyn Background>,
}
impl World {
    pub fn nearest_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.spheres
            .iter()
            .filter_map(|s| s.hit(ray, t_min, t_max))
            .reduce(|acc, x| if acc.t < x.t { acc } else { x })
    }
    pub fn to_bvh(self, time_0: f32, time_1: f32) -> Self {
        let sphere_len = self.spheres.len();
        Self {
            spheres: vec![Rc::new(bvh::BvhNode::new(
                self.spheres,
                0,
                sphere_len,
                time_0,
                time_1,
            ))],
            lights: self.lights.clone(),
            background: self.background,
        }
    }
}
#[allow(dead_code)]
fn random_scene() -> (World, Camera) {
    let big: [Rc<dyn Hittable>; 4] = [
        Rc::new(Sphere {
            radius: 1000.0,
            origin: Point3::new(0.0, -1000.0, 1000.0),
            material: Rc::new(RefCell::new(Lambertian {
                albedo: Box::new(SolidColor {
                    color: RgbColor {
                        red: 0.5,
                        green: 0.5,
                        blue: 0.5,
                    },
                }),
            })),
        }),
        Rc::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Dielectric {
                index_refraction: 1.5,
                color: RgbColor::new(1.0, 1.0, 1.0),
            })),
        }),
        Rc::new(Sphere {
            radius: 1.0,
            origin: Point3::new(-4.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Lambertian {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.4, 0.2, 0.1),
                }),
            })),
        }),
        Rc::new(Sphere {
            radius: 1.0,
            origin: Point3::new(4.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Metal {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.4, 0.2, 0.1),
                }),
                fuzz: 0.0,
            })),
        }),
    ];
    let spheres = (-11..11)
        .flat_map(|a| {
            (-11..11).filter_map::<Rc<dyn Hittable>, _>(move |b| {
                let choose_mat = rand::random::<f32>();
                let center = Point3::new(
                    a as f32 + 0.9 * rand::random::<f32>(),
                    0.2,
                    b as f32 + 0.9 * rand::random::<f32>(),
                );
                let check = center - Point3::new(4.0, 0.2, 0.0);
                if check.dot(check).sqrt() > 0.9 {
                    if choose_mat < 0.8 {
                        Some(Rc::new(MovingSphere {
                            radius: 0.2,
                            center_0: center,
                            center_1: center + Vector3::new(0.0, rand_f32(0.0, 0.5), 0.0),
                            time_0: 0.0,
                            time_1: 1.0,
                            material: Rc::new(RefCell::new(Lambertian {
                                albedo: Box::new(SolidColor {
                                    color: RgbColor::random(),
                                }),
                            })),
                        }))
                    } else if choose_mat < 0.95 {
                        Some(Rc::new(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Metal {
                                albedo: Box::new(SolidColor {
                                    color: RgbColor::random(),
                                }),
                                fuzz: rand::random::<f32>() * 0.5 + 0.5,
                            })),
                        }))
                    } else {
                        Some(Rc::new(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Dielectric {
                                color: RgbColor::new(1.0, 1.0, 1.0),
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
        World {
            spheres,
            lights: vec![],
            background: Box::new(Sky {}),
        },
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
    let origin = Point3::new(10.0f32, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    (
        World {
            spheres: vec![
                Rc::new(Sphere {
                    radius: 100.0,
                    origin: Point3 {
                        x: 0.0,
                        y: -100.5,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Lambertian {
                        albedo: Box::new(CheckerTexture {
                            even: Box::new(SolidColor {
                                color: RgbColor::new(0.5, 1.0, 0.0),
                            }),
                            odd: Box::new(Perlin::new()),
                        }),
                    })),
                }),
                Rc::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Lambertian {
                        albedo: Box::new(ImageTexture::new("./assets/earthmap.jpg")),
                    })),
                }),
                Rc::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: -1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Dielectric {
                        color: RgbColor::new(1.0, 0.8, 0.8),
                        index_refraction: 1.5,
                    })),
                }),
                Rc::new(Sphere {
                    radius: -0.45,
                    origin: Point3 {
                        x: -1.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    material: Rc::new(RefCell::new(Dielectric {
                        color: RgbColor::new(1.0, 1.0, 1.0),
                        index_refraction: 1.5,
                    })),
                }),
                Rc::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Metal {
                        albedo: Box::new(DebugV {}),
                        fuzz: 0.0,
                    })),
                }),
                Rc::new(XYRect {
                    x0: -0.5,
                    x1: 0.5,
                    y0: -0.5 + 1.0,
                    y1: 0.5 + 1.0,
                    k: -2.3,
                    material: Rc::new(RefCell::new(DiffuseLight {
                        emit: Box::new(SolidColor {
                            color: 0.5 * RgbColor::new(1.0, 1.0, 1.0),
                        }),
                    })),
                }),
                Rc::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 0.0,
                        y: 1.5,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(DiffuseLight {
                        emit: Box::new(SolidColor {
                            color: RgbColor::new(4.0, 4.0, 4.0),
                        }),
                    })),
                }),
            ],
            lights: vec![],
            background: Box::new(ConstantColor {
                color: RgbColor::new(0.05, 0.05, 0.05),
            }),
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
#[allow(dead_code)]
fn one_sphere() -> (World, Camera) {
    let look_at = Point3::new(0.0f32, 0.0, -1.0);
    let origin = Point3::new(3.0f32, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    (
        World {
            spheres: vec![Rc::new(Sphere {
                radius: 0.5,
                origin: Point3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                },
                material: Rc::new(RefCell::new(Lambertian {
                    albedo: Box::new(SolidColor {
                        color: RgbColor {
                            red: 0.1,
                            green: 0.2,
                            blue: 0.5,
                        },
                    }),
                })),
            })],
            lights: vec![],
            background: Box::new(Sky {}),
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
#[allow(dead_code)]
fn two_spheres() -> (World, Camera) {
    let look_at = Point3::new(0.0f32, 0.0, -1.0);
    let origin = Point3::new(3.0f32, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    (
        World {
            spheres: vec![
                Rc::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Lambertian {
                        albedo: Box::new(SolidColor {
                            color: RgbColor {
                                red: 0.1,
                                green: 0.2,
                                blue: 0.5,
                            },
                        }),
                    })),
                }),
                Rc::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Metal {
                        albedo: Box::new(SolidColor {
                            color: RgbColor::new(0.8, 0.6, 0.2),
                        }),
                        fuzz: 0.0,
                    })),
                }),
            ],
            lights: vec![],
            background: Box::new(Sky {}),
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
#[allow(dead_code)]
fn cornell_box() -> (World, Camera) {
    let look_at = Point3::new(278.0f32, 278.0, 0.0);
    let origin = Point3::new(278.0, 278.0, -800.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let green = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.12, 0.45, 0.15),
        }),
    }));
    let red = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.65, 0.05, 0.05),
        }),
    }));
    let light = Rc::new(RefCell::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: RgbColor::new(15.0, 15.0, 15.0),
        }),
    }));
    let white = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.73, 0.73, 0.73),
        }),
    }));
    let top_light = Rc::new(FlipNormals {
        item: Rc::new(XZRect {
            x0: 213.0,
            x1: 343.0,
            z0: 227.0,
            z1: 332.0,
            k: 554.0,
            material: light.clone(),
        }),
    });
    (
        World {
            spheres: vec![
                Rc::new(YZRect {
                    y0: 0.0,
                    y1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 555.0,
                    material: green.clone(),
                }),
                Rc::new(YZRect {
                    y0: 0.0,
                    y1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 0.0,
                    material: red.clone(),
                }),
                top_light.clone(),
                Rc::new(XZRect {
                    x0: 0.0,
                    x1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 0.0,
                    material: white.clone(),
                }),
                Rc::new(XZRect {
                    x0: 0.0,
                    x1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 555.0,
                    material: white.clone(),
                }),
                Rc::new(XYRect {
                    x0: 0.0,
                    x1: 555.0,
                    y0: 0.0,
                    y1: 555.0,
                    k: 555.0,
                    material: white.clone(),
                }),
                Rc::new(Translate {
                    item: Rc::new(RenderBox::new(
                        Point3::new(0.0, 0.0, 0.0),
                        Point3::new(165.0, 330.0, 165.0),
                        white.clone(),
                    )),

                    offset: Vector3::new(265.0, 0.0, 295.0),
                }),
                Rc::new(Translate {
                    item: Rc::new(RenderBox::new(
                        Point3::new(0.0, 0.0, 0.0),
                        Point3::new(165.0, 165.0, 165.0),
                        white.clone(),
                    )),
                    offset: Vector3::new(130.0, 0.0, 65.0),
                }),
            ],
            lights: vec![top_light],
            background: Box::new(ConstantColor {
                color: RgbColor::new(0.0, 0.0, 0.0),
            }),
        },
        Camera::new(
            IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32,
            40.0,
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
#[allow(dead_code)]
fn cornell_smoke() -> (World, Camera) {
    let look_at = Point3::new(278.0f32, 278.0, 0.0);
    let origin = Point3::new(278.0, 278.0, -800.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let green = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.12, 0.45, 0.15),
        }),
    }));
    let red = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.65, 0.05, 0.05),
        }),
    }));
    let light = Rc::new(RefCell::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: RgbColor::new(7.0, 7.0, 7.0),
        }),
    }));

    let white = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.73, 0.73, 0.73),
        }),
    }));
    let top_light = Rc::new(XZRect {
        x0: 113.0,
        x1: 443.0,
        z0: 127.0,
        z1: 423.0,
        k: 554.0,
        material: light.clone(),
    });
    (
        World {
            spheres: vec![
                Rc::new(YZRect {
                    y0: 0.0,
                    y1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 555.0,
                    material: green.clone(),
                }),
                Rc::new(YZRect {
                    y0: 0.0,
                    y1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 0.0,
                    material: red.clone(),
                }),
                top_light.clone(),
                Rc::new(XZRect {
                    x0: 0.0,
                    x1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 0.0,
                    material: white.clone(),
                }),
                Rc::new(XZRect {
                    x0: 0.0,
                    x1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 555.0,
                    material: white.clone(),
                }),
                Rc::new(XYRect {
                    x0: 0.0,
                    x1: 555.0,
                    y0: 0.0,
                    y1: 555.0,
                    k: 555.0,
                    material: white.clone(),
                }),
                Rc::new(ConstantMedium::new(
                    Rc::new(Translate {
                        item: Rc::new(RenderBox::new(
                            Point3::new(0.0, 0.0, 0.0),
                            Point3::new(165.0, 330.0, 165.0),
                            white.clone(),
                        )),

                        offset: Vector3::new(265.0, 0.0, 295.0),
                    }),
                    Rc::new(RefCell::new(Isotropic {
                        albedo: Box::new(SolidColor {
                            color: RgbColor::new(0.0, 0.0, 0.0),
                        }),
                    })),
                    0.01,
                )),
                Rc::new(ConstantMedium::new(
                    Rc::new(Translate {
                        item: Rc::new(Sphere {
                            radius: 100.0,
                            origin: Point3::new(0.0, 0.0, 0.0),
                            material: white.clone(),
                        }),

                        offset: Vector3::new(265.0, 500.0, 295.0),
                    }),
                    Rc::new(RefCell::new(Isotropic {
                        albedo: Box::new(SolidColor {
                            color: RgbColor::new(0.5, 0.0, 0.0),
                        }),
                    })),
                    0.01,
                )),
                /*
                Rc::new(Translate {
                    item: Rc::new(RotateY::new(
                        Rc::new(RenderBox::new(
                            Point3::new(0.0, 0.0, 0.0),
                            Point3::new(165.0, 330.0, 165.0),
                            white.clone(),
                        )),
                        15.0,
                    )),
                    offset: Vector3::new(265.0, 0.0, 295.0),
                }),

                 */
                Rc::new(Translate {
                    item: Rc::new(RotateY::new(
                        Rc::new(RenderBox::new(
                            Point3::new(0.0, 0.0, 0.0),
                            Point3::new(165.0, 165.0, 165.0),
                            white.clone(),
                        )),
                        -18.0,
                    )),
                    offset: Vector3::new(130.0, 0.0, 65.0),
                }),
            ],
            lights: vec![top_light],
            background: Box::new(ConstantColor {
                color: RgbColor::new(0.0, 0.0, 0.0),
            }),
        },
        Camera::new(
            IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32,
            40.0,
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

        let (world, camera) = cornell_box();
        let world = world.to_bvh(camera.start_time(), camera.end_time());
        println!(
            "world bounding box: {:#?}",
            world.spheres[0].bounding_box(0.0, 0.0)
        );

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
