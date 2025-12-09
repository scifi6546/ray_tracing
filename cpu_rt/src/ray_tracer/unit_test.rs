#[cfg(test)]
mod world_test {
    use super::super::{
        camera::CameraInfo,
        hittable::{Object, OctTree, Sphere, Transform, VoxelMaterial},
        Camera, ConstantColor, DiffuseLight, MaterialEffect, Ray, RayScalar, RgbColor, SolidColor,
        WorldInfo,
    };
    use cgmath::{prelude::*, Point3, Vector3};

    #[test]
    fn oct_tree() {
        let look_at = Point3::<RayScalar>::new(5.0, 5.0, 5.0);
        let origin = Point3::<RayScalar>::new(-20., 5., -20.);
        let fov = 40.0;
        let focus_distance = {
            let t = look_at - origin;
            (t.dot(t)).sqrt()
        };
        let light = Box::new(DiffuseLight {
            emit: Box::new(SolidColor {
                color: 500.0 * RgbColor::WHITE,
            }),
        });
        let light = Object::new(
            Box::new(Sphere {
                radius: 1.0,
                origin: Point3::new(200.0, 5.0, 200.0),
                material: light,
            }),
            Transform::identity(),
        );
        let mut tree = OctTree::<VoxelMaterial>::empty();
        for x in 0..10 {
            for y in 1..9 {
                for z in 0..10 {
                    tree.set(
                        Point3 { x, y, z },
                        VoxelMaterial::Solid {
                            //  density: 0.3,
                            color: RgbColor::new(0.5, 0.05, 0.5),
                        },
                    );
                }
            }
        }
        let world = WorldInfo {
            objects: vec![
                Object::new(Box::new(tree), Transform::identity()),
                light.clone(),
            ],
            // lights: vec![light],
            lights: Vec::new(),
            background: Box::new(ConstantColor {
                color: 0.1 * RgbColor::new(1.0, 1.0, 1.0),
            }),
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
            sun: None,
        }
        .build_world();
        let hit = world
            .nearest_hit(
                &Ray {
                    origin,
                    direction: look_at - origin,
                    time: 0.,
                },
                0.0,
                100.,
            )
            .unwrap();
        let is_scatter = match hit.material_effect {
            MaterialEffect::Scatter(_) => true,
            _ => false,
        };
        if !is_scatter {
            panic!(
                "invalid material effect type, expecting to hit oct tree first, \n{:#?}",
                hit
            )
        }
    }
}
