pub mod background;
mod bloom;
mod bvh;
pub mod camera;
pub mod hittable;
pub mod logger;
pub mod material;
mod pdf;
mod sun;
pub mod texture;
pub mod world;

use super::prelude::*;
use crate::reflect;
use bloom::bloom;

pub use logger::LogMessage;
use logger::Logger;

use background::{Background, ConstantColor, Sky};
use bvh::Aabb;
use camera::Camera;
use cgmath::{InnerSpace, Point3, Vector3};
#[allow(unused_imports)]
use hittable::{
    ConstantMedium, HitRay, HitRecord, Hittable, MaterialEffect, MovingSphere, Object, RayAreaInfo,
    RenderBox, Sphere, Transform, XYRect, XZRect, YZRect,
};
#[allow(unused_imports)]
use material::{Dielectric, DiffuseLight, Isotropic, Lambertian, Material, Metal};
use pdf::{CosinePdf, LightPdf, PdfList, ScatterRecord};
#[allow(unused_imports)]
use texture::{CheckerTexture, DebugV, ImageTexture, MultiplyTexture, Perlin, SolidColor, Texture};
pub use world::{ScenarioCtor, World};

use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver},
        Arc, RwLock,
    },
    thread,
    thread::{Builder as ThreadBuilder, Scope},
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
#[derive(Clone, Debug)]
pub(crate) struct DebugRayTraceStep {
    position: Point3<f32>,
    front_face: bool,
}
#[derive(Clone, Debug)]
/// Color Output for shader. if tracing feature is enabled also traces old rays
pub(crate) struct RayColorOutput {
    pub(crate) color: RgbColor,
    #[cfg(feature = "debug_tracing")]
    pub(crate) steps: Vec<DebugRayTraceStep>,
}
pub(crate) trait Shader {
    fn ray_color(&self, ray: Ray, world: &World, depth: u32) -> RayColorOutput;
}
#[derive(Clone)]
pub struct LightMapShader {}
impl Shader for LightMapShader {
    fn ray_color(&self, ray: Ray, world: &World, depth: u32) -> RayColorOutput {
        if depth == 0 {
            return RayColorOutput {
                color: RgbColor::BLACK,
                #[cfg(feature = "debug_tracing")]
                steps: vec![],
            };
        }
        if let Some(record) = world.nearest_hit(&ray, 0.001, f32::MAX) {
            let color = world
                .lights
                .iter()
                .map(|l| {
                    let area = l.generate_ray_in_area(record.position, record.t);
                    if let Some(r) = world.nearest_hit(&area.to_area, 0.001, f32::MAX) {
                        let at = area.end_point;
                        let t = at - r.position;
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
                .fold(RgbColor::BLACK, |acc, x| acc + x);

            RayColorOutput {
                color,
                #[cfg(feature = "debug_tracing")]
                steps: vec![DebugRayTraceStep {
                    position: record.position,
                    front_face: true,
                }],
            }
        } else {
            RayColorOutput {
                color: RgbColor::BLACK,
                #[cfg(feature = "debug_tracing")]
                steps: vec![],
            }
        }
    }
}
#[derive(Clone)]
pub struct DiffuseShader {}
impl Shader for DiffuseShader {
    fn ray_color(&self, ray: Ray, world: &World, depth: u32) -> RayColorOutput {
        if depth == 0 {
            return RayColorOutput {
                color: RgbColor::BLACK,
                #[cfg(feature = "debug_tracing")]
                steps: vec![],
            };
        }
        #[cfg(feature = "debug_tracing")]
        let steps = vec![];
        if let Some(record) = world.nearest_hit(&ray, 0.001, f32::MAX) {
            match record.material_effect {
                MaterialEffect::Emmit(color) => RayColorOutput {
                    color,
                    #[cfg(feature = "debug_tracing")]
                    steps,
                },
                MaterialEffect::Scatter(record) => RayColorOutput {
                    color: record.attenuation,
                    #[cfg(feature = "debug_tracing")]
                    steps,
                },
                MaterialEffect::NoEmmit => RayColorOutput {
                    color: RgbColor::BLACK,
                    #[cfg(feature = "debug_tracing")]
                    steps,
                },
            }
        } else {
            RayColorOutput {
                color: world.background.color(ray),
                #[cfg(feature = "debug_tracing")]
                steps,
            }
        }
    }
}
#[derive(Clone)]
pub struct RayTracingShader {}
impl Shader for RayTracingShader {
    fn ray_color(&self, ray: Ray, world: &World, depth: u32) -> RayColorOutput {
        #[cfg(feature = "debug_tracing")]
        fn has_false_front_face(steps: &[DebugRayTraceStep]) -> bool {
            steps.iter().fold(true, |acc, x| acc == x.front_face)
        }
        if depth == 0 {
            return RayColorOutput {
                color: RgbColor::BLACK,
                #[cfg(feature = "debug_tracing")]
                steps: vec![],
            };
        }
        let output = if let Some(record) = world.nearest_hit(&ray, 0.001, f32::MAX) {
            #[cfg(feature = "debug_tracing")]
            let front_face = if (record.normal.dot(ray.direction) <= 0.0) != record.front_face {
                if rand_u32(0, 1_000_000) == 0 {
                    error!("not front face!",)
                    //error!("not front face?")
                }
                false
            } else {
                true
            };
            #[cfg(feature = "debug_tracing")]
            let step = DebugRayTraceStep {
                position: record.position,
                front_face,
            };
            match record.material_effect.clone() {
                MaterialEffect::Emmit(emitted) => {
                    if emitted.is_nan() {
                        error!("emmitted color is nan");
                    }
                    RayColorOutput {
                        color: emitted,
                        #[cfg(feature = "debug_tracing")]
                        steps: vec![step],
                    }
                }
                MaterialEffect::Scatter(scatter_record) => {
                    if let Some(specular_ray) = scatter_record.specular_ray {
                        let ray_color = self.ray_color(specular_ray, world, depth - 1);
                        let color = scatter_record.attenuation * ray_color.color;
                        #[cfg(feature = "debug_tracing")]
                        let mut steps = {
                            let mut out = vec![step];
                            out.append(&mut ray_color.steps.clone());
                            out
                        };
                        #[cfg(feature = "debug_tracing")]
                        {
                            if has_false_front_face(&ray_color.steps) {
                                if rand_u32(0, 1_000) == 0 {
                                    /*
                                    error!(
                                        "{:#?}\nrecord: {:#?}\nray:{:#?}",
                                        ray_color, record, ray
                                    );

                                     */
                                    //panic!()
                                }
                            }
                        }
                        RayColorOutput {
                            color,
                            #[cfg(feature = "debug_tracing")]
                            steps,
                        }
                    } else if let Some((pdf_direction, value)) = scatter_record
                        .pdf
                        .expect("if material is not specular there should be a pdf")
                        .generate(ray, record.position, world)
                    {
                        let scattering_pdf_fn = scatter_record.scattering_pdf;
                        let scattering_pdf = scattering_pdf_fn(
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
                                return RayColorOutput {
                                    color: RgbColor::BLACK,
                                    #[cfg(feature = "debug_tracing")]
                                    steps: vec![step],
                                };
                            }

                            let value = value / scattering_pdf;

                            let mut ray_color = self.ray_color(
                                Ray {
                                    origin: record.position,
                                    direction: pdf_direction,
                                    time: record.t,
                                },
                                world,
                                depth - 1,
                            );
                            #[cfg(feature = "debug_tracing")]
                            {
                                if has_false_front_face(&ray_color.steps) {
                                    if rand_u32(0, 1_000) == 0 {
                                        /*
                                        error!(
                                            "{:#?}\nrecord: {:#?}\nray:{:#?}",
                                            ray_color, record, ray
                                        );

                                         */
                                        //panic!()
                                    }
                                }
                            }
                            #[cfg(feature = "debug_tracing")]
                            let mut steps = {
                                let mut out = vec![step];
                                out.append(&mut ray_color.steps);
                                out
                            };
                            let color = scatter_record.attenuation * ray_color.color / value;
                            RayColorOutput {
                                color,
                                #[cfg(feature = "debug_tracing")]
                                steps,
                            }
                        } else {
                            RayColorOutput {
                                color: RgbColor::BLACK,
                                #[cfg(feature = "debug_tracing")]
                                steps: vec![step],
                            }
                        }
                    } else {
                        RayColorOutput {
                            color: RgbColor::BLACK,
                            #[cfg(feature = "debug_tracing")]
                            steps: vec![],
                        }
                    }
                }
                MaterialEffect::NoEmmit => RayColorOutput {
                    color: RgbColor::BLACK,
                    #[cfg(feature = "debug_tracing")]
                    steps: vec![],
                },
            }
        } else {
            RayColorOutput {
                color: world.background.color(ray),
                #[cfg(feature = "debug_tracing")]
                steps: vec![],
            }
        };
        #[cfg(feature = "debug_tracing")]
        if rand_u32(0, 1_000_000) == 0 {
            info!("{:#?}", output);
        }
        output
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
impl Clone for RayTracer {
    fn clone(&self) -> Self {
        Self {
            scenarios: self
                .scenarios
                .iter()
                .map(|(k, v)| (k.clone(), dyn_clone::clone_box(v.as_ref())))
                .collect(),
            world: self.world.clone(),
            current_shader: self.current_shader.clone(),
            ray_tracing_shader: self.ray_tracing_shader.clone(),
            diffuse_shader: self.diffuse_shader.clone(),
            light_map_shader: self.light_map_shader.clone(),
        }
    }
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
                .map(|(name, _scenario)| name.clone())
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
    pub fn trace_image(&self, rgb_img: &mut ParallelImage) {
        let mut imgs = rgb_img.split(1);
        self.trace_part(&mut imgs[0]);

        *rgb_img = ParallelImage::join(imgs.iter().collect());
    }
    fn trace_part(&self, part: &mut ParallelImagePart) {
        let image_width = part.width();
        let image_height = part.height();
        let total_width = part.total_width();
        let total_height = part.total_height();
        let offset = part.offset();
        let mut total_color = RgbColor::BLACK;
        for x in offset.x..offset.x + image_width {
            for y in offset.y..offset.y + image_height {
                let u = (x as f32 + rand_f32(0.0, 1.0)) / (total_width as f32 - 1.0);
                let v = (y as f32 + rand_f32(0.0, 1.0)) / (total_height as f32 - 1.0);
                let r = self.world.camera.get_ray(u, v);
                let c = match self.current_shader {
                    CurrentShader::Diffuse => self.diffuse_shader.ray_color(r, &self.world, 50),
                    CurrentShader::Raytracing => {
                        self.ray_tracing_shader.ray_color(r, &self.world, 50)
                    }
                    CurrentShader::LightMap => self.light_map_shader.ray_color(r, &self.world, 50),
                };

                if c.color.is_nan() {
                    error!("ray color retuned NaN");
                }
                total_color += c.color;
                part.add_xy(x, y, c.color);
            }
        }
    }
    /// performs post processing step on image
    pub fn post_process(&self, rgb_img: &mut ParallelImage) {
        bloom(rgb_img);
    }
    /// renders current scene to image
    pub fn tracing_loop(&self, parallel_image: &mut ParallelImage, num_samples: usize) {
        for _ in 0..num_samples {
            self.trace_image(parallel_image);
        }
        let mut post_process = parallel_image.clone() / num_samples as f32;
        self.post_process(&mut post_process);

        *parallel_image = post_process;
    }

    pub fn threaded_render(self, mut image: ParallelImage) -> ParallelImageCollector {
        fn loop_try_get(lock: &RwLock<RayTracer>) -> std::sync::RwLockReadGuard<RayTracer> {
            loop {
                let t = lock.try_read();
                match t {
                    Ok(t) => return t,
                    Err(e) => match e {
                        std::sync::TryLockError::Poisoned(_) => {
                            panic!()
                        }
                        std::sync::TryLockError::WouldBlock => {
                            thread::sleep(std::time::Duration::from_millis(10))
                        }
                    },
                }
            }
        }
        let num_threads = 8;
        let mut parts = image.split(num_threads);
        let mut receivers = vec![];
        let mut senders = vec![];
        let self_rw_lock = Arc::new(RwLock::new(self.clone()));
        for part in parts.drain(..) {
            let (mut sender, receiver) = image_channel();
            let (message_sender, message_receiver) = channel();
            senders.push(message_sender);
            let self_rw_lock = self_rw_lock.clone();
            thread::spawn(move || {
                let mut part = part;

                loop {
                    for msg in message_receiver.try_iter() {
                        match msg {
                            RayTracerMessage::LoadScenario(name) => part.set_black(),
                        };
                    }
                    {
                        let self_read_res = loop_try_get(&self_rw_lock);
                        //let self_read_res = self_rw_lock.read().expect("failed to get lock");
                        self_read_res.trace_part(&mut part);
                        sender.send(part.clone());
                    }
                    //self_clone.trace_part(&mut part);

                    //sender.send(part.clone());
                }
            });
            receivers.push(receiver);
        }
        ParallelImageCollector::new(receivers, senders, self_rw_lock)
    }
}
