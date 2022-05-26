mod background;
mod bvh;
mod camera;
mod hittable;
mod logger;
mod material;
mod pdf;
mod texture;
mod world;

use super::{prelude::*, Image};
use crate::reflect;

use background::{Background, ConstantColor, Sky};
use bvh::Aabb;
use camera::Camera;
use cgmath::{InnerSpace, Vector3};
#[allow(unused_imports)]
use hittable::{
    ConstantMedium, FlipNormals, HitRecord, Hittable, Light, MovingSphere, RayAreaInfo, RenderBox,
    RotateY, Sphere, Translate, XYRect, XZRect, YZRect,
};
#[allow(unused_imports)]
use material::{Dielectric, DiffuseLight, Isotropic, Lambertian, Material, Metal};
use pdf::{CosinePdf, LightPdf, PdfList, ScatterRecord};
#[allow(unused_imports)]
use texture::{CheckerTexture, DebugV, ImageTexture, Perlin, SolidColor, Texture};
use world::World;

use crate::ray_tracer::world::Scenario;
use std::collections::HashMap;
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Instant,
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
        if let Some(emitted) = record.material.borrow().emmit(&record) {
            return emitted;
        };
        if let Some(scatter_record) = record.material.borrow().scatter(ray, &record) {
            if let Some(specular_ray) = scatter_record.specular_ray {
                scatter_record.attenuation * ray_color(specular_ray, world, depth - 1)
            } else if let Some((pdf_direction, value)) = scatter_record
                .pdf
                .expect("if material is not specular there should be a pdf")
                .generate(ray, record.position, world)
            {
                let scattering_pdf = record.material.borrow().scattering_pdf(
                    ray,
                    &record,
                    Ray {
                        origin: record.position,
                        direction: pdf_direction,
                        time: record.t,
                    },
                );

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
            // emitted
            RgbColor::BLACK
        }
    } else {
        world.background.color(ray)
    }
}

pub struct RayTracer {
    sender: Sender<Image>,
    msg_reciever: Receiver<Message>,
    num_samples: usize,
    scenarios: HashMap<String, Scenario>,
}
pub struct RayTracerInfo {
    pub scenarios: Vec<String>,
}

pub enum Message {
    LoadScenario(String),
}
impl RayTracer {
    const SAMPLES_PER_PIXEL: usize = 1000;
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> (Receiver<Image>, Sender<Message>, RayTracerInfo) {
        let (sender, recvier) = channel();
        let (message_sender, msg_reciever) = channel();
        let scenarios = world::get_scenarios();
        let scenario_names = scenarios.keys().cloned().collect();
        let s = Self {
            sender,
            msg_reciever,
            num_samples: 0,
            scenarios,
        };
        thread::spawn(move || s.start_tracing());
        (
            recvier,
            message_sender,
            RayTracerInfo {
                scenarios: scenario_names,
            },
        )
    }
    fn tracing_loop(
        &self,
        world: &World,
        camera: &Camera,
        rgb_img: &mut RgbImage,
        num_samples: usize,
    ) {
        for x in 0..IMAGE_WIDTH {
            for y in 0..IMAGE_WIDTH {
                let u = (x as f32 + rand_f32(0.0, 1.0)) / (IMAGE_WIDTH as f32 - 1.0);
                let v = (y as f32 + rand_f32(0.0, 1.0)) / (IMAGE_HEIGHT as f32 - 1.0);
                let r = camera.get_ray(u, v);

                rgb_img.add_xy(x, y, ray_color(r, &world, 50));
            }
        }

        self.sender
            .send(Image::from_rgb_image(
                &(rgb_img.clone() / num_samples as f32),
            ))
            .expect("channel failed");
    }
    pub fn start_tracing(&self) {
        self.sender
            .send(Image::from_fn(
                |_x, _y| [0, 0, 0, 0xff],
                IMAGE_WIDTH,
                IMAGE_HEIGHT,
            ))
            .expect("failed to send");

        let (mut world, mut camera) = world::cornell_smoke();
        let mut world = world.into_bvh(camera.start_time(), camera.end_time());
        println!(
            "world bounding box: {:#?}",
            world.spheres[0].bounding_box(0.0, 0.0)
        );

        let mut rgb_img = RgbImage::new_black(1000, 1000);
        let total_time = Instant::now();
        let mut num_samples = 1usize;
        loop {
            if let Ok(message) = self.msg_reciever.try_recv() {
                match message {
                    Message::LoadScenario(scenario) => {
                        if let Some(scenario) = self.scenarios.get(&scenario) {
                            (world, camera) = (scenario.ctor)();
                            world = world.into_bvh(camera.start_time(), camera.end_time());
                            rgb_img = RgbImage::new_black(1000, 1000);
                            //camera = t_camera;
                            num_samples = 1;
                        } else {
                            todo!("error handling, invalid scenario");
                        }

                        println!("todo: load {}", scenario)
                    }
                }
            }
            self.tracing_loop(&world, &camera, &mut rgb_img, num_samples);
            let average_time_s = total_time.elapsed().as_secs_f32() / (num_samples) as f32;
            println!("average time per frame: {} (s)", average_time_s);
            num_samples += 1;
        }
    }
}
