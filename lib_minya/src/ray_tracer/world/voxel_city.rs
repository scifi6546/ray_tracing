use super::{
    hittable_objects::*, world_prelude::*, Camera, CubeWorld, DiffuseLight, Object, Sky,
    SolidColor, Sphere, Transform, WorldInfo,
};
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};
use rand::prelude::*;
struct PerlinGrid {
    data: Vec<f32>,
    width: usize,
    height: usize,
}
impl PerlinGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1233897876);
        let mut data = vec![];
        data.reserve(width * height);
        for _ in 0..width {
            for _ in 0..height {
                data.push(rng.gen_range(0.0..40.0))
            }
        }
        Self {
            data,
            width,
            height,
        }
    }
    fn get_idx(&self, x: usize, y: usize) -> usize {
        x + self.width * y
    }
    //get in range
    pub fn get(&self, x: f32, y: f32) -> f32 {
        if x >= 0.0 && x <= 1.0 && y >= 0.0 && y <= 1.0 {
            let x_get = x * self.width as f32;
            let x0 = (x_get.floor() as usize).min(self.width - 1);
            let x1 = (x0 + 1).min(self.width - 1);

            let y_get = y * self.height as f32;
            let y0 = (y_get.floor() as usize).min(self.height - 1);
            let y1 = (y0 + 1).min(self.height - 1);

            let rx0_y0 = self.data[self.get_idx(x0, y0)];
            let rx1_y0 = self.data[self.get_idx(x1, y0)];
            let ry0 = (1.0 - x_get.fract()) * rx0_y0 + x_get.fract() * rx1_y0;
            let rx0_y1 = self.data[self.get_idx(x0, y1)];
            let rx1_y1 = self.data[self.get_idx(x1, y1)];
            let ry1 = (1.0 - x_get.fract()) * rx0_y1 + x_get.fract() * rx1_y1;
            (1.0 - y_get.fract()) * ry0 + y_get.fract() * ry1
        } else {
            panic!()
        }
    }
}
pub fn voxel_city() -> WorldInfo {
    const BLOCK_X: i32 = 200;
    const BLOCK_Y: i32 = 500;
    const BLOCK_Z: i32 = 200;

    let look_at = Point3::new(BLOCK_X as f32 / 2.0, 10.0, BLOCK_Z as f32 / 2.0);

    let origin = Point3::new(-600.0f32, 800.0, -600.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: 200.0 * RgbColor::new(252.0 / 255.0, 79.0 / 255.0, 5.0 / 255.0),
        }),
    });
    let lava_light = Object::new(
        Box::new(Sphere {
            radius: 3.0,
            origin: Point3::new(50.0, 100.0, 50.0),
            material: light.clone(),
        }),
        Transform::identity(),
    );
    let lava_light_far = Object::new(
        Box::new(Sphere {
            radius: 3.0,
            origin: Point3::new(500000.0, 280000.0, 500000.0),
            material: light.clone(),
        }),
        Transform::identity(),
    );

    fn height(x: isize, z: isize) -> isize {
        let center = Point2::new(BLOCK_X as f32 / 2.0, BLOCK_Z as f32 / 2.0);
        let radius = center.distance(Point2::new(x as f32, z as f32));
        let h = (-radius / 10.0).exp() * 300.0;
        h.max(20.0).min(BLOCK_Y as f32) as isize
    }
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
    const X_GRID: i32 = BLOCK_X / 10;
    const Z_GRID: i32 = BLOCK_Z / 10;
    let mut rng = rand::rngs::StdRng::seed_from_u64(1233897876);
    let mut grid = vec![];
    for x in 0..X_GRID {
        let mut z_row = [0.0f32; Z_GRID as usize];
        for z in 0..Z_GRID {
            z_row[z as usize] = rng.gen_range(0.0..20.0);
        }
        grid.push(z_row);
    }
    let noise = PerlinGrid::new(X_GRID as usize, Z_GRID as usize);
    for x in 0..BLOCK_X as isize {
        for z in 0..BLOCK_Z as isize {
            let h = height(x, z);
            let x_get = (x as f32 / BLOCK_X as f32) * X_GRID as f32;
            let x0_get = x_get.floor() as usize;
            let x1_get = (x0_get + 1).min(X_GRID as usize - 1);

            let z_get = ((z as f32 / BLOCK_Z as f32) * Z_GRID as f32);
            let z0_get = z_get.floor() as usize;
            let z1_get = (z0_get + 1).min(Z_GRID as usize - 1);
            let rx0_z0 = grid[x0_get][z0_get];
            let rx1_z0 = grid[x1_get][z0_get];
            let rz0 = (1.0 - x_get.fract()) * rx0_z0 + x_get.fract() * rx1_z0;
            let rx0_z1 = grid[x0_get][z1_get];
            let rx1_z1 = grid[x1_get][z1_get];
            let rz1 = (1.0 - x_get.fract()) * rx0_z1 + x_get.fract() * rx1_z1;
            let rand_sample = (1.0 - z_get.fract()) * rz0 + z_get.fract() * rz1;
            let rand_sample = noise.get(
                x as f32 / (BLOCK_X as f32 - 1.0),
                z as f32 / (BLOCK_Z as f32 - 1.0),
            );
            let terrain_height = h + rand_sample as isize;
            for y in 0..=terrain_height {
                let index = if terrain_height <= 50 { 2 } else { 1 };

                world.update(x, y, z, CubeMaterialIndex::new_solid(index));
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
            lava_light.clone(),
        ],
        lights: vec![lava_light],
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
