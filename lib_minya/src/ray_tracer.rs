pub mod background;
mod bloom;
mod bvh;
pub mod camera;
pub mod hittable;
pub mod logger;
pub mod material;
mod pdf;
pub mod texture;
pub mod world;
use super::{prelude::*, Image};
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
pub use world::{ScenarioCtor, World};

use std::collections::HashMap;
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Instant,
};

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
pub trait Shader {
    fn ray_color(&self, ray: Ray, world: &World, depth: u32) -> RgbColor;
}
pub struct LightMapShader {}
impl Shader for LightMapShader {
    fn ray_color(&self, ray: Ray, world: &World, depth: u32) -> RgbColor {
        if depth == 0 {
            return RgbColor {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            };
        }
        if let Some(record) = world.nearest_hit(&ray, 0.001, f32::MAX) {
            world
                .lights
                .iter()
                .map(|l| {
                    let area = l.generate_ray_in_area(record.position, record.t);
                    if let Some(r) = world.nearest_hit(&area.to_area, 0.001, f32::MAX) {
                        let at = area.end_point;
                        let t = (at - r.position);
                        let m = t.magnitude();
                        let o = m * RgbColor::WHITE;
                        if o.is_nan() {
                            RgbColor::new(1.0, 0.0, 0.0)
                        } else {
                            o
                        }
                    } else {
                        RgbColor::BLACK
                    }
                })
                .fold(RgbColor::BLACK, |acc, x| acc + x)
        } else {
            RgbColor::BLACK
        }
    }
}
pub struct DiffuseShader {}
impl Shader for DiffuseShader {
    fn ray_color(&self, ray: Ray, world: &World, depth: u32) -> RgbColor {
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
                return scatter_record.attenuation;
            } else {
                // emitted
                RgbColor::BLACK
            }
        } else {
            world.background.color(ray)
        }
    }
}
pub struct RayTracingShader {}
impl Shader for RayTracingShader {
    fn ray_color(&self, ray: Ray, world: &World, depth: u32) -> RgbColor {
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
                    scatter_record.attenuation * self.ray_color(specular_ray, world, depth - 1)
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
                            * self.ray_color(
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
}
#[derive(Clone, Copy, Debug)]
pub enum CurrentShader {
    Raytracing,
    Diffuse,
    LightMap,
}
impl CurrentShader {
    pub fn names() -> [String; 3] {
        [
            "Ray Tracing".to_string(),
            "Diffuse".to_string(),
            "LightMap".to_string(),
        ]
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "Ray Tracing" => Self::Raytracing,
            "Diffuse" => Self::Diffuse,
            "LightMap" => Self::LightMap,
            _ => panic!("invalid name"),
        }
    }
}
pub struct RayTracer {
    scenarios: HashMap<String, Box<dyn ScenarioCtor>>,
    world: World,
    current_shader: CurrentShader,
    ray_tracing_shader: RayTracingShader,
    diffuse_shader: DiffuseShader,
    light_map_shader: LightMapShader,
}
pub struct RayTracerInfo {
    pub scenarios: Vec<String>,
}

impl RayTracer {
    pub fn new(
        additional_scenarios: Option<HashMap<String, Box<dyn ScenarioCtor>>>,
        default_scenario: Option<String>,
        default_shader: Option<CurrentShader>,
    ) -> Self {
        Logger::init();
        let current_shader = match default_shader {
            Some(s) => s,
            None => CurrentShader::Raytracing,
        };
        let mut scenarios = world::get_scenarios();
        if let Some(mut add_scenarios) = additional_scenarios {
            for (k, scenario) in add_scenarios.drain() {
                scenarios.items.insert(k, scenario);
            }
        }
        let default = match default_scenario {
            Some(s) => s,
            None => scenarios.default,
        };
        let world = scenarios.items[&default].build();
        Self {
            scenarios: scenarios.items,
            world,
            ray_tracing_shader: RayTracingShader {},
            diffuse_shader: DiffuseShader {},
            light_map_shader: LightMapShader {},
            current_shader,
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
    pub fn set_shader(&mut self, shader: CurrentShader) {
        self.current_shader = shader
    }
    /// Does one ray tracing step and saves result to image
    pub fn trace_image(&self, rgb_img: &mut RgbImage) {
        let image_width = rgb_img.width();
        let image_height = rgb_img.height();
        for x in 0..image_width {
            for y in 0..image_height {
                let u = (x as f32 + rand_f32(0.0, 1.0)) / (image_width as f32 - 1.0);
                let v = (y as f32 + rand_f32(0.0, 1.0)) / (image_height as f32 - 1.0);
                let r = self.world.camera.get_ray(u, v);
                let c = match self.current_shader {
                    CurrentShader::Diffuse => self.diffuse_shader.ray_color(r, &self.world, 50),
                    CurrentShader::Raytracing => {
                        self.ray_tracing_shader.ray_color(r, &self.world, 50)
                    }
                    CurrentShader::LightMap => self.light_map_shader.ray_color(r, &self.world, 50),
                };

                if c.is_nan() {
                    error!("ray color retuned NaN");
                }
                rgb_img.add_xy(x, y, c);
            }
        }
    }
    /// performs post processing step on image
    pub fn post_process(&self, rgb_img: &mut RgbImage) {
        bloom(rgb_img);
    }
    /// renders current scene to image
    pub fn tracing_loop(&self, rgb_img: &mut RgbImage, num_samples: usize) {
        for _ in 0..num_samples {
            self.trace_image(rgb_img);
        }
        *rgb_img = rgb_img.clone() / num_samples as f32;
        self.post_process(rgb_img);
    }
}
