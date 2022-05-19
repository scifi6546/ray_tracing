mod random_scene;

use super::{
    bvh::BvhNode, hittable::*, material::*, texture::*, Background, Camera, HitRecord, Hittable,
    Light, Sky,
};
use crate::prelude::*;
use std::rc::Rc;
pub struct World {
    pub spheres: Vec<Rc<dyn Hittable>>,
    pub lights: Vec<Rc<dyn Light>>,
    pub background: Box<dyn Background>,
}
impl World {
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
