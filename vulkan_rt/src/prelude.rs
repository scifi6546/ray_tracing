mod mesh;
pub mod voxel;

use crate::Base;
use ash::vk;
use cgmath::{
    num_traits::FloatConst, Deg, Euler, Matrix, Matrix4, Point3, Quaternion, Rad, SquareMatrix,
    Vector3,
};
use generational_arena::{Arena, Index as ArenaIndex};
use gpu_allocator::vulkan::*;
pub use mesh::*;

use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct Mat4 {
    pub data: [f32; 4 * 4],
}
impl Mat4 {
    pub fn identity() -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}
pub fn mat4_to_bytes<'a>(mat: &'a cgmath::Matrix4<f32>) -> &'a [u8] {
    let ptr = mat.as_ptr() as *const u8;
    let arr = std::ptr::slice_from_raw_parts(ptr, std::mem::size_of::<f32>() * 4 * 4);
    unsafe { &*arr }
}
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}
impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl Vector4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: Vector4,
    pub uv: Vector2,
}
impl Vertex {
    pub const fn position_format() -> vk::Format {
        vk::Format::R32G32B32A32_SFLOAT
    }
    pub const fn format() -> VertexFormat {
        VertexFormat {
            position: Self::position_format(),
            uv: vk::Format::R32G32_SFLOAT,
        }
    }
    pub const fn stride() -> usize {
        std::mem::size_of::<Self>()
    }
}
#[derive(Clone, Debug, Copy)]
pub struct VertexFormat {
    pub position: vk::Format,
    pub uv: vk::Format,
}
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
impl Mesh {
    pub fn plane() -> Self {
        Self {
            vertices: vec![
                Vertex {
                    pos: Vector4::new(-1.0, -1.0, 0.0, 1.0),
                    uv: Vector2::new(0.0, 0.0),
                },
                Vertex {
                    pos: Vector4::new(-1.0, 1.0, 0.0, 1.0),
                    uv: Vector2::new(0.0, 1.0),
                },
                Vertex {
                    pos: Vector4::new(1.0, 1.0, 0.0, 1.0),
                    uv: Vector2::new(1.0, 1.0),
                },
                Vertex {
                    pos: Vector4::new(1.0, -1.0, 0.0, 1.0),
                    uv: Vector2::new(1.0, 0.0),
                },
            ],
            indices: vec![0u32, 1, 2, 2, 3, 0],
        }
    }
    pub fn cylinder() -> Self {
        let num_segments = 16;
        let vertices = (0..num_segments)
            .flat_map(|i| {
                let theta = 2.0 * f32::PI() * ((i) as f32 / num_segments as f32);
                let x = theta.sin();
                let z = theta.cos();
                let v = (i) as f32 / num_segments as f32;
                [
                    Vertex {
                        pos: Vector4::new(x, 1.0, z, 1.0),
                        uv: Vector2::new(1.0, v),
                    },
                    Vertex {
                        pos: Vector4::new(x, -1.0, z, 1.0),
                        uv: Vector2::new(0.0, v),
                    },
                ]
            })
            .chain([
                Vertex {
                    pos: Vector4::new(0.0, 1.0, 0.0, 1.0),
                    uv: Vector2::new(0.5, 0.5),
                },
                Vertex {
                    pos: Vector4::new(0.0, -1.0, 0.0, 1.0),
                    uv: Vector2::new(0.5, 0.5),
                },
            ])
            .collect::<Vec<_>>();

        let indices = (0..num_segments)
            .flat_map(|i| [i * 2 + 1, i * 2 + 3, i * 2, i * 2 + 3, i * 2 + 2, i * 2])
            .map(|idx| {
                if idx < num_segments * 2 {
                    idx
                } else {
                    idx - num_segments * 2
                }
            })
            .chain(
                // top of cylinder
                (0..num_segments).flat_map(|i| {
                    [
                        i * 2,
                        if (i + 1) * 2 < num_segments * 2 {
                            (i + 1) * 2
                        } else {
                            (i + 1) * 2 - num_segments * 2
                        },
                        num_segments * 2,
                    ]
                }),
            )
            .chain((0..num_segments).flat_map(|i| {
                [
                    i * 2 + 1,
                    if (i + 1) * 2 + 1 < num_segments * 2 {
                        (i + 1) * 2 + 1
                    } else {
                        (i + 1) * 2 + 1 - num_segments * 2
                    },
                    num_segments * 2 + 1,
                ]
            }))
            .collect::<Vec<_>>();
        Self { vertices, indices }
    }
    pub fn sphere(num_vertical_segments: u32, num_horizontal_segments: u32) -> Self {
        let vertices = (1..num_vertical_segments)
            .flat_map(|i| {
                let phi = (i as f32 / num_vertical_segments as f32 - 0.5) * f32::PI();
                let r = phi.cos();

                (0..num_horizontal_segments + 1).map(move |j| {
                    let theta = 2.0 * f32::PI() * ((j) as f32 / num_horizontal_segments as f32);

                    let x = theta.sin() * r;
                    let y = phi.sin();
                    let z = theta.cos() * r;
                    let uv = Vector2::new(
                        j as f32 / (num_horizontal_segments) as f32,
                        i as f32 / num_vertical_segments as f32,
                    );

                    Vertex {
                        pos: Vector4::new(x, y, z, 1.0),
                        uv,
                    }
                })
            })
            .chain([
                Vertex {
                    pos: Vector4::new(0.0, -1.0, 0.0, 1.0),
                    uv: Vector2::new(0.5, 0.0),
                },
                Vertex {
                    pos: Vector4::new(0.0, 1.0, 0.0, 1.0),
                    uv: Vector2::new(0.5, 1.0),
                },
            ])
            .collect::<Vec<_>>();
        let indices = (0..num_vertical_segments - 2)
            .flat_map(|i| {
                (0..num_horizontal_segments).flat_map(move |j| {
                    let next_j = j + 1;
                    let zero = i * (num_horizontal_segments + 1) + j;
                    let one = i * (num_horizontal_segments + 1) + next_j;
                    let two = (i + 1) * (num_horizontal_segments + 1) + next_j;
                    let three = (i + 1) * (num_horizontal_segments + 1) + j;
                    [zero, one, three, one, two, three]
                })
            })
            .chain((0..num_horizontal_segments).flat_map(|i| {
                let i_next = i + 1;
                [
                    i,
                    (num_horizontal_segments + 1) * (num_vertical_segments - 1),
                    i_next,
                ]
            }))
            .chain((0..num_horizontal_segments).flat_map(|i| {
                let i_next = i + 1;
                let offset = (num_horizontal_segments + 1) * (num_vertical_segments - 2);
                [
                    i + offset,
                    i_next + offset,
                    num_horizontal_segments * (num_vertical_segments - 1) + 1,
                ]
            }))
            .collect::<Vec<_>>();
        for idx in indices.iter() {
            if *idx >= vertices.len() as u32 {
                let vert_idx = idx / num_horizontal_segments;
                let horiz_idx = idx % num_horizontal_segments;
                panic!(
                    "idx out of range: {}, vert_idx: {}, horiz_idx: {}, len: {}",
                    idx,
                    vert_idx,
                    horiz_idx,
                    vertices.len()
                );
            }
        }
        Self { vertices, indices }
    }

