mod cornell_box;
mod cornell_smoke;
mod easy_cornell_box;
mod easy_scene;
mod one_sphere;
mod random_scene;
mod two_spheres;

use super::{
    bvh::BvhNode, hittable::*, material::*, texture::*, Background, Camera, ConstantColor,
    HitRecord, Hittable, Light, Sky, IMAGE_HEIGHT, IMAGE_WIDTH,
};
use crate::prelude::*;
pub use cornell_box::cornell_box;
pub use cornell_smoke::cornell_smoke;
pub use easy_cornell_box::easy_cornell_box;
pub use easy_scene::easy_scene;
pub use one_sphere::one_sphere;
pub use random_scene::random_scene;
use std::collections::HashMap;
use std::rc::Rc;
pub use two_spheres::two_spheres;

pub struct World {
    pub spheres: Vec<Rc<dyn Hittable>>,
    pub lights: Vec<Rc<dyn Light>>,
    pub background: Box<dyn Background>,
}
impl World {
    pub fn nearest_light_hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(Rc<dyn Light>, HitRecord)> {
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
    pub fn into_bvh(self, time_0: f32, time_1: f32) -> Self {
        let sphere_len = self.spheres.len();
        Self {
            spheres: vec![Rc::new(BvhNode::new(
                self.spheres,
                0,
                sphere_len,
                time_0,
                time_1,
            ))],
            lights: self.lights.clone(),
            background: self.background,
        }
    }
}
#[derive(Clone)]
pub struct Scenario {
    pub name: String,
    pub ctor: fn() -> (World, Camera),
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
    ]
    .iter()
    .cloned()
    .map(|scenario| (scenario.name.clone(), scenario))
    .collect()
}
