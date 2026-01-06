use bevy::{
    asset::RenderAssetUsages,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use cgmath::{InnerSpace, Zero};
use core::f32;
use rand::{Rng, rngs::ThreadRng};
use std::{
    f32::consts::{FRAC_PI_2, PI},
    ops::Range,
};
#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    // Clamp pitch to this range
    pub pitch_range: Range<f32>,
    pub yaw_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            // These values are completely arbitrary, chosen because they seem to produce
            // "sensible" results for this example. Adjust as required.
            orbit_distance: 20.0,
            pitch_speed: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
            yaw_speed: 0.004,
        }
    }
}
#[derive(Debug, Resource, Component)]
struct DirectionVector {
    direction: cgmath::Vector3<f32>,
}
impl std::default::Default for DirectionVector {
    fn default() -> Self {
        Self {
            direction: cgmath::Vector3::unit_x(),
        }
    }
}
#[derive(Component, Debug)]
struct DrawPoint {
    theta: f32,
    phi: f32,
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_linear()))
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, (setup, axis_cube, ui_startup))
        .init_resource::<CameraSettings>()
        .init_resource::<DirectionVector>()
        .add_systems(Update, orbit)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}
fn ui_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let theta_num = 50;
    let phi_num = 50;
    let sphere = meshes.add(Sphere::default().mesh().uv(32, 18));
    let sphere_color = materials.add(StandardMaterial {
        base_color: Color::LinearRgba(LinearRgba {
            red: 0.5,
            green: 0.0,
            blue: 1.0,
            alpha: 1.0,
        }),
        metallic: 1.0,
        perceptual_roughness: 0.0,
        ..default()
    });
    for i in 0..theta_num {
        for j in 0..phi_num {
            let theta = (i as f32 * f32::consts::PI * 2.) / theta_num as f32;
            let phi = (j as f32 * f32::consts::PI * 2.) / phi_num as f32;
            commands.spawn((
                Mesh3d(sphere.clone()),
                MeshMaterial3d(sphere_color.clone()),
                Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(0.1, 0.1, 0.1)),
                DrawPoint { theta, phi },
            ));
        }
    }
}

