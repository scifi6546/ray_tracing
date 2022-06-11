mod cornell_box;
mod cornell_smoke;
mod easy_cornell_box;
mod easy_scene;
mod one_sphere;
mod random_scene;
mod two_spheres;
use super::{
    bvh::BvhNode, hittable::*, material::*, texture::*, Background, Camera, ConstantColor,
    HitRecord, Hittable, Sky, IMAGE_HEIGHT, IMAGE_WIDTH,
};
use crate::prelude::*;
use cgmath::{Point3, Vector3};
pub use cornell_box::cornell_box;
pub use cornell_smoke::cornell_smoke;
pub use easy_cornell_box::easy_cornell_box;
pub use easy_scene::easy_scene;
pub use one_sphere::one_sphere;
pub use random_scene::random_scene;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub use two_spheres::two_spheres;

pub struct World {
    pub spheres: Vec<Rc<dyn Hittable>>,
    pub lights: Vec<Rc<dyn Hittable>>,
    pub background: Box<dyn Background>,
    pub camera: Camera,
}
impl World {
    pub fn from_scene(scene: &base_lib::Scene) -> Self {
        let objects_temp = scene
            .objects
            .iter()
            .map(|obj| {
                let material: Rc<RefCell<dyn Material>> = match &obj.material {
                    base_lib::Material::Light(tex) => Rc::new(RefCell::new(DiffuseLight {
                        emit: match tex {
                            base_lib::Texture::ConstantColor(color) => {
                                Box::new(SolidColor { color: *color })
                            }
                        },
                    })),
                };
                let obj_out: Rc<dyn Hittable> = match obj.shape {
                    base_lib::Shape::Sphere{radius,origin} => Rc::new(Sphere {
                        radius,
                        origin,
                        material,
                    }),
                };
                (
                    match obj.material {
                        base_lib::Material::Light(..) => true,
                    },
                    obj_out,
                )
            })
            .collect::<Vec<(bool,Rc<dyn Hittable>)>>();
        let lights = objects_temp
            .iter()
            .filter(|(is_light, _obj)| *is_light)
            .map(|(_is_light, obj)| obj.clone())
            .collect::<Vec<_>>();
        let spheres = objects_temp
            .iter()
            .map(|(_is_light, obj)| obj.clone())
            .collect::<Vec<Rc<dyn Hittable>>>();
        let background = match scene.background {
            base_lib::Background::Sky => Box::new(Sky {}),
        };
        World {
            spheres,
            lights,
            background,
            camera: Camera::new(
                scene.camera.aspect_ratio,
                scene.camera.fov,
                scene.camera.origin,
                scene.camera.look_at,
                scene.camera.up_vector,
                scene.camera.aperture,
                scene.camera.focus_distance,
                scene.camera.start_time,
                scene.camera.end_time,
            ),
        }
    }
    pub fn nearest_light_hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(Rc<dyn Hittable>, HitRecord)> {
        self.lights
            .iter()
            .map(|light| (light.clone(), light.hit(ray, t_min, t_max)))
            .filter(|(_light, hit_opt)| hit_opt.is_some())
            .map(|(light, hit_opt)| (light, hit_opt.unwrap()))
            .reduce(|acc, x| if acc.1.t < x.1.t { acc } else { x })
    }

    pub fn nearest_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.spheres
            .iter()
            .filter_map(|s| s.hit(ray, t_min, t_max))
            .reduce(|acc, x| if acc.t < x.t { acc } else { x })
    }
    pub fn into_bvh(self) -> Self {
        let sphere_len = self.spheres.len();
        Self {
            spheres: vec![Rc::new(BvhNode::new(
                self.spheres,
                0,
                sphere_len,
                self.camera.start_time(),
                self.camera.end_time(),
            ))],
            lights: self.lights.clone(),
            background: self.background,
            camera: self.camera,
        }
    }
}
#[derive(Clone)]
pub struct Scenario {
    pub name: String,
    pub ctor: fn() -> World,
}
pub fn get_scenarios() -> HashMap<String, Scenario> {

    [
        Scenario {
            name: "Cornell Box".to_string(),
            ctor: cornell_box,
        },
        Scenario {
            name: "Cornell Smoke".to_string(),
            ctor: cornell_smoke,
        },
        Scenario {
            name: "Easy Cornell Box".to_string(),
            ctor: easy_cornell_box,
        },
        Scenario {
            name: "Easy Scene".to_string(),
            ctor: easy_scene,
        },
        Scenario {
            name: "One Sphere".to_string(),
            ctor: one_sphere,
        },
        Scenario {
            name: "Random Scene".to_string(),
            ctor: random_scene,
        },
        Scenario {
            name: "Two Sphere".to_string(),
            ctor: two_spheres,
        },
        Scenario {
            name: "test baselib scene".to_string(),
            ctor: || {
                let s = base_lib::Scene {
                    name: "test baselib scene".to_string(),
                    objects: vec![base_lib::Object {
                        shape: base_lib::Shape::Sphere {
                            radius: 0.5,
                            origin: Point3::new(0.0, 0.0, 0.0),
                        },
                        material: base_lib::Material::Light(base_lib::Texture::ConstantColor(
                            RgbColor::new(200000000000.0,0.0,0.0),
                        )),
                    }],
                    background: base_lib::Background::Sky,
                    camera: base_lib::Camera {
                        aspect_ratio: IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32,
                        fov: 20.0,
                        origin: Point3::new(10.0, 10.0, 10.0),
                        look_at: Point3::new(0.0, 0.0, 0.0),
                        up_vector: Vector3::new(0.0, 1.0, 0.0),
                        aperture: 0.00001,
                        focus_distance: 10.0,
                        start_time: 0.0,
                        end_time: 0.0,
                    },
                };
                World::from_scene(&s)
            },
        },
    ]
    .iter()
    .cloned()
    .map(|scenario| (scenario.name.clone(), scenario))
    .collect()
}
