use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, Object, OctTree, RayScalar, RgbColor,
    SolidColor, SolidVoxel, Sphere, Transform, Voxel, WorldInfo,
};
use cgmath::{prelude::*, Point3, Vector3};
pub fn volcano() -> WorldInfo {
    fn cone(position: Point3<RayScalar>, center: Point3<RayScalar>, slope: RayScalar) -> bool {
        let desired_radius = (position.y - center.y) / slope;
        let desired_radius_squared = if desired_radius > 0. {
            desired_radius * desired_radius
        } else {
            0.
        };
        let distance_squared = (position.x - center.x).powi(2) + (position.z - center.z).powi(2);
        distance_squared < desired_radius_squared
    }
    let origin = Point3::<RayScalar>::new(1000., 500.0, -300.0);
    let look_at = Point3::<RayScalar>::new(500., 200.0, 500.0);
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

    let mut c = OctTree::empty();
    for x in 0..1000 {
        for y in 0..1000 {
            for z in 0..1000 {
                let mountain_cone = cone(
                    Point3::new(x as RayScalar, y as RayScalar, z as RayScalar),
                    Point3::new(500., 300., 500.),
                    -1.,
                );
                let crater = cone(
                    Point3::new(x as RayScalar, y as RayScalar, z as RayScalar),
                    Point3::new(500., 125., 500.),
                    0.3,
                );
                if mountain_cone && (!crater || y < 150) {
                    let v = if crater {
                        Voxel::Solid(SolidVoxel::Lambertian {
                            albedo: RgbColor::from_color_hex("#ffbb00") * 80.,
                        })
                    } else {
                        Voxel::Solid(SolidVoxel::Lambertian {
                            albedo: RgbColor::from_color_hex("#5f1515"),
                        })
                    };
                    c.set(Point3::new(x, y, z), v);
                }
            }
        }
    }
    WorldInfo {
        objects: vec![
            Object::new(Box::new(c), Transform::identity()),
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
