pub mod background;
mod bloom;
mod bvh;
pub mod camera;
pub mod hittable;
pub mod logger;
pub mod material;

mod pdf;
pub mod ray_tracer_info;
mod save_file;
mod scenario_info;
mod sun;
pub mod texture;
mod unit_test;
pub mod world;
use ray_tracer_info::{RayTracerInfo, ScenarioInfo};

use super::prelude::*;
use crate::{prelude, reflect};
use bloom::bloom;

pub use logger::LogMessage;
use logger::Logger;

use crate::ray_tracer::ray_tracer_info::EntityField;

use background::{Background, ConstantColor};
use bvh::Aabb;
use camera::Camera;
use cgmath::{InnerSpace, Point3, Vector3};
use hittable::{HitRay, HitRecord, Hittable, MaterialEffect};
#[allow(unused_imports)]
use material::{Dielectric, DiffuseLight, Isotropic, Lambertian, Material, Metal};
use pdf::ScatterRecord;
use prelude::RayScalar;
use save_file::SceneFile;
use scenario_info::LoadScenario;
use std::{
    collections::HashMap,
    sync::{mpsc::channel, Arc, RwLock},
    thread,
};
#[allow(unused_imports)]
use texture::{CheckerTexture, DebugV, ImageTexture, MultiplyTexture, Perlin, SolidColor, Texture};
pub use world::{ScenarioCtor, World, WorldInfo};

pub fn rand_unit_vec() -> Vector3<RayScalar> {
    loop {
        let v = 2.0 * (rand_vec() - Vector3::new(0.5, 0.5, 0.5));
        if v.dot(v) < 1.0 {
            return v;
        }
    }
}
/// generates random vec with all components in range [0,1)
pub fn rand_vec() -> Vector3<RayScalar> {
    Vector3 {
        x: rand::random(),
        y: rand::random(),
        z: rand::random(),
    }
}
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct DebugRayTraceStep {
    #[allow(dead_code)]
    position: Point3<f32>,
    #[allow(dead_code)]
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

            RayColorOutput { color }
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

                            let ray_color = self.ray_color(
                                Ray {
                                    origin: record.position,
                                    direction: pdf_direction,
                                    time: record.t,
                                },
                                world,
                                depth - 1,
                            );

                            let color = scatter_record.attenuation * ray_color.color / value;
                            RayColorOutput { color }
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}
impl std::str::FromStr for CurrentShader {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Ray Tracing" => Ok(Self::Raytracing),
            "Diffuse" => Ok(Self::Diffuse),
            "LightMap" => Ok(Self::LightMap),
            _ => Err(format!("invalid name: {}", s)),
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
            current_shader: self.current_shader,
            ray_tracing_shader: self.ray_tracing_shader.clone(),
            diffuse_shader: self.diffuse_shader.clone(),
            light_map_shader: self.light_map_shader.clone(),
        }
    }
}

