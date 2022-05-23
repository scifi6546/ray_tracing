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
        let t = self
            .lights
            .iter()
            .map(|s| (s.clone(), s.hit(ray, t_min, t_max)))
            .filter(|(light, hit)| hit.is_some())
            .map(|(light, hit)| (light, hit.unwrap()))
            .reduce(|acc, x| if acc.1.t < x.1.t { acc } else { x });
        todo!()
    }

    pub fn nearest_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.spheres
            .iter()
            .filter_map(|s| s.hit(ray, t_min, t_max))
            .reduce(|acc, x| if acc.t < x.t { acc } else { x })
    }
    pub fn to_bvh(self, time_0: f32, time_1: f32) -> Self {
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
