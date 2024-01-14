mod voxel;

use cgmath::{InnerSpace, MetricSpace, Point3, Vector3};
use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Div, Mul, Sub},
};
pub use voxel::VoxelGrid;

pub struct Scene {
    pub name: String,
    pub objects: Vec<Object>,
    pub camera: Camera,
    pub background: Background,
}
pub enum Background {
    Sky,
    ConstantColor(RgbColor),
}
pub struct Camera {
    pub near_clip: f32,
    pub far_clip: f32,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub look_at: Point3<f32>,
    pub origin: Point3<f32>,
    pub up_vector: Vector3<f32>,
    pub aperture: f32,
    pub focus_distance: f32,
    pub start_time: f32,
    pub end_time: f32,
}
#[derive(Clone, Debug)]
pub enum Texture {
    ConstantColor(RgbColor),
}
#[derive(Clone, Debug)]
pub enum Material {
    Light(Texture),
    Lambertian(Texture),
}
pub enum Modifiers {
    FlipNormals,
}
pub struct Object {
    pub shape: Shape,
    pub material: Material,
    pub modifiers: Vec<Modifiers>,
}
pub enum Shape {
    Sphere {
        radius: f32,
        origin: Point3<f32>,
    },
    XZRect {
        center: Vector3<f32>,
        size_x: f32,
        size_z: f32,
    },
    YZRect {
        center: Vector3<f32>,
        size_y: f32,
        size_z: f32,
    },
    XYRect {
        center: Vector3<f32>,
        size_x: f32,
        size_y: f32,
    },
    RenderBox {
        center: Vector3<f32>,
        size_x: f32,
        size_y: f32,
        size_z: f32,
    },
    Voxels(VoxelGrid),
}
pub fn clamp<T: std::cmp::PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
pub fn get_scenarios() -> Vec<(String, fn() -> Scene)> {
    vec![
        ("from baselib".to_string(), || Scene {
            name: "test baselib scene".to_string(),
            objects: vec![Object {
                shape: Shape::Sphere {
                    radius: 0.5,
                    origin: Point3::new(0.0, 0.0, 0.0),
                },
                modifiers: vec![],
                material: Material::Light(Texture::ConstantColor(RgbColor::new(20.0, 0.0, 0.0))),
            }],
            background: Background::Sky,
            camera: Camera {
                aspect_ratio: 1.0,
                fov: 20.0,
                origin: Point3::new(10.0, 10.0, 10.0),
                look_at: Point3::new(0.0, 0.0, 0.0),
                up_vector: Vector3::new(0.0, 1.0, 0.0),
                aperture: 0.00001,
                focus_distance: 10.0,
                start_time: 0.0,
                end_time: 0.0,
                near_clip: 0.1,
                far_clip: 100.0,
            },
        }),
        ("Cornell Box Baselib".to_string(), || {
            let look_at = Point3::new(278.0f32, 278.0, 0.0);
            let origin = Point3::new(278.0, 278.0, -800.0);

            Scene {
                name: "Cornell Box".to_string(),
                objects: vec![
                    Object {
                        shape: Shape::XYRect {
                            center: Vector3::new(555.0 / 2.0, 555.0 / 2.0, 555.0),
                            size_x: 555.0 / 2.0,
                            size_y: 555.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.73, 0.73, 0.73,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::YZRect {
                            center: Vector3::new(555.0, 555.0 / 2.0, 555.0 / 2.0),
                            size_y: 555.0 / 2.0,
                            size_z: 555.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.12, 0.45, 0.15,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::YZRect {
                            center: Vector3::new(0.0, 555.0 / 2.0, 555.0 / 2.0),
                            size_y: 555.0 / 2.0,
                            size_z: 555.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.65, 0.05, 0.05,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::XZRect {
                            center: Vector3::new(555.0 / 2.0, 0.0, 555.0 / 2.0),
                            size_x: 555.0 / 2.0,
                            size_z: 555.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.73, 0.73, 0.73,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::XZRect {
                            center: Vector3::new(555.0 / 2.0, 555.0, 555.0 / 2.0),
                            size_x: 555.0 / 2.0,
                            size_z: 555.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.73, 0.73, 0.73,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::RenderBox {
                            center: Vector3::new(
                                165.0 / 2.0 + 265.0,
                                333.0 / 2.0,
                                165.0 / 2.0 + 295.0,
                            ),
                            size_x: 165.0 / 2.0,
                            size_y: 330.0 / 2.0,
                            size_z: 165.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.73, 0.73, 0.73,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::RenderBox {
                            center: Vector3::new(
                                165.0 / 2.0 + 130.0,
                                165.0 / 2.0,
                                165.0 / 2.0 + 65.0,
                            ),
                            size_x: 165.0 / 2.0,
                            size_y: 330.0 / 2.0,
                            size_z: 165.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.73, 0.73, 0.73,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::XZRect {
                            center: Vector3::new(
                                (343.0 + 213.0) / 2.0,
                                554.0,
                                (332.0 + 227.0) / 2.0,
                            ),
                            size_x: (343.0 - 213.0) / 2.0,
                            size_z: (332.0 - 227.0) / 2.0,
                        },
                        material: Material::Light(Texture::ConstantColor(RgbColor::new(
                            15.0, 15.0, 15.0,
                        ))),
                        modifiers: vec![Modifiers::FlipNormals],
                    },
                ],
                background: Background::ConstantColor(RgbColor::new(0.0, 0.0, 0.0)),
                camera: Camera {
                    aspect_ratio: 1.0,
                    fov: 40.0,
                    origin,
                    look_at,
                    up_vector: Vector3::new(0.0, 1.0, 0.0),
                    aperture: 0.00001,
                    focus_distance: {
                        let t = look_at - origin;
                        (t.dot(t)).sqrt()
                    },
                    start_time: 0.0,
                    end_time: 0.0,
                    near_clip: 1.0,
                    far_clip: 10000.0,
                },
            }
        }),
        ("Cornell Box Only".to_string(), || {
            let look_at = Point3::new(278.0f32, 278.0, 0.0);
            let origin = Point3::new(278.0, 278.0, -800.0);

            Scene {
                name: "Cornell Box Only".to_string(),
                objects: vec![
                    Object {
                        shape: Shape::RenderBox {
                            center: Vector3::new(
                                165.0 / 2.0 + 265.0,
                                333.0 / 2.0,
                                165.0 / 2.0 + 295.0,
                            ),
                            size_x: 165.0 / 2.0,
                            size_y: 330.0 / 2.0,
                            size_z: 165.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.73, 0.73, 0.73,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::RenderBox {
                            center: Vector3::new(
                                165.0 / 2.0 + 130.0,
                                165.0 / 2.0,
                                165.0 / 2.0 + 65.0,
                            ),
                            size_x: 165.0 / 2.0,
                            size_y: 330.0 / 2.0,
                            size_z: 165.0 / 2.0,
                        },
                        material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                            0.73, 0.73, 0.73,
                        ))),
                        modifiers: vec![],
                    },
                    Object {
                        shape: Shape::XZRect {
                            center: Vector3::new(
                                (343.0 + 213.0) / 2.0,
                                554.0,
                                (332.0 + 227.0) / 2.0,
                            ),
                            size_x: (343.0 - 213.0) / 2.0,
                            size_z: (332.0 - 227.0) / 2.0,
                        },
                        material: Material::Light(Texture::ConstantColor(RgbColor::new(
                            15.0, 15.0, 15.0,
                        ))),
                        modifiers: vec![Modifiers::FlipNormals],
                    },
                ],
                background: Background::ConstantColor(RgbColor::new(0.0, 0.0, 0.0)),
                camera: Camera {
                    aspect_ratio: 1.0,
                    fov: 40.0,
                    origin,
                    look_at,
                    up_vector: Vector3::new(0.0, 1.0, 0.0),
                    aperture: 0.00001,
                    focus_distance: {
                        let t = look_at - origin;
                        (t.dot(t)).sqrt()
                    },
                    start_time: 0.0,
                    end_time: 0.0,
                    near_clip: 1.0,
                    far_clip: 10000.0,
                },
            }
        }),
        ("Baselib Voxel Sphere".to_string(), || Scene {
            name: "Voxel Grid".into(),
            objects: vec![Object {
                shape: Shape::Voxels(VoxelGrid::new(
                    Vector3::new(320, 320, 320),
                    Point3::new(0.0, 0.0, 0.0),
                    |current_pos| {
                        let center = Point3::new(160.0, 160.0, 160.0);
                        let current_pos = Point3::new(
                            current_pos.x as f32,
                            current_pos.y as f32,
                            current_pos.z as f32,
                        );

                        center.distance(current_pos) < 50.0
                    },
                )),
                material: Material::Lambertian(Texture::ConstantColor(RgbColor::new(
                    0.73, 0.73, 0.73,
                ))),
                modifiers: vec![],
            }],
            background: Background::ConstantColor(RgbColor::new(1.0, 0.1, 0.1)),
            camera: Camera {
                aspect_ratio: 1.0,
                fov: 40.0,
                origin: Point3::new(500.0, 500.0, 100.0),
                look_at: Point3::new(0.0, 0.0, 0.0),
                up_vector: Vector3::new(0.0, 1.0, 0.0),
                aperture: 0.00001,
                focus_distance: {
                    let t = Point3::new(0.0f32, 0.0, 0.0) - Point3::new(500.0, 500.0, 100.0);
                    (t.dot(t)).sqrt()
                },
                start_time: 0.0,
                end_time: 0.0,
                near_clip: 1.0,
                far_clip: 10000.0,
            },
        }),
    ]
}
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}
impl RgbColor {
    pub const BLACK: Self = Self {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };
    pub const WHITE: Self = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };
    pub fn distance(&self, other: &Self) -> f32 {
        ((self.red - other.red).powi(2)
            + (self.green - other.green).powi(2)
            + (self.blue - other.blue).powi(2))
        .sqrt()
    }
    pub fn magnitude_squared(&self) -> f32 {
        self.red.powi(2) + self.green.powi(2) + self.blue.powi(2)
    }
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }
    pub fn random() -> Self {
        Self {
            red: rand::random(),
            green: rand::random(),
            blue: rand::random(),
        }
    }
    pub fn pow(&self, p: f32) -> Self {
        Self {
            red: self.red.powf(p),
            green: self.green.powf(p),
            blue: self.blue.powf(p),
        }
    }
    pub fn clamp(&self) -> RgbColor {
        Self {
            red: self.red.clamp(0.0, 1.0),
            green: self.green.clamp(0.0, 1.0),
            blue: self.blue.clamp(0.0, 1.0),
        }
    }
    pub fn exp(&self) -> Self {
        Self {
            red: self.red.exp(),
            green: self.green.exp(),
            blue: self.blue.exp(),
        }
    }
    pub fn as_rgb_u8(&self) -> [u8; 3] {
        let r = (clamp(self.red.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let g = (clamp(self.green.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let b = (clamp(self.blue.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        [r, g, b]
    }
    pub fn as_rgba_u8(&self) -> [u8; 4] {
        let [r, g, b] = self.as_rgb_u8();
        [r, g, b, 0xff]
    }
    pub fn normalize(self) -> Self {
        let mag = (self.red.powi(2) + self.green.powi(2) + self.blue.powi(2)).sqrt();
        mag * self
    }
    pub fn is_nan(&self) -> bool {
        self.red.is_nan() || self.green.is_nan() || self.blue.is_nan()
    }
}
impl Mul<f32> for RgbColor {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}
impl Mul<RgbColor> for f32 {
    type Output = RgbColor;
    fn mul(self, rhs: RgbColor) -> Self::Output {
        rhs * self
    }
}
impl Div<f32> for RgbColor {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            red: self.red / rhs,
            green: self.green / rhs,
            blue: self.blue / rhs,
        }
    }
}
impl Mul for RgbColor {
    type Output = RgbColor;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
        }
    }
}
impl Add for RgbColor {
    type Output = RgbColor;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}
impl Sub for RgbColor {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red - rhs.red,
            green: self.green - rhs.green,
            blue: self.blue - rhs.blue,
        }
    }
}
impl AddAssign for RgbColor {
    fn add_assign(&mut self, rhs: Self) {
        self.red += rhs.red;
        self.green += rhs.green;
        self.blue += rhs.blue;
    }
}
impl std::iter::Sum for RgbColor {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(
            RgbColor {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            },
            |acc, x| acc + x,
        )
    }
}

impl Display for RgbColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.red, self.green, self.blue)
    }
}
