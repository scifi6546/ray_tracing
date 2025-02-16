use super::{Camera, CameraInfo, ConstantColor, WorldInfo};
use crate::prelude::*;

use cgmath::{prelude::*, Point3, Vector3};

pub fn empty_scene() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(0.0, 0.0, -1.0);
    let origin = Point3::<RayScalar>::new(10.0, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    WorldInfo {
        objects: vec![],
        lights: vec![],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.00, 0.00, 0.00),
        }),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov: 20.,
            origin,
            look_at,
            up_vector: Vector3::unit_y(),
            aperture: 0.00001,
            focus_distance,
            start_time: 0.0,
            end_time: 0.0,
        }),
        sun: None,
    }
}