impl RayTracer {
    pub const SCENE_FILE_EXTENSION: &'static str = SceneFile::FILE_EXTENSION;
    fn new(builder: RayTracerBuilder) -> Self {
        Logger::init();
        let current_shader = builder.default_shader.unwrap_or(CurrentShader::Raytracing);
        let mut scenarios = world::get_scenarios();
        if let Some(mut add_scenarios) = builder.additional_scenarios {
            for (k, scenario) in add_scenarios.drain() {
                scenarios.items.insert(k, scenario);
            }
        }
        let world = match builder.default_scenario {
            LoadScenario::None => scenarios
                .items
                .get(&scenarios.default)
                .expect("scenario not found")
                .build(),
            LoadScenario::Prebuilt(key) => {
                if let Some(world) = scenarios.items.get(&key) {
                    world.build()
                } else {
                    error!("failed to load scenario: \"{}\" loading default", key);
                    scenarios.items[&scenarios.default].build()
                }
            }
            LoadScenario::Custom(info) => info.build_world(),
        };

        Self {
            scenarios: scenarios.items,
            world,
            ray_tracing_shader: RayTracingShader {},
            diffuse_shader: DiffuseShader {},
            light_map_shader: LightMapShader {},
            current_shader,
        }
    }
    pub fn builder() -> RayTracerBuilder {
        RayTracerBuilder::default()
    }
    pub fn get_info(&self) -> RayTracerInfo {
        RayTracerInfo {
            scenarios: self
                .scenarios
                .keys()
                .map(|name| ScenarioInfo { name: name.clone() })
                .collect(),
            loaded_entities: self.world.get_entity_info(),
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
    pub fn set_camera_data(&mut self, key: String, value: EntityField) {
        self.world.set_camera_data(key, value)
    }
    pub fn save_scene(&self, scene_path: std::path::PathBuf) {
        if let Err(e) = SceneFile::builder(scene_path).save(self) {
            error!("failed to save scene reason: {:?}", e)
        }
    }
    pub fn load_scene(path: std::path::PathBuf) -> Self {
        Self::builder()
            .custom_scenario(SceneFile::builder(path).load().unwrap())
            .build()
    }
    pub fn set_entity_data(&mut self, entity_index: usize, key: String, value: EntityField) {
        self.world.set_entity_data(entity_index, key, value);
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
                let u = (x as RayScalar + rand_scalar(0.0, 1.0)) / (total_width as RayScalar - 1.0);
                let v =
                    (y as RayScalar + rand_scalar(0.0, 1.0)) / (total_height as RayScalar - 1.0);
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

    pub fn threaded_render(self, image: ParallelImage) -> ParallelImageCollector {
        fn loop_try_get(lock: &'_ RwLock<RayTracer>) -> std::sync::RwLockReadGuard<'_, RayTracer> {
            loop {
                let t = lock.try_read();
                match t {
                    Ok(t) => return t,
                    Err(e) => match e {
                        std::sync::TryLockError::Poisoned(_) => {
                            panic!()
                        }
                        std::sync::TryLockError::WouldBlock => {
                            thread::sleep(std::time::Duration::from_millis(1))
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
                let mut render = true;
                let mut part = part;

                loop {
                    for msg in message_receiver.try_iter() {
                        match msg {
                            RayTracerMessage::SceneChanged => part.set_black(),

                            RayTracerMessage::SetShader(_) => part.set_black(),
                            RayTracerMessage::StopRendering => {
                                render = false;
                            }
                            RayTracerMessage::ContinueRendering => {
                                render = true;
                            }
                            RayTracerMessage::SetCameraData(_) => part.set_black(),
                        };
                    }
                    if render {
                        let self_read_res = loop_try_get(&self_rw_lock);
                        self_read_res.trace_part(&mut part);
                        sender.send(part.clone());
                    }
                }
            });
            receivers.push(receiver);
        }
        ParallelImageCollector::new(receivers, senders, self_rw_lock)
    }
}
pub struct RayTracerBuilder {
    additional_scenarios: Option<HashMap<String, Box<dyn ScenarioCtor>>>,
    default_scenario: LoadScenario,
    default_shader: Option<CurrentShader>,
}
impl std::default::Default for RayTracerBuilder {
    fn default() -> Self {
        Self {
            additional_scenarios: None,
            default_scenario: LoadScenario::None,
            default_shader: None,
        }
    }
}
impl RayTracerBuilder {
    pub fn add_scenarios(
        mut self,
        additional_scenarios: HashMap<String, Box<dyn ScenarioCtor>>,
    ) -> Self {
        if let Some(map) = self.additional_scenarios.as_mut() {
            for (key, value) in additional_scenarios {
                map.insert(key, value);
            }
        } else {
            self.additional_scenarios = Some(additional_scenarios)
        }
        self
    }
    pub fn set_scenario(mut self, default_scenario: String) -> Self {
        self.default_scenario = LoadScenario::Prebuilt(default_scenario);
        self
    }
    pub fn set_default_shader(mut self, shader: CurrentShader) -> Self {
        self.default_shader = Some(shader);
        self
    }
    pub fn custom_scenario(mut self, scenario: WorldInfo) -> Self {
        self.default_scenario = LoadScenario::Custom(Box::new(scenario));
        self
    }
    pub fn build(self) -> RayTracer {
        RayTracer::new(self)
    }
}
