use super::{
    world_prelude::{
        Camera, CameraInfo, ConstantColor, DiffuseLight, FastOctTree, Object, RayScalar, RgbColor,
        SolidColor, SolidVoxel, Sphere, Transform, Voxel,
    },
    WorldInfo,
};
use cgmath::{prelude::*, Point3, Vector3};
pub fn fast_oct_tree() -> WorldInfo {
    let origin = Point3::<RayScalar>::new(10.0, 10.0, 10.0);
    let look_at = Point3::<RayScalar>::new(0., 0., 0.);
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
    let mut tree = FastOctTree::<Voxel>::new();
    tree.set(
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::RED * 0.5,
        }),
        Point3::new(0, 0, 0),
    );

    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
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
