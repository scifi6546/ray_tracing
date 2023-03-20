use super::{
    hittable_objects::*, world_prelude::*, Camera, CubeWorld, DiffuseLight, Object, Sky,
    SolidColor, Sphere, Transform, WorldInfo,
};
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};
use rand::prelude::*;

pub fn load_vox() -> WorldInfo {
    const BLOCK_X: i32 = 20;
    const BLOCK_Y: i32 = 50;
    const BLOCK_Z: i32 = 20;

    let look_at = Point3::new(BLOCK_X as f32 / 2.0, 10.0, BLOCK_Z as f32 / 2.0);

    let origin = Point3::new(-60.0f32, 80.0, -60.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let light = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(10.0, 10.0, 0.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: RgbColor::WHITE,
                }),
            }),
        }),
        Transform::identity(),
    );

    let mut world = CubeWorld::new(
        vec![
            CubeMaterial::new(RgbColor::new(0.2, 0.05, 0.05)),
            CubeMaterial::new(RgbColor::new(0.65, 0.8, 0.05)),
            CubeMaterial::new(RgbColor::new(0.0, 0.0, 0.5)),
        ],
        vec![],
        BLOCK_X,
        BLOCK_Y,
        BLOCK_Z,
    );
    let files = dot_vox::load("voxel_assets/building.vox").expect("voxel files");
    for m in files.models.iter() {
        for v in m.voxels.iter() {
            world.update(
                v.x as isize,
                v.z as isize,
                v.y as isize,
                CubeMaterialIndex::new_solid(0),
            )
        }
    }
    let sun_radius = 10.0 * f32::PI() / 180.0;
    let sun = Object::new(Box::new(Sun { radius: 1.0 }), Transform::identity());
    let sun_sky = SunSky {
        intensity: 0.0,
        sun_radius,
        sun_theta: 3.0 * f32::PI() / 4.0,
        sun_phi: f32::PI() / 4.0,
    };
    WorldInfo {
        objects: vec![
            Object::new(Box::new(world), Transform::identity()),
            light.clone(),
        ],
        lights: vec![light],
        background: Box::new(Sky { intensity: 0.2 }),
        camera: Camera::new(
            1.0,
            fov,
            origin,
            look_at,
            Vector3::new(0.0, 1.0, 0.0),
            0.00001,
            focus_distance,
            0.0,
            0.0,
        ),
    }
}
