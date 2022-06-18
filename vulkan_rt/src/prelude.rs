use cgmath::{num_traits::FloatConst, Matrix};

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
                println!("r: {}", r);
                (0..num_horizontal_segments).map(move |j| {
                    let theta = 2.0 * f32::PI() * ((j) as f32 / num_horizontal_segments as f32);

                    let x = theta.sin() * r;
                    let y = phi.sin();
                    let z = theta.cos() * r;

                    println!("x: {}, y: {}, z: {}", x, y, z);
                    Vertex {
                        pos: Vector4::new(x, y, z, 1.0),
                        uv: Vector2::new(
                            j as f32 / (num_horizontal_segments) as f32,
                            i as f32 / num_vertical_segments as f32,
                        ),
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
                    let next_j = if j + 1 < num_horizontal_segments {
                        j + 1
                    } else {
                        0
                    };
                    let zero = i * num_horizontal_segments + j;
                    let one = i * num_horizontal_segments + next_j;
                    let two = (i + 1) * num_horizontal_segments + next_j;
                    let three = (i + 1) * num_horizontal_segments + j;
                    [zero, one, three, one, two, three]
                })
            })
            .chain((0..num_horizontal_segments).flat_map(|i| {
                let i_next = if i + 1 < num_horizontal_segments {
                    i + 1
                } else {
                    0
                };
                [
                    i,
                    num_horizontal_segments * (num_vertical_segments - 1),
                    i_next,
                ]
            }))
            .chain((0..num_horizontal_segments).flat_map(|i| {
                let i_next = if i + 1 < num_horizontal_segments {
                    i + 1
                } else {
                    0
                };
                let offset = num_horizontal_segments * (num_vertical_segments - 2);
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
