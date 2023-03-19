use super::{new_demo, Lambertian, Object, RenderBox, SolidColor, Transform, WorldInfo};
use crate::prelude::*;
use cgmath::{Point3, Vector3};

pub fn build_field() -> WorldInfo {
    struct CubeInfo {
        x: f32,
        z: f32,
        rotate_y: f32,
    }
    let cube_arr = [
        CubeInfo {
            x: -0.5,
            z: -0.6,
            rotate_y: 20.0,
        },
        CubeInfo {
            x: 0.6,
            z: 0.6,
            rotate_y: -2.0,
        },
    ];
    let objects = cube_arr
        .iter()
        .map(|c| {
            Object::new(
                Box::new(RenderBox::new(
                    Point3::new(-0.5, -0.5, -0.5),
                    Point3::new(0.5, 0.5, 0.5),
                    Box::new(Lambertian {
                        albedo: Box::new(SolidColor {
                            color: RgbColor::new(0.5, 0.1, 0.0),
                        }),
                    }),
                )),
                Transform::identity()
                    .translate(Vector3::new(c.x, 0.5, c.z))
                    .rotate_y(c.rotate_y),
            )
        })
        .collect();
    new_demo(objects)
}