fn ui_system(
    mut query: Query<(&DrawPoint, &mut Transform)>,
    mut direction: ResMut<DirectionVector>,
    mut contexts: EguiContexts,
) -> Result {
    egui::Window::new("Hello").show(contexts.ctx_mut()?, |ui| {
        ui.label("world");
        let mut x = direction.direction.x;
        let range = (0.)..=(1.);
        ui.add(egui::Slider::new(&mut x, range.clone()).text("x"));
        direction.direction.x = x;

        let mut y = direction.direction.y;

        ui.add(egui::Slider::new(&mut y, range.clone()).text("y"));
        direction.direction.y = y;

        let mut z = direction.direction.z;

        ui.add(egui::Slider::new(&mut z, range).text("z"));
        direction.direction.z = z;

        direction.direction = direction.direction.normalize();
        let p = if (direction.direction.x * direction.direction.x
            + direction.direction.y * direction.direction.y)
            .abs()
            >= 0.1
        {
            cgmath::vec3(-direction.direction.y, direction.direction.x, 0.).normalize()
        } else {
            cgmath::vec3(0., -direction.direction.z, direction.direction.y)
        };

        ui.label(format!(
            "perpendicular: <{:.2}, {:.2}, {:.2}>",
            p.x, p.y, p.z
        ));
        ui.label(format!("dot product: {:.2}", p.dot(direction.direction)));
        let cross = direction.direction.cross(p);
        ui.label(format!(
            "perpendicular 2: <{:.2}, {:.2}, {:.2}>",
            cross.x, cross.y, cross.z
        ));
        ui.label(format!(
            "dot product: {:.2}",
            cross.dot(direction.direction)
        ));
        ui.label(format!("dot product: {:.2}", cross.dot(p)));
        let matrix = cgmath::Matrix3::from_cols(p, cross, cgmath::Vector3::zero());
        for (draw_point, mut transform) in query.iter_mut() {
            let x = draw_point.theta.cos();
            let y = draw_point.theta.sin();
            let world_point = matrix * cgmath::Vector3::new(x, y, 0.);

            let final_matrix = cgmath::Matrix3::from_cols(
                direction.direction,
                world_point,
                cgmath::Vector3::zero(),
            );
            let phi_x = draw_point.phi.cos();
            let phi_y = draw_point.phi.sin();
            let final_point = final_matrix * cgmath::Vector3::new(phi_x, phi_y, 0.);
            transform.translation = vec3(final_point.x, final_point.y, final_point.z);
        }
    });

    Ok(())
}
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
fn orbit(
    mut camera: Single<&mut Transform, With<Camera>>,
    mut camera_settings: ResMut<CameraSettings>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
) {
    if accumulated_mouse_scroll.delta.y > 0.1 {
        camera_settings.orbit_distance *= 1.2;
        if camera_settings.orbit_distance > 20. {
            camera_settings.orbit_distance = 20.
        }
    } else if accumulated_mouse_scroll.delta.y < -0.1 {
        camera_settings.orbit_distance *= 0.8;
        if camera_settings.orbit_distance < 1. {
            camera_settings.orbit_distance = 1.
        }
    }
    let delta = mouse_motion.delta;

    // Mouse motion is one of the few inputs that should not be multiplied by delta time,
    // as we are already receiving the full movement since the last frame was rendered. Multiplying
    // by delta time here would make the movement slower that it should be.
    let delta_pitch = delta.y * camera_settings.pitch_speed;
    let delta_yaw = delta.x * camera_settings.yaw_speed;

    // Obtain the existing pitch, yaw, and roll values from the transform.
    let (yaw, pitch, _roll) = camera.rotation.to_euler(EulerRot::YXZ);

    // Establish the new yaw and pitch, preventing the pitch value from exceeding our limits.
    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );

    let yaw = yaw + delta_yaw;
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.);

    // Adjust the translation to maintain the correct orientation toward the orbit target.
    // In our example it's a static target, but this could easily be customized.
    let target = Vec3::ZERO;
    camera.translation = target - camera.forward() * camera_settings.orbit_distance;
}
fn axis_cube(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let cube = meshes.add(Cuboid::default());
    let x_material = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba {
            red: 1.,
            green: 0.,
            blue: 0.,
            alpha: 1.,
        }),
        metallic: 1.0,
        perceptual_roughness: 0.0,
        ..default()
    });
    let y_material = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba {
            red: 0.,
            green: 1.,
            blue: 0.,
            alpha: 1.,
        }),
        metallic: 1.0,
        perceptual_roughness: 0.0,
        ..default()
    });
    let z_material = materials.add(StandardMaterial {
        base_color: Color::Srgba(Srgba {
            red: 0.,
            green: 0.,
            blue: 1.,
            alpha: 1.,
        }),
        metallic: 1.0,
        perceptual_roughness: 0.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(cube.clone()),
        Transform::from_xyz(-10., -10., -10.).with_scale(Vec3::new(5., 1., 1.)),
        MeshMaterial3d(x_material),
    ));
    commands.spawn((
        Mesh3d(cube.clone()),
        Transform::from_xyz(-10., -10., -10.).with_scale(Vec3::new(1., 5., 1.)),
        MeshMaterial3d(y_material),
    ));
    commands.spawn((
        Mesh3d(cube),
        Transform::from_xyz(-10., -10., -10.).with_scale(Vec3::new(1., 1., 5.)),
        MeshMaterial3d(z_material),
    ));
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 3., 5.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));
    let sphere = meshes.add(Sphere::default().mesh().uv(32, 18));
    commands.spawn((
        Mesh3d(sphere.clone()),
        MeshMaterial3d(debug_material.clone()),
        Transform::from_xyz(0., 0., 0.),
    ));
    let mut rng = rand::rng();
    for _ in 0..1000 {
        let _v = rand_unit_vec(&mut rng);
        let v = hg_distribution_3d(&mut rng, Vec3::new(1., 1., 0.));
        commands.spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                metallic: 1.0,
                perceptual_roughness: 0.0,
                ..default()
            })),
            Transform::from_xyz(v.x, v.y, v.z).with_scale(Vec3::new(0.1, 0.1, 0.1)),
        ));
    }
}
fn rand_unit_vec(rng: &mut ThreadRng) -> Vec3 {
    let x = 2.0 * rng.random::<f32>() - 1.0;
    let y = 2.0 * rng.random::<f32>() - 1.0;
    let z = 2.0 * rng.random::<f32>() - 1.0;
    Vec3::new(x, y, z).normalize()
}
fn hg_distribution(r: f32, g: f32) -> f32 {
    //lim(hg) g->0 = s therefore solving to avoid div/0 errors
    let s = 2. * r - 1.;
    if g == 0. || g == -0. {
        s
    } else {
        1. / (2.0 * g)
            * (1. + g * g - ((1. - g * g) / (1. + g * s)) * ((1. - g * g) / (1. + g * s)))
    }
}

fn hg_distribution_3d(rng: &mut ThreadRng, look_at: Vec3) -> Vec3 {
    use cgmath::{Matrix3, Matrix4, Point3, Vector3, Vector4, prelude::*};

    let orthogonal = Vector3::unit_y();

    let m = Matrix4::look_at_rh(
        Point3::origin(),
        Point3::new(look_at.x, look_at.y, look_at.z),
        orthogonal,
    );
    let r = rng.random();
    let u = hg_distribution(r, 0.9);

    let theta = u.acos();
    let phi = rng.random::<f32>() * 2. * PI;

    let distribution_vector = Vector4::new(
        theta.sin() * phi.cos(),
        theta.sin() * phi.sin(),
        theta.cos(),
        1.,
    );
    let o = m * distribution_vector;
    Vec3::new(o.x, o.y, o.z)
}
