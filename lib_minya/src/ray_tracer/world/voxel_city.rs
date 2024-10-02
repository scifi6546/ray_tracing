use super::{
    hittable_objects::*, world_prelude::*, Camera, CameraInfo, Object, Transform, VoxelWorld,
    WorldInfo,
};
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};

pub fn voxel_city() -> WorldInfo {
    const BLOCK_X: i32 = 200;
    const BLOCK_Y: i32 = 500;
    const BLOCK_Z: i32 = 200;

    let look_at = Point3::new(BLOCK_X as RayScalar / 2.0, 10.0, BLOCK_Z as RayScalar / 2.0);

    let origin = Point3::<RayScalar>::new(-150.0, 200.0, -150.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    fn height(x: isize, z: isize) -> isize {
        let center = Point2::new(BLOCK_X as f32 / 2.0, BLOCK_Z as f32 / 2.0);
        let radius = center.distance(Point2::new(x as f32, z as f32));
        let h = (-radius / 10.0).exp() * 30.0;
        h.max(20.0).min(BLOCK_Y as f32) as isize
    }
    let mut world = VoxelWorld::new(
        vec![
            CubeMaterial::new(RgbColor::new(0.2, 0.05, 0.05)),
            CubeMaterial::new(0.1 * RgbColor::new(0.65, 0.8, 0.05)),
            CubeMaterial::new(0.1 * RgbColor::new(0.0, 0.0, 0.5)),
        ],
        vec![CubeMaterial::new(0.1 * RgbColor::new(0.0, 0.0, 0.5))],
        BLOCK_X,
        BLOCK_Y,
        BLOCK_Z,
    );
    let noise = PerlinBuilder::new(BLOCK_X as usize).num_layers(2).build();

    for x in 0..BLOCK_X as isize {
        for z in 0..BLOCK_Z as isize {
            let h = height(x, z);

            let rand_sample = 0.3
                * noise
                    .get(
                        x as f32 / (BLOCK_X as f32 - 1.0),
                        z as f32 / (BLOCK_Z as f32 - 1.0),
                    )
                    .min(200.0);
            let terrain_height = h + rand_sample as isize;
            for y2 in terrain_height + 1..30 {
                // let material = CubeMaterialIndex::new_translucent(0, 0.1);

                world.update(x, y2, z, CubeMaterialIndex::new_solid(2));
            }
            for y in 0..=terrain_height {
                world.update(x, y, z, CubeMaterialIndex::new_solid(1));
            }
        }
    }

    let num_roads = 6;
    let road_width = 4;
    let block_size = 32;

    for road in 0..num_roads {
        for z in 0..100 {
            let width = road_width + block_size;
            let offset = road * width;
            for x in 0..road_width {
                let x_put = x + offset;
                if x_put < 100 {
                    world.update(x_put, 10, z, CubeMaterialIndex::new_solid(0));
                }
            }
        }
    }

    let sun = Sun {
        phi: 1.0 * RayScalar::PI() / 6.0,
        theta: 1.231,
        radius: 5.0 * RayScalar::PI() / 180.0,
    };
    let sun_sky = SunSky::new(sun, 0.05, 12.0);
    WorldInfo {
        objects: vec![
            Object::new(Box::new(world), Transform::identity()),
            //   lava_light.clone(),
        ],
        //lights: vec![lava_light],
        lights: vec![],
        background: Box::new(sun_sky),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov,
            origin,
            look_at,
            up_vector: Vector3::unit_y(),
            aperture: 0.00001,
            focus_distance,
            start_time: 0.0,
            end_time: 0.0,
        }),
        sun: Some(sun),
    }
}
