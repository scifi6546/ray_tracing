mod cornell_box;
mod cornell_smoke;
mod easy_cornell_box;
mod easy_scene;
mod one_sphere;
mod random_scene;
mod two_spheres;
use super::{
    bvh::BvhNode, hittable::*, material::*, texture::*, Background, Camera, ConstantColor,
    FlipNormals, HitRecord, Hittable, Sky, IMAGE_HEIGHT, IMAGE_WIDTH,
};
use crate::prelude::*;
use cgmath::{Point3, Vector3};
pub use cornell_box::cornell_box;
pub use cornell_smoke::cornell_smoke;
pub use easy_cornell_box::easy_cornell_box;
pub use easy_scene::easy_scene;
pub use one_sphere::one_sphere;
pub use random_scene::random_scene;
use std::ffi::OsString;
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
                    base_lib::Material::Lambertian(tex) => Rc::new(RefCell::new(Lambertian {
                        albedo: match tex {
                            base_lib::Texture::ConstantColor(color) => {
                                Box::new(SolidColor { color: *color })
                            }
                        },
                    })),
                };
                let obj_out: Rc<dyn Hittable> = match obj.shape {
                    base_lib::Shape::Sphere { radius, origin } => Rc::new(Sphere {
                        radius,
                        origin,
                        material,
                    }),
                    base_lib::Shape::XYRect {
                        center,
                        size_x,
                        size_y,
                    } => Rc::new(XYRect {
                        material,
                        x0: center.x - size_x,
                        x1: center.x + size_x,
                        y0: center.y - size_y,
                        y1: center.y + size_y,
                        k: center.z,
                    }),
                    base_lib::Shape::YZRect {
                        center,
                        size_y,
                        size_z,
                    } => Rc::new(YZRect {
                        material,
                        y0: center.y - size_y,
                        y1: center.y + size_y,
                        z0: center.z - size_z,
                        z1: center.z + size_z,
                        k: center.x,
                    }),
                    base_lib::Shape::XZRect {
                        center,
                        size_x,
                        size_z,
                    } => Rc::new(XZRect {
                        material,
                        x0: center.x - size_x,
                        x1: center.x + size_x,
                        z0: center.z - size_z,
                        z1: center.z + size_z,
                        k: center.y,
                    }),
                    base_lib::Shape::RenderBox {
                        center,
                        size_x,
                        size_y,
                        size_z,
                    } => Rc::new(RenderBox::new(
                        Point3::new(center.x - size_x, center.y - size_y, center.z - size_z),
                        Point3::new(center.x + size_x, center.y + size_y, center.z + size_z),
                        material,
                    )),
                };
                let mut obj_out = obj_out;
                for modifier in obj.modifiers.iter() {
                    match modifier {
                        base_lib::Modifiers::FlipNormals => {
                            obj_out = Rc::new(FlipNormals { item: obj_out });
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
            .collect::<Vec<(bool, Rc<dyn Hittable>)>>();
        let lights = objects_temp
            .iter()
            .filter(|(is_light, _obj)| *is_light)
            .map(|(_is_light, obj)| obj.clone())
            .collect::<Vec<_>>();
        let spheres = objects_temp
            .iter()
            .map(|(_is_light, obj)| obj.clone())
            .collect::<Vec<Rc<dyn Hittable>>>();
        let background: Box<dyn Background> = match scene.background {
            base_lib::Background::Sky => Box::new(Sky {}),
            base_lib::Background::ConstantColor(color) => Box::new(ConstantColor { color }),
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
pub trait ScenarioCtor {
    fn build(&self) -> World;
    fn name(&self) -> String;
}
pub struct ScenarioFn {
    f: fn() -> World,
    name: String,
}
struct BaselibScenario {
    ctor: fn() -> base_lib::Scene,
    name: String,
}
impl ScenarioCtor for BaselibScenario {
    fn build(&self) -> World {
        World::from_scene(&(self.ctor)())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
impl ScenarioCtor for ScenarioFn {
    fn build(&self) -> World {
        (self.f)()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
pub fn get_scenarios() -> HashMap<String, Box<dyn ScenarioCtor>> {
    let mut scenes: Vec<Box<dyn ScenarioCtor>> = vec![
        Box::new(ScenarioFn {
            name: "Cornell Box".to_string(),
            f: cornell_box,
        }),
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
    map
}
