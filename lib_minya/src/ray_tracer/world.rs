mod cornell_smoke;
mod demo;
mod dielectric;

mod cube_world;
mod cube_world_big;
mod easy_cornell_box;
mod easy_scene;
mod empty_scene;
mod light_demo;
mod load_vox_model;
mod oct_tree_world;
mod one_sphere;
mod random_scene;
mod sinnoh;
mod translucent_cubeworld;
mod two_spheres;
mod voxel_city;
mod voxel_city_big;

use super::sun::Sun;
use super::{
    background::{Sky, SunSky},
    bvh::BvhTree,
    camera::{Camera, CameraInfo},
    hittable::hittable_objects,
    hittable::*,
    material::*,
    ray_tracer_info::{EntityField, WorldEntityCollection},
    texture::*,
    Background, ConstantColor, HitRecord, Hittable,
};

mod world_prelude {
    pub(crate) use super::super::background::SunSky;
    pub(crate) use super::super::hittable::voxel_world::{
        CubeMaterial, CubeMaterialIndex, PerlinBuilder, VoxelWorld,
    };
    pub use super::super::sun::Sun;
}
use crate::prelude::*;
use cgmath::Point3;
use dyn_clone::{clone_box, DynClone};

pub use cornell_smoke::cornell_smoke;
pub use easy_cornell_box::easy_cornell_box;
pub use easy_scene::easy_scene;

pub use one_sphere::one_sphere;
pub use random_scene::random_scene;
use std::{collections::HashMap, ops::Deref};

