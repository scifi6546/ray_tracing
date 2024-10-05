use super::{
    hittable::{Hittable, Object},
    ray_tracer_info::EntityInfo,
    HitRecord, Ray,
};
use crate::prelude::*;

use cgmath::Point3;

use crate::ray_tracer::ray_tracer_info::Entity;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub minimum: Point3<RayScalar>,
    pub maximum: Point3<RayScalar>,
}
impl Aabb {
    pub fn hit(&self, ray: Ray, t_min: RayScalar, t_max: RayScalar) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / ray.direction[a];
            let mut t0 = (self.minimum[a] - ray.origin[a]) * inv_d;
            let mut t1 = (self.maximum[a] - ray.origin[a]) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            let t_min = if t0 > t_min { t0 } else { t_min };
            let t_max = if t1 < t_max { t1 } else { t_max };
            if t_max < t_min {
                return false;
            }
        }
        true
    }
    pub fn surrounding_box(self, box1: Aabb) -> Self {
        Self {
            minimum: Point3 {
                x: p_min(self.minimum.x, box1.minimum.x),
                y: p_min(self.minimum.y, box1.minimum.y),
                z: p_min(self.minimum.z, box1.minimum.z),
            },
            maximum: Point3 {
                x: p_max(self.maximum.x, box1.maximum.x),
                y: p_max(self.maximum.y, box1.maximum.y),
                z: p_max(self.maximum.z, box1.maximum.z),
            },
        }
    }
    pub fn contains_point(&self, point: Point3<RayScalar>) -> bool {
        self.minimum.x <= point.x
            && self.minimum.y <= point.y
            && self.minimum.z <= point.z
            && self.maximum.x >= point.x
            && self.maximum.y >= point.y
            && self.maximum.z >= point.z
    }
}
#[derive(Clone)]
pub struct BvhTree {
    objects: Vec<Object>,
    root_node: BvhTreeNode,
}
impl BvhTree {
    pub fn new(objects: Vec<Object>, time_0: RayScalar, time_1: RayScalar) -> Self {
        if objects.is_empty() {
            Self {
                objects: Vec::new(),
                root_node: BvhTreeNode::None,
            }
        } else {
            let root_node =
                BvhTreeNode::new(&objects, &objects, 0, objects.len(), 0, time_0, time_1);
            Self { objects, root_node }
        }
    }
    pub fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
        self.root_node.hit(&self.objects, ray, t_min, t_max)
    }
    pub fn bounding_box(&self, time_0: RayScalar, time_1: RayScalar) -> Option<Aabb> {
        self.root_node.bounding_box(&self.objects, time_0, time_1)
    }
    pub fn get_info(&self) -> Vec<EntityInfo> {
        self.objects
            .iter()
            .map(|obj| EntityInfo {
                name: Entity::name(obj),
                fields: Entity::fields(obj),
            })
            .collect()
    }
}
#[derive(Clone)]
enum BvhTreeNode {
    None,
    Child {
        bounding_box: Aabb,
        left: Box<BvhTreeNode>,
        right: Box<BvhTreeNode>,
    },
    Leaf {
        idx: usize,
    },
}
impl BvhTreeNode {
    fn hit(
        &self,
        objects: &[Object],
        ray: &Ray,
        t_min: RayScalar,
        t_max: RayScalar,
    ) -> Option<HitRecord> {
        if !self
            .bounding_box(objects, t_min, t_max)
            .expect("object does not have bounding box")
            .hit(*ray, t_min, t_max)
        {
            None
        } else {
            match self {
                Self::None => None,
                Self::Child { left, right, .. } => {
                    let left_hit = left.hit(objects, ray, t_min, t_max);
                    if left_hit.is_none() {
                        right.hit(objects, ray, t_min, t_max)
                    } else {
                        let right_hit =
                            right.hit(objects, ray, t_min, left_hit.as_ref().unwrap().t);
                        if right_hit.is_some() {
                            right_hit
                        } else {
                            left_hit
                        }
                    }
                }
                Self::Leaf { idx } => objects[*idx].hit(ray, t_min, t_max),
            }
        }
    }
    fn bounding_box(
        &self,
        objects: &[Object],
        time_0: RayScalar,
        time_1: RayScalar,
    ) -> Option<Aabb> {
        match self {
            Self::None => Some(Aabb {
                minimum: Point3::new(0.0, 0.0, 0.0),
                maximum: Point3::new(0.0, 0.0, 0.0),
            }),
            Self::Child { bounding_box, .. } => Some(bounding_box.clone()),
            Self::Leaf { idx } => Some(objects[*idx].bounding_box(time_0, time_1).unwrap()),
        }
    }
    fn box_compare(a: Object, b: Object, axis: usize) -> Ordering {
        let a_box = a
            .bounding_box(0.0, 0.0)
            .expect("bvh node does not have bounding box");
        let b_box = b
            .bounding_box(0.0, 0.0)
            .expect("bvh node does not have bounding box");

        if a_box.minimum[axis] < b_box.minimum[axis] {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
    fn box_x_compare(a: Object, b: Object) -> Ordering {
        Self::box_compare(a, b, 0)
    }
    fn box_y_compare(a: Object, b: Object) -> Ordering {
        Self::box_compare(a, b, 1)
    }
    fn box_z_compare(a: Object, b: Object) -> Ordering {
        Self::box_compare(a, b, 2)
    }
    pub fn new(
        objects: &[Object],
        objects_full: &[Object],
        start: usize,
        end: usize,
        offset: usize,
        time_0: RayScalar,
        time_1: RayScalar,
    ) -> Self {
        let axis = rand_u32(0, 2);
        let span = end - start;
        let comparator = if axis == 0 {
            |a: &Object, b: &Object| Self::box_x_compare((*a).clone(), (*b).clone())
        } else if axis == 1 {
            |a: &Object, b: &Object| Self::box_y_compare((*a).clone(), (*b).clone())
        } else {
            |a: &Object, b: &Object| Self::box_z_compare((*a).clone(), (*b).clone())
        };
        let (left, right) = if span == 1 {
            (
                Self::Leaf {
                    idx: start + offset,
                },
                Self::Leaf {
                    idx: start + offset,
                },
            )
        } else if span == 2 {
            if comparator(&objects[start].clone(), &objects[start + 1].clone()) == Ordering::Less {
                (
                    Self::Leaf {
                        idx: start + offset,
                    },
                    Self::Leaf {
                        idx: start + 1 + offset,
                    },
                )
            } else {
                (
                    Self::Leaf {
                        idx: start + 1 + offset,
                    },
                    Self::Leaf {
                        idx: start + offset,
                    },
                )
            }
        } else {
            let mut s_vec = (start..end).map(|i| objects[i].clone()).collect::<Vec<_>>();
            s_vec.sort_by(comparator);
            let middle = s_vec.len() / 2;

            let left = Self::new(
                &s_vec[0..middle],
                objects_full,
                0,
                middle,
                offset,
                time_0,
                time_1,
            );
            let s = &s_vec[middle..s_vec.len()];
            let right = Self::new(
                &s_vec[middle..s_vec.len()],
                objects_full,
                0,
                s.len(),
                offset + middle,
                time_0,
                time_1,
            );

            (left, right)
        };
        let left_box = left
            .bounding_box(objects_full, time_0, time_1)
            .expect("no bounding box for object");
        let right_box = right
            .bounding_box(objects_full, time_0, time_1)
            .expect("no bounding box for object");
        let bounding_box = left_box.surrounding_box(right_box);

        Self::Child {
            left: Box::new(left),
            right: Box::new(right),
            bounding_box,
        }
    }
}
