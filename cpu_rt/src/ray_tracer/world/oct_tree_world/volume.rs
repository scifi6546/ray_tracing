use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, Object, OctTree, RayScalar, RgbColor,
    SolidColor, Sphere, Transform, VoxelMaterial, WorldInfo,
};
use cgmath::{prelude::*, Point3, Vector3};

pub fn oct_tree_volume() -> WorldInfo {
    let origin = Point3::<RayScalar>::new(1.0, 1.0, 1.0);
    let look_at = Point3::<RayScalar>::new(10., 10., 10.);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let top_light = Object::new(
        Box::new(Sphere {
            radius: 10.0,
            origin: Point3::new(-320.0, 100.0, -100.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 400.0 * RgbColor::WHITE,
                }),
            }),
        }),
        Transform::identity(),
    );
    WorldInfo {
        objects: vec![
            Object::new(
                Box::new(OctTree::sphere(10, VoxelMaterial::Volume { density: 0.5 })),
                Transform::identity(),
            ),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: 0.1 * RgbColor::WHITE,
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
