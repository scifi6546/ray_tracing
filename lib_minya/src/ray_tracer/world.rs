mod cornell_smoke;
mod demo;
mod dielectric;

mod cube_world;
mod cube_world_big;
mod easy_cornell_box;
mod easy_scene;
mod empty_scene;
mod light_demo;
mod one_sphere;
mod random_scene;
mod two_spheres;

use super::{
    bvh::BvhTree, hittable::*, material::*, texture::*, Background, Camera, ConstantColor,
    HitRecord, Hittable, Sky,
};
use crate::prelude::*;
use cgmath::Point3;
use dyn_clone::clone_box;

pub use cornell_smoke::cornell_smoke;
pub use easy_cornell_box::easy_cornell_box;
pub use easy_scene::easy_scene;
pub use one_sphere::one_sphere;
pub use random_scene::random_scene;

use std::{collections::HashMap, ops::Deref};

pub use two_spheres::two_spheres;

pub struct WorldInfo {
    pub objects: Vec<Object>,
    pub lights: Vec<Object>,
    pub background: Box<dyn Background>,
    pub camera: Camera,
}
impl WorldInfo {
    pub fn build_world(self) -> World {
        World {
            bvh: BvhTree::new(
                self.objects,
                self.camera.start_time(),
                self.camera.end_time(),
            ),
            lights: self.lights.clone(),
            background: self.background,
            camera: self.camera,
        }
    }
}
pub struct World {
    pub bvh: BvhTree,
    pub lights: Vec<Object>,
    pub background: Box<dyn Background>,
    pub camera: Camera,
}
impl World {
    pub fn from_baselib_scene(scene: &base_lib::Scene) -> Self {
        let objects_temp = scene
            .objects
            .iter()
            .map(|obj| {
                let material: Box<dyn Material> = match &obj.material {
                    base_lib::Material::Light(tex) => Box::new(DiffuseLight {
                        emit: match tex {
                            base_lib::Texture::ConstantColor(color) => {
                                Box::new(SolidColor { color: *color })
                            }
                        },
                    }),
                    base_lib::Material::Lambertian(tex) => Box::new(Lambertian {
                        albedo: match tex {
                            base_lib::Texture::ConstantColor(color) => {
                                Box::new(SolidColor { color: *color })
                            }
                        },
                    }),
                };
                let obj_out: Box<dyn Hittable> = match obj.shape {
                    base_lib::Shape::Sphere { radius, origin } => Box::new(Sphere {
                        radius,
                        origin,
                        material,
                    }),

                    base_lib::Shape::XYRect {
                        center,
                        size_x,
                        size_y,
                    } => Box::new(XYRect::new(
                        center.x - size_x,
                        center.x + size_x,
                        center.y - size_y,
                        center.y + size_y,
                        center.z,
                        material,
                        false,
                    )),

                    base_lib::Shape::YZRect {
                        center,
                        size_y,
                        size_z,
                    } => Box::new(YZRect::new(
                        center.y - size_y,
                        center.y + size_y,
                        center.z - size_z,
                        center.z + size_z,
                        center.x,
                        material,
                        false,
                    )),
                    base_lib::Shape::XZRect {
                        center,
                        size_x,
                        size_z,
                    } => Box::new(XZRect::new(
                        center.x - size_x,
                        center.x + size_x,
                        center.z - size_z,
                        center.z + size_z,
                        center.y,
                        material,
                        false,
                    )),
                    base_lib::Shape::RenderBox {
                        center,
                        size_x,
                        size_y,
                        size_z,
                    } => Box::new(RenderBox::new(
                        Point3::new(center.x - size_x, center.y - size_y, center.z - size_z),
                        Point3::new(center.x + size_x, center.y + size_y, center.z + size_z),
                        material,
                    )),
                };
                let mut obj_out = obj_out;
                for modifier in obj.modifiers.iter() {
                    match modifier {
                        base_lib::Modifiers::FlipNormals => {
                            obj_out = todo!();
                        }
                    }
                }
                (
                    match obj.material {
                        base_lib::Material::Light(..) => true,
                        base_lib::Material::Lambertian(_) => false,
                    },
                    obj_out,
                )
            })
            .collect::<Vec<(bool, Box<dyn Hittable>)>>();
        let lights = objects_temp
            .iter()
            .filter(|(is_light, _obj)| *is_light)
            .map(|(_is_light, obj)| Object::new(clone_box(obj.deref()), Transform::identity()))
            .collect::<Vec<_>>();
        let spheres = objects_temp
            .iter()
            .map(|(_is_light, obj)| Object::new(clone_box(obj.deref()), Transform::identity()))
            .collect::<_>();
        let background: Box<dyn Background> = match scene.background {
            base_lib::Background::Sky => Box::new(Sky::default()),
            base_lib::Background::ConstantColor(color) => Box::new(ConstantColor { color }),
        };

        Self {
            bvh: BvhTree::new(spheres, scene.camera.start_time, scene.camera.end_time),
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
    ) -> Option<(Object, HitRecord)> {
        self.lights
            .iter()
            .map(|light| (light.clone(), light.hit(ray, t_min, t_max)))
            .filter(|(_light, hit_opt)| hit_opt.is_some())
            .map(|(light, hit_opt)| (light, hit_opt.unwrap()))
            .reduce(|acc, x| if acc.1.t < x.1.t { acc } else { x })
    }

    pub fn nearest_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.bvh.hit(ray, t_min, t_max)
    }
}
pub trait ScenarioCtor: Send {
    fn build(&self) -> World;
    fn name(&self) -> String;
}
pub struct ScenarioFn {
    f: fn() -> WorldInfo,
    name: String,
}
impl ScenarioCtor for ScenarioFn {
    fn build(&self) -> World {
        (self.f)().build_world()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
struct BaselibScenario {
    ctor: fn() -> base_lib::Scene,
    name: String,
}
impl ScenarioCtor for BaselibScenario {
    fn build(&self) -> World {
        World::from_baselib_scene(&(self.ctor)())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
pub struct Scenarios {
    pub items: HashMap<String, Box<dyn ScenarioCtor>>,
    pub default: String,
}
pub fn get_scenarios() -> Scenarios {
    let mut scenes: Vec<Box<dyn ScenarioCtor>> = vec![
        Box::new(ScenarioFn {
            name: "Cornell Smoke".to_string(),
            f: cornell_smoke,
        }),
        Box::new(ScenarioFn {
            name: "Easy Cornell Box".to_string(),
            f: easy_cornell_box,
        }),
        Box::new(ScenarioFn {
            name: "Easy Scene".to_string(),
            f: easy_scene,
        }),
        Box::new(ScenarioFn {
            name: "One Sphere".to_string(),
            f: one_sphere,
        }),
        Box::new(ScenarioFn {
            name: "Random Scene".to_string(),
            f: random_scene,
        }),
        Box::new(ScenarioFn {
            name: "Two Sphere".to_string(),
            f: two_spheres,
        }),
        Box::new(ScenarioFn {
            name: "Lambertian Demonstration".to_string(),
            f: demo::lambertian::demo,
        }),
        Box::new(ScenarioFn {
            name: "Metallic Demonstration Smooth".to_string(),
            f: demo::metalic_demo::metallic_smooth,
        }),
        Box::new(ScenarioFn {
            name: "Metallic Demonstration Rough".to_string(),
            f: demo::metalic_demo::metallic_rough,
        }),
        Box::new(ScenarioFn {
            name: "Dielectric Demonstration, Low Refraction".to_string(),
            f: dielectric::dielectric_no_refraction,
        }),
        Box::new(ScenarioFn {
            name: "Dielectric Demonstration, High Refraction".to_string(),
            f: dielectric::dielectric_refraction,
        }),
        Box::new(ScenarioFn {
            name: "Light Demonstration".to_string(),
            f: light_demo::light_demo,
        }),
        Box::new(ScenarioFn {
            name: "Cube Field".to_string(),
            f: demo::cube_field::build_field,
        }),
        Box::new(ScenarioFn {
            name: "Cube World".to_string(),
            f: cube_world::cube_world,
        }),
        Box::new(ScenarioFn {
            name: "Cube World Big".to_string(),
            f: cube_world_big::cube_world_big,
        }),
        Box::new(ScenarioFn {
            name: "Empty Scene".to_string(),
            f: empty_scene::empty_scene,
        }),
    ];
    let mut map: HashMap<String, Box<dyn ScenarioCtor>> = scenes
        .drain(..)
        .map(|scenario| (scenario.name(), scenario))
        .collect::<HashMap<String, _>>();
    for (name, scene) in base_lib::get_scenarios() {
        let ctor = Box::new(BaselibScenario {
            ctor: scene,
            name: name.clone(),
        });
        map.insert(name, ctor);
    }
    Scenarios {
        items: map,
        default: "Cube World Big".to_string(),
    }
}
