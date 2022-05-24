mod background;
mod bvh;
mod camera;
mod hittable;
mod material;
mod pdf;
mod texture;
mod world;

use super::{prelude::*, vec_near_zero, Image};
use crate::reflect;

use background::{Background, ConstantColor, Sky};
use bvh::AABB;
use camera::Camera;
use cgmath::{InnerSpace, Point3, Vector3};
use hittable::{
    ConstantMedium, FlipNormals, HitRecord, Hittable, Light, MovingSphere, RayAreaInfo, RenderBox,
    RotateY, Sphere, Translate, XYRect,
};
use material::{Dielectric, DiffuseLight, Isotropic, Lambertian, Material, Metal};
use pdf::{CosinePdf, LightPdf, PdfList, ScatterRecord};
use texture::{CheckerTexture, DebugV, ImageTexture, Perlin, SolidColor, Texture};
use world::World;

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
        if let Some(emitted) = record.material.borrow().emmit(&record) {
            return emitted;
        };
        if let Some(scatter_record) = record.material.borrow().scatter(ray, &record) {
            if let Some(specular_ray) = scatter_record.specular_ray {
                scatter_record.attenuation * ray_color(specular_ray, world, depth - 1)
            } else {
                if let Some((pdf_direction, value)) = scatter_record
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
                    if debug() {
                        println!("scattering pdf: {}", scattering_pdf);
                    }
                    let value = value / scattering_pdf;
                    let temp_c = scatter_record.attenuation
                        * ray_color(
                            Ray {
                                origin: record.position,
                                direction: pdf_direction,
                                time: record.t,
                            },
                            world,
                            depth - 1,
                        )
                        / value;
                    if debug() {
                        println!("out color: {}", temp_c);
                    }
                    return temp_c;
                } else {
                    if debug() {
                        println!("pdf not generated!");
                    }
                    RgbColor::BLACK
                }
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
}

impl RayTracer {
    const SAMPLES_PER_PIXEL: usize = 1000;
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

        //  let (world, camera) = world::easy_cornell_box();
        let (world, camera) = world::cornell_smoke();
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
