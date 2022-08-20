mod background;
mod bloom;
mod bvh;
mod camera;
mod hittable;
pub mod logger;
mod material;
mod pdf;
mod texture;
mod world;
use super::{prelude::*, Image, Message};
use crate::reflect;
use bloom::bloom;

use log::{debug, error, info, trace, warn};
pub use logger::LogMessage;
use logger::Logger;

use background::{Background, ConstantColor, Sky};
use bvh::Aabb;
use camera::Camera;
use cgmath::{InnerSpace, Vector3};
#[allow(unused_imports)]
use hittable::{
    ConstantMedium, FlipNormals, HitRecord, Hittable, MovingSphere, Object, RayAreaInfo, RenderBox,
    Sphere, Transform, XYRect, XZRect, YZRect,
};
#[allow(unused_imports)]
use material::{Dielectric, DiffuseLight, Isotropic, Lambertian, Material, Metal};
use pdf::{CosinePdf, LightPdf, PdfList, ScatterRecord};
#[allow(unused_imports)]
use texture::{CheckerTexture, DebugV, ImageTexture, MultiplyTexture, Perlin, SolidColor, Texture};
use world::World;

use std::collections::HashMap;
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Instant,
};
use world::ScenarioCtor;

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
        if let Some(emitted) = record.material.emmit(&record) {
            if emitted.is_nan() {
                error!("emmited color is nan");
            }
            return emitted;
        };
        if let Some(scatter_record) = record.material.scatter(ray, &record) {
            if let Some(specular_ray) = scatter_record.specular_ray {
                scatter_record.attenuation * ray_color(specular_ray, world, depth - 1)
            } else if let Some((pdf_direction, value)) = scatter_record
                .pdf
                .expect("if material is not specular there should be a pdf")
                .generate(ray, record.position, world)
            {
                let scattering_pdf = record.material.scattering_pdf(
                    ray,
                    &record,
                    Ray {
                        origin: record.position,
                        direction: pdf_direction,
                        time: record.t,
                    },
                );
                if let Some(scattering_pdf) = scattering_pdf {
                    if scattering_pdf == 0.0 {
                        return RgbColor::BLACK;
                    }

                    let value = value / scattering_pdf;

                    scatter_record.attenuation
                        * ray_color(
                            Ray {
                                origin: record.position,
                                direction: pdf_direction,
                                time: record.t,
                            },
                            world,
                            depth - 1,
                        )
                        / value
                } else {
                    RgbColor::BLACK
                }
            } else {
                RgbColor::BLACK
            }
        } else {
            // emitted
            RgbColor::BLACK
        }
    } else {
        world.background.color(ray)
    }
}
static mut LOGGER: Option<Logger> = None;
pub struct RayTracer {
    scenarios: HashMap<String, Box<dyn ScenarioCtor>>,
    world: World,
}
pub struct RayTracerInfo {
    pub scenarios: Vec<String>,
}

impl RayTracer {
    pub fn new() -> Self {
        Logger::init();

        let scenarios = world::get_scenarios();
        let world = scenarios.items[&scenarios.default].build();
        Self {
            scenarios: scenarios.items,
            world,
        }
    }
    pub fn get_info(&self) -> RayTracerInfo {
        RayTracerInfo {
            scenarios: self
                .scenarios
                .iter()
                .map(|(name, scenario)| name.clone())
                .collect(),
        }
    }

    pub fn load_scenario(&mut self, scenario: String) {
        self.world = self.scenarios[&scenario].build();
    }
    /// renders current scene to image
    pub fn tracing_loop(&self, rgb_img: &mut RgbImage, num_samples: usize) {
        info!("tracing loop!!");
        for x in 0..IMAGE_WIDTH {
            for y in 0..IMAGE_WIDTH {
                let u = (x as f32 + rand_f32(0.0, 1.0)) / (IMAGE_WIDTH as f32 - 1.0);
                let v = (y as f32 + rand_f32(0.0, 1.0)) / (IMAGE_HEIGHT as f32 - 1.0);
                let r = self.world.camera.get_ray(u, v);
                let c = ray_color(r, &self.world, 50);
                if c.is_nan() {
                    error!("ray color retuned NaN");
                }
                rgb_img.add_xy(x, y, c);
            }
        }
        let mut send_img = rgb_img.clone() / num_samples as f32;

        bloom(&mut send_img);
    }
}