use crate::ray_tracer::hittable::voxel_world::CubeMaterialIndex;

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
            sun: self.sun.clone(),
        }
    }
}
impl World {
    pub fn from_baselib_scene(scene: &base_lib::Scene) -> Self {
        let objects_temp = scene
            .objects
            .iter()
            .map(|obj| {
                let material: Box<dyn Material + Send> = match &obj.material {
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
                let obj_out: Box<dyn Hittable + Send> = match &obj.shape {
                    base_lib::Shape::Sphere { radius, origin } => Box::new(Sphere {
                        radius: *radius as RayScalar,
                        origin: origin.map(|v| v as RayScalar),
                        material,
                    }),

                    base_lib::Shape::XYRect {
                        center,
                        size_x,
                        size_y,
                    } => Box::new(XYRect::new(
                        (center.x - size_x) as RayScalar,
                        (center.x + size_x) as RayScalar,
                        (center.y - size_y) as RayScalar,
                        (center.y + size_y) as RayScalar,
                        center.z as RayScalar,
                        material,
                        false,
                    )),

                    base_lib::Shape::YZRect {
                        center,
                        size_y,
                        size_z,
                    } => Box::new(YZRect::new(
                        (center.y - size_y) as RayScalar,
                        (center.y + size_y) as RayScalar,
                        (center.z - size_z) as RayScalar,
                        (center.z + size_z) as RayScalar,
                        (center.x) as RayScalar,
                        material,
                        false,
                    )),
                    base_lib::Shape::XZRect {
                        center,
                        size_x,
                        size_z,
                    } => Box::new(XZRect::new(
                        (center.x - size_x) as RayScalar,
                        (center.x + size_x) as RayScalar,
                        (center.z - size_z) as RayScalar,
                        (center.z + size_z) as RayScalar,
                        center.y as RayScalar,
                        material,
                        false,
                    )),
                    base_lib::Shape::RenderBox {
                        center,
                        size_x,
                        size_y,
                        size_z,
                    } => Box::new(RenderBox::new(
                        Point3::new(
                            (center.x - size_x) as RayScalar,
                            (center.y - size_y) as RayScalar,
                            (center.z - size_z) as RayScalar,
                        ),
                        Point3::new(
                            (center.x + size_x) as RayScalar,
                            (center.y + size_y) as RayScalar,
                            (center.z + size_z) as RayScalar,
                        ),
                        material,
                    )),
                    base_lib::Shape::Voxels(voxel_grid) => {
                        let solid_materials = match &obj.material {
                            base_lib::Material::Lambertian(texture) => match texture {
                                base_lib::Texture::ConstantColor(c) => {
                                    vec![world_prelude::CubeMaterial::new(*c)]
                                }
                            },
                            _ => panic!("invalid material: {:#?}", obj.material),
                        };
                        let mut voxel_world = world_prelude::VoxelWorld::new(
                            solid_materials,
                            Vec::new(),
                            voxel_grid.size_x() as i32,
                            voxel_grid.size_y() as i32,
                            voxel_grid.size_z() as i32,
                        );
                        for x in 0..voxel_grid.size_x() {
                            for y in 0..voxel_grid.size_y() {
                                for z in 0..voxel_grid.size_z() {
                                    if voxel_grid.get_tile(Point3::new(x, y, z)) {
                                        voxel_world.update(
                                            x as isize,
                                            y as isize,
                                            z as isize,
                                            CubeMaterialIndex::Solid { index: 0 },
                                        );
                                    }
                                }
                            }
                        }
                        Box::new(voxel_world)
                    }
                };
                let obj_out = obj_out;
                for modifier in obj.modifiers.iter() {
                    match modifier {
                        base_lib::Modifiers::FlipNormals => {
                            todo!();
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
            .collect::<Vec<(bool, Box<dyn Hittable + Send>)>>();
        let lights = objects_temp
            .iter()
            .filter(|(is_light, _obj)| *is_light)
            .map(|(_is_light, obj)| Object::new(clone_box(obj.deref()), Transform::identity()))
            .collect::<Vec<_>>();
        let spheres = objects_temp
            .iter()
            .map(|(_is_light, obj)| Object::new(clone_box(obj.deref()), Transform::identity()))
            .collect::<_>();
        let background: Box<dyn Background + Send> = match scene.background {
            base_lib::Background::Sky => Box::new(Sky::default()),
            base_lib::Background::ConstantColor(color) => Box::new(ConstantColor { color }),
        };

        Self {
            bvh: BvhTree::new(
                spheres,
                scene.camera.start_time as RayScalar,
                scene.camera.end_time as RayScalar,
            ),
            lights,
            background,
            camera: Camera::new(CameraInfo {
                aspect_ratio: scene.camera.aspect_ratio as RayScalar,
                fov: scene.camera.fov as RayScalar,
                origin: scene.camera.origin.map(|v| v as RayScalar),
                look_at: scene.camera.look_at.map(|v| v as RayScalar),
                up_vector: scene.camera.up_vector.map(|v| v as RayScalar),
                aperture: scene.camera.aperture as RayScalar,
                focus_distance: scene.camera.focus_distance as RayScalar,
                start_time: scene.camera.start_time as RayScalar,
                end_time: scene.camera.end_time as RayScalar,
            }),
            sun: None,
        }
    }

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
#[derive(Clone)]
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
        Box::new(ScenarioFn {
            name: "Voxel City".to_string(),
            f: voxel_city::voxel_city,
        }),
        Box::new(ScenarioFn {
            name: "Voxel City Big".to_string(),
            f: voxel_city_big::voxel_city_big,
        }),
        Box::new(ScenarioFn {
            name: "Translucent Cube World".to_string(),
            f: translucent_cubeworld::translucent_cube_world,
        }),
        Box::new(ScenarioFn {
            name: "Load Voxel Model".to_string(),
            f: load_vox_model::load_vox_model,
        }),
        Box::new(ScenarioFn {
            name: "Twinleaf Town Map".to_string(),
            f: sinnoh::twinleaf_map,
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
            name: "Oct Tree Sinnoh Test".to_string(),
            f: oct_tree_world::compare_voxel_world::sinnoh,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Cube Test".to_string(),
            f: oct_tree_world::compare_voxel_world::simple_cube,
        }),
        Box::new(ScenarioFn {
            name: "Oct Tree Cube Recreation".to_string(),
            f: oct_tree_world::compare_voxel_world::cube_recreation,
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
        default: "Twinleaf Town Map".to_string(),
    }
}
