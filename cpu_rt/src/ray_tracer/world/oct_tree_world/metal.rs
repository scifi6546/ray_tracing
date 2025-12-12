use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, Object, OctTree, RayScalar, RgbColor,
    SolidColor, SolidVoxel, Sphere, Transform, Voxel, WorldInfo,
};
use cgmath::{prelude::*, Point3, Vector3};
pub fn gold_cube() -> WorldInfo {
    let origin = Point3::<RayScalar>::new(64., 100.0, -300.0);
    let look_at = Point3::<RayScalar>::new(64., 0.0, 64.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let top_light = Object::new(
        Box::new(Sphere {
            radius: 10.0,
            origin: Point3::new(-100., 50., -100.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: RgbColor::WHITE * 100.,
                }),
            }),
        }),
        Transform::identity(),
    );
    let g1 = OctTree::cube_pow(
        4,
        Voxel::Solid(SolidVoxel::Reflect {
            albedo: RgbColor::new(255. / 255., 254. / 255., 0.),
            fuzz: 0.8,
        }),
    );
    let g2 = OctTree::cube_pow(
        4,
        Voxel::Solid(SolidVoxel::Reflect {
            albedo: RgbColor::new(255. / 255., 254. / 255., 0.),
            fuzz: 0.4,
        }),
    );
    let g3 = OctTree::cube_pow(
        4,
        Voxel::Solid(SolidVoxel::Reflect {
            albedo: RgbColor::new(255. / 255., 254. / 255., 0.),
            fuzz: 0.1,
        }),
    );
    let mut floor = OctTree::<Voxel>::empty();
    for x in 0..128 {
        for z in 0..128 {
            floor.set(
                Point3::new(x, 0, z),
                Voxel::Solid(SolidVoxel::Lambertian {
                    albedo: RgbColor {
                        red: 0.8,
                        green: 0.8,
                        blue: 0.8,
                    },
                }),
            )
        }
    }
    let world = floor
        .combine(&g1, Point3::new(30, 1, 64))
        .combine(&g2, Point3::new(60, 1, 64))
        .combine(&g3, Point3::new(90, 1, 64));
    WorldInfo {
        objects: vec![
            Object::new(Box::new(world), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.1, 0.1, 0.1),
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
