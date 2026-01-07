mod cornell_smoke;
mod demo;
mod dielectric;

mod easy_cornell_box;
mod easy_scene;
mod empty_scene;
mod light_demo;
mod oct_tree_world;
mod one_sphere;
mod random_scene;
mod two_spheres;

use super::sun::Sun;
use super::{
    background::{Sky, SunSky},
    bvh::BvhTree,
    camera::{Camera, CameraInfo},
    hittable::*,
    material::*,
    ray_tracer_info::{EntityField, WorldEntityCollection},
    texture::*,
    Background, ConstantColor, HitRecord, Hittable,
};

mod world_prelude {

    pub(crate) use super::super::{
        background::Sky,
        camera::{Camera, CameraInfo},
        hittable::Transform,
    };
}
use crate::prelude::*;

use dyn_clone::{clone_box, DynClone};

pub use cornell_smoke::cornell_smoke;
pub use easy_cornell_box::easy_cornell_box;
pub use easy_scene::easy_scene;

pub use one_sphere::one_sphere;
pub use random_scene::random_scene;
use std::collections::HashMap;

use crate::ray_tracer::ray_tracer_info::Entity;
pub use two_spheres::two_spheres;

pub struct WorldInfo {
    pub objects: Vec<Object>,
    pub lights: Vec<Object>,
    pub background: Box<dyn Background + Send>,
    pub camera: Camera,
    pub sun: Option<Sun>,
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
            sun: self.sun,
        }
    }
}

pub struct World {
    pub bvh: BvhTree,
    pub lights: Vec<Object>,
    pub background: Box<dyn Background + Send>,
    pub camera: Camera,
    pub sun: Option<Sun>,
}
impl Clone for World {
    fn clone(&self) -> Self {
        Self {
            bvh: self.bvh.clone(),
            lights: self.lights.clone(),
            background: clone_box(&*self.background),
            camera: self.camera.clone(),
            sun: self.sun,
        }
    }
}
impl World {
    pub fn nearest_light_hit(
        &self,
        ray: &Ray,
        t_min: RayScalar,
        t_max: RayScalar,
    ) -> Option<(Object, HitRecord)> {
        self.lights
            .iter()
            .map(|light| (light.clone(), light.hit(ray, t_min, t_max)))
            .filter(|(_light, hit_opt)| hit_opt.is_some())
            .map(|(light, hit_opt)| (light, hit_opt.unwrap()))
            .reduce(|acc, x| if acc.1.t < x.1.t { acc } else { x })
    }

    pub fn nearest_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.bvh.hit(ray, t_min as RayScalar, t_max as RayScalar)
    }
    pub fn get_entity_info(&self) -> WorldEntityCollection {
        WorldEntityCollection {
            main_camera: self.camera.clone(),
            entities: self.bvh.get_info(),
        }
    }

    pub fn set_camera_data(&mut self, key: String, value: EntityField) {
        self.camera.set_field(key, value);
    }
    pub fn set_entity_data(&mut self, index: usize, key: String, value: EntityField) {
        self.bvh.update_entity(index, key, value)
    }
}
pub trait ScenarioCtor: Send + Sync + DynClone {
    fn build(&self) -> World;
    fn name(&self) -> String;
}
#[derive(Clone)]
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
            name: "Empty Scene".to_string(),
            f: empty_scene::empty_scene,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Sphere".to_string(),
            f: oct_tree_world::basic_sphere,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Temple".to_string(),
            f: oct_tree_world::temple,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Below".to_string(),
            f: oct_tree_world::temple_below,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Cube".to_string(),
            f: oct_tree_world::cube,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Back".to_string(),
            f: oct_tree_world::cube_back,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Sinnoh".to_string(),
            f: oct_tree_world::compare_voxel_world::sinnoh_direct,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Cube Test".to_string(),
            f: oct_tree_world::compare_voxel_world::simple_cube,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Cube Recreation".to_string(),
            f: oct_tree_world::compare_voxel_world::cube_recreation,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Translucent".to_string(),
            f: oct_tree_world::volume::oct_tree_volume,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Translucent Two Density".to_string(),
            f: oct_tree_world::volume::oct_tree_volume_two_density,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Translucent Many Density".to_string(),
            f: oct_tree_world::volume::oct_tree_volume_many_density,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Translucent Lambertian".to_string(),
            f: oct_tree_world::volume::oct_tree_volume_lambertian,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Volume Metal".to_string(),
            f: oct_tree_world::volume::oct_tree_volume_metal,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Volume Ice".to_string(),
            f: oct_tree_world::volume::oct_tree_volume_ice,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree GoldCube".to_string(),
            f: oct_tree_world::metal::gold_cube,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Load Model".to_string(),
            f: oct_tree_world::load_voxel_model,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Cube World".to_string(),
            f: oct_tree_world::cube_world,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Explosion".to_string(),
            f: oct_tree_world::explosion,
        }),
    ];
    let map: HashMap<String, Box<dyn ScenarioCtor>> = scenes
        .drain(..)
        .map(|scenario| (scenario.name(), scenario))
        .collect::<HashMap<String, _>>();

    Scenarios {
        items: map,
        default: "One Sphere".to_string(),
    }
}