    pub fn xy_rect() -> Self {
        let vertices = vec![
            Vertex {
                pos: Vector4::new(-0.5, -0.5, 0.0, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.5, 0.0, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(0.5, 0.5, 0.0, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(0.5, -0.5, 0.0, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
        ];
        let indices = vec![0, 3, 2, 0, 2, 1, 2, 3, 0, 1, 2, 0];
        Self { vertices, indices }
    }
    pub fn yz_rect() -> Self {
        let vertices = vec![
            Vertex {
                pos: Vector4::new(0.0, -0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.0, -0.5, 0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(0.0, 0.5, 0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(0.0, 0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
        ];
        let indices = vec![0, 3, 2, 0, 2, 1, 2, 3, 0, 1, 2, 0];
        Self { vertices, indices }
    }
    pub fn xz_rect() -> Self {
        let vertices = vec![
            Vertex {
                pos: Vector4::new(-0.5, 0.0, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.0, 0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(0.5, 0.0, 0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(0.5, 0.0, -0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
        ];
        let indices = vec![0, 3, 2, 0, 2, 1, 2, 3, 0, 1, 2, 0];
        Self { vertices, indices }
    }
    pub fn cube() -> Self {
        let vertices = vec![
            Vertex {
                pos: Vector4::new(-0.5, -0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, -0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, 0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, -0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, -0.5, 0.5, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, 0.5, 0.5, 1.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(0.5, 0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(0.5, -0.5, 0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, -0.5, 0.5, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.5, 0.5, 1.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.5, 0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, -0.5, 0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, -0.5, -0.5, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.5, -0.5, 1.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.5, 0.5, 1.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, 0.5, -0.5, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, 0.5, 0.5, 1.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, 0.5, 0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, -0.5, -0.5, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, -0.5, -0.5, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                pos: Vector4::new(0.5, -0.5, 0.5, 1.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                pos: Vector4::new(-0.5, -0.5, 0.5, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
        ];
        #[rustfmt::skip]
        let indices = vec![
            0, 1,2,
            1,3,2,

            4, 5, 6,
            4, 6, 7,

            10, 9, 8,
            11, 10, 8,

            12, 13, 14,
            12, 14, 15,

            16,17,20,
            17,18,20,
/*
            21,24,22,
            22,24,23
            */

        ];
        Self::new(vertices, indices)
    }
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        #[cfg(feature = "validate_models")]
        {
            for index in indices.iter() {
                if *index >= vertices.len() as u32 {
                    panic!(
                        "index: {} is out of range, [0,{}]",
                        index,
                        vertices.len() - 1
                    )
                }
            }
        }

        if vertices.is_empty() {
            panic!("vertex buffer len is zero")
        }
        if indices.is_empty() {
            panic!("index buffer is empty")
        }
        Self { vertices, indices }
    }
}
fn base_lib_to_texture(texture: &base_lib::Texture) -> image::RgbaImage {
    match texture {
        base_lib::Texture::ConstantColor(c) => image::RgbaImage::from_pixel(
            100,
            100,
            image::Rgba([
                (c.red * 255.0) as u8,
                (c.green * 255.0) as u8,
                (c.blue * 255.0) as u8,
                255,
            ]),
        ),
    }
}
pub fn meshes_from_scene(scene: &base_lib::Scene) -> (Vec<Model>, Camera) {
    let models = scene
        .objects
        .iter()
        .flat_map(|object| {
            let texture = match object.material.clone() {
                base_lib::Material::Light(texture) => base_lib_to_texture(&texture),
                base_lib::Material::Lambertian(texture) => base_lib_to_texture(&texture),
            };
            match &object.shape {
                base_lib::Shape::Sphere { radius, origin } => {
                    let mesh = Mesh::sphere(64, 64);
                    let radius = *radius;
                    let transform = AnimationList::new(vec![
                        Rc::new(StaticPosition { position: *origin }),
                        Rc::new(Scale {
                            scale: Vector3::new(radius, radius, radius),
                        }),
                    ]);
                    vec![(mesh, transform)]
                }
                base_lib::Shape::XYRect {
                    center,
                    size_x,
                    size_y,
                } => {
                    let mesh = Mesh::xy_rect();
                    let transform: Vec<Rc<dyn Animation>> = vec![
                        Rc::new(Scale {
                            scale: Vector3::new(2.0 * size_x, 2.0 * size_y, 1.0),
                        }),
                        Rc::new(StaticPosition {
                            position: Point3::new(center.x, center.y, center.z),
                        }),
                    ];
                    vec![(mesh, AnimationList::new(transform))]
                }
                base_lib::Shape::YZRect {
                    center,
                    size_y,
                    size_z,
                } => {
                    let mesh = Mesh::yz_rect();
                    let transform: Vec<Rc<dyn Animation>> = vec![
                        Rc::new(Scale {
                            scale: Vector3::new(1.0, 2.0 * size_y, 2.0 * size_z),
                        }),
                        Rc::new(StaticPosition {
                            position: Point3::new(center.x, center.y, center.z),
                        }),
                    ];
                    vec![(mesh, AnimationList::new(transform))]
                }
                base_lib::Shape::XZRect {
                    center,
                    size_x,
                    size_z,
                } => {
                    let mesh = Mesh::xz_rect();
                    let transform: Vec<Rc<dyn Animation>> = vec![
                        Rc::new(Scale {
                            scale: Vector3::new(2.0 * size_x, 1.0, 2.0 * size_z),
                        }),
                        Rc::new(StaticPosition {
                            position: Point3::new(center.x, center.y, center.z),
                        }),
                    ];
                    vec![(mesh, AnimationList::new(transform))]
                }
                base_lib::Shape::RenderBox {
                    center,
                    size_x,
                    size_y,
                    size_z,
                } => {
                    let mesh = Mesh::cube();
                    let transform: Vec<Rc<dyn Animation>> = vec![
                        Rc::new(Scale {
                            scale: Vector3::new(2.0 * size_x, 2.0 * size_y, 2.0 * size_z),
                        }),
                        Rc::new(StaticPosition {
                            position: Point3::new(center.x, center.y, center.z),
                        }),
                    ];
                    vec![(mesh, AnimationList::new(transform))]
                }
                base_lib::Shape::Voxels(v) => voxel::VoxelWorld::from_voxel_grid(v).build_model(),
            }
            .drain(..)
            .map(|(mesh, animation)| Model {
                animation,
                mesh,
                texture: texture.clone(),
            })
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    (
        models,
        Camera {
            fov: scene.camera.fov,
            aspect_ratio: scene.camera.aspect_ratio,
            near_clip: scene.camera.near_clip,
            far_clip: scene.camera.far_clip,
            position: scene.camera.origin,
            look_at: scene.camera.look_at,
            up: scene.camera.up_vector,
        },
    )
}
pub fn make_meshes() -> (Vec<Model>, Camera) {
    let scene = (base_lib::get_scenarios()[0].1)();

    meshes_from_scene(&scene)
}
#[derive(Clone)]
pub struct AnimationList {
    animations: Vec<Rc<dyn Animation>>,
}
impl AnimationList {
    pub fn new(animations: Vec<Rc<dyn Animation>>) -> Self {
        Self { animations }
    }
    pub fn build_transform_mat(&self, frame_number: usize) -> Matrix4<f32> {
        self.animations
            .iter()
            .rev()
            .map(|anm| anm.get_transform(frame_number).build_matrix())
            .fold(Matrix4::identity(), |acc, x| acc * x)
    }
}
pub trait Animation {
    fn get_transform(&self, frame_number: usize) -> Transform;
}
pub struct StaticPosition {
    pub position: Point3<f32>,
}
impl Animation for StaticPosition {
    fn get_transform(&self, _frame_number: usize) -> Transform {
        Transform {
            position: self.position,
            rotation: Quaternion::from(Euler {
                x: Rad(0.0),
                y: Rad(0.0),
                z: Rad(0.0),
            }),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
pub struct RotateX {
    pub rotate_rate: f32,
}
impl Animation for RotateX {
    fn get_transform(&self, frame_number: usize) -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from(Euler {
                x: Rad(frame_number as f32 * self.rotate_rate),
                y: Rad(0.0),
                z: Rad(0.0),
            }),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
pub struct Orbit {
    pub radius: f32,
    pub orbit_period: f32,
}
impl Animation for Orbit {
    fn get_transform(&self, frame_number: usize) -> Transform {
        let theta = (frame_number as f32 / self.orbit_period) * 2.0 * f32::PI();
        Transform {
            position: Point3::new(theta.sin() * self.radius, 0.0, theta.cos() * self.radius),
            rotation: Quaternion::from(Euler {
                x: Rad(0.0),
                y: Rad(0.0),
                z: Rad(0.0),
            }),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
pub struct Scale {
    pub scale: Vector3<f32>,
}
impl Animation for Scale {
    fn get_transform(&self, _frame_number: usize) -> Transform {
        Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from(Euler {
                x: Rad(0.0),
                y: Rad(0.0),
                z: Rad(0.0),
            }),
            scale: self.scale,
        }
    }
}
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}
impl Transform {
    pub fn build_matrix(&self) -> Matrix4<f32> {
        let rotation_matrix: Matrix4<f32> = self.rotation.into();
        Matrix4::from_translation(self.position.to_homogeneous().truncate())
            * rotation_matrix
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }
}
#[derive(Clone, Debug)]
pub struct Camera {
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub position: Point3<f32>,
    pub look_at: Point3<f32>,
    pub up: Vector3<f32>,
}
impl Camera {
    pub fn make_transform_mat(&self) -> Matrix4<f32> {
        Matrix4::from_diagonal(cgmath::Vector4::new(1.0, -1.0, 1.0, 1.0))
            * cgmath::perspective(
                Deg(self.fov),
                self.aspect_ratio,
                self.near_clip,
                self.far_clip,
            )
            * cgmath::Matrix4::look_at_rh(self.position, self.look_at, self.up)
    }
}
struct RuntimeScenerio {
    mesh_ids: Vec<ArenaIndex>,
    camera_id: usize,
}
pub struct RenderModelIter<'a> {
    iter: generational_arena::Iter<'a, RenderModel>,
}
impl<'a> std::iter::Iterator for RenderModelIter<'a> {
    type Item = (ArenaIndex, &'a RenderModel);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
pub struct EngineEntities {
    meshes: Arena<RenderModel>,
    cameras: Vec<Camera>,
    selected_name: String,
    scenes: HashMap<String, RuntimeScenerio>,
}
impl EngineEntities {
    pub fn new(
        base: &Base,
        allocator: Arc<Mutex<Allocator>>,
        descriptor_pool: &vk::DescriptorPool,
        descriptor_layouts: &[vk::DescriptorSetLayout],
    ) -> Self {
        let raw_scenes = base_lib::get_scenarios();
        let mut meshes = Arena::new();
        let mut cameras = vec![];
        let mut scenes = HashMap::new();
        let mut selected_name = String::new();
        for (name, raw_scene_fn) in raw_scenes.iter() {
            selected_name = name.clone();
            let raw_scene = (*raw_scene_fn)();
            let (scene_mesh, camera) = meshes_from_scene(&raw_scene);
            let mut mesh_ids = vec![];
            for mesh in scene_mesh.iter() {
                let runtime_model = mesh.build_render_model(
                    base,
                    &mut allocator.lock().expect("failed to get allocator"),
                    descriptor_pool,
                    descriptor_layouts,
                );
                let idx = meshes.insert(runtime_model);
                mesh_ids.push(idx);
            }
            let camera_id = cameras.len();
            cameras.push(camera);
            scenes.insert(
                name.to_string(),
                RuntimeScenerio {
                    mesh_ids,
                    camera_id,
                },
            );
        }
        Self {
            meshes,
            cameras,
            scenes,
            selected_name,
        }
    }
    pub fn get_selected_meshes(&self) -> (&Camera, Vec<&RenderModel>) {
        let scene = self.scenes.get(&self.selected_name).unwrap();
        let camera = &self.cameras[scene.camera_id];

        (
            camera,
            scene.mesh_ids.iter().map(|id| &self.meshes[*id]).collect(),
        )
    }
    pub fn names(&self) -> Vec<&str> {
        self.scenes.keys().map(|s| s.as_str()).collect()
    }
    pub fn set_name(&mut self, name: String) {
        self.selected_name = name
    }
    pub unsafe fn free_resources(&mut self, base: &Base, allocator: Arc<Mutex<Allocator>>) {
        for (_idx, model) in self.meshes.drain() {
            model.free_resources(
                base,
                &mut allocator.lock().expect("failed to get allocator"),
            )
        }
    }
    pub fn iter_models(&self) -> RenderModelIter {
        RenderModelIter {
            iter: self.meshes.iter(),
        }
    }
}
