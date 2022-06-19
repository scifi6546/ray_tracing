use cgmath::{
    num_traits::FloatConst, Euler, Matrix, Matrix4, Point3, Quaternion, Rad, SquareMatrix, Vector3,
    Zero,
};
use std::rc::Rc;
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
pub fn Mat4ToBytes<'a>(mat: &'a cgmath::Matrix4<f32>) -> &'a [u8] {
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
        let mut vertices = (0..num_segments)
            .flat_map(|i| {
                let theta = 2.0 * f32::PI() * ((i) as f32 / num_segments as f32);
                let x = theta.sin();
                let z = theta.cos();
                let v = ((i) as f32 / num_segments as f32);
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
                println!(
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
    fn get_transform(&self, frame_number: usize) -> Transform {
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
