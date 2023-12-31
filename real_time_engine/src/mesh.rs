use bevy::prelude::*;
use cgmath::Point3;
use wgpu::util::DeviceExt;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub(crate) position: [f32; 3],
    pub(crate) tex_coords: [f32; 2],
}
impl Vertex {
    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ],
        }
    }
}
#[derive(Component)]
pub(crate) struct RuntimeMesh {
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) num_indices: usize,
}
impl RuntimeMesh {
    pub(crate) fn from_mesh(mesh: &Mesh, device: &wgpu::Device) -> Self {
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self {
            index_buffer,
            vertex_buffer,
            num_indices: mesh.indices.len(),
        }
    }
}
pub(crate) struct Mesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u16>,
}
impl Mesh {
    pub(crate) fn add_offset(&mut self, offset: Point3<f32>) {
        for vertex in self.vertices.iter_mut() {
            vertex.position[0] += offset.x;
            vertex.position[1] += offset.y;
            vertex.position[2] += offset.z;
        }
    }
    /// adds a mesh to self
    pub(crate) fn add_mesh(&mut self, mesh: &Mesh) {
        let self_vertices_len = self.vertices.len();
        self.vertices
            .reserve(self_vertices_len + (&mesh).vertices.len());
        for vertex in mesh.vertices.iter() {
            self.vertices.push(vertex.clone());
        }
        let self_indeces_len = self.indices.len();
        self.indices.reserve(self_indeces_len + mesh.indices.len());
        for index in mesh.indices.iter() {
            self.indices.push(*index + self_vertices_len as u16);
        }
    }
    pub(crate) fn cube() -> Self {
        Self {
            vertices: vec![
                //0
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    tex_coords: [0.0, 0.0],
                },
                //1
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    tex_coords: [0.0, 1.0],
                },
                //2
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    tex_coords: [0.0, 1.0],
                },
                //3
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    tex_coords: [1.0, 0.0],
                },
                //face 1 4
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    tex_coords: [0.0, 0.0],
                },
                //5
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    tex_coords: [1.0, 0.0],
                },
                //6
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    tex_coords: [1.0, 0.0],
                },
                //7
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    tex_coords: [1.0, 0.0],
                },
                //face 2 8
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    tex_coords: [0.0, 0.0],
                },
                //9
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    tex_coords: [0.0, 1.0],
                },
                //10
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    tex_coords: [0.0, 1.0],
                },
                //11
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    tex_coords: [1.0, 0.0],
                }, //face 3 12
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    tex_coords: [0.0, 0.0],
                }, //13
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    tex_coords: [1.0, 0.0],
                },
                //14
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    tex_coords: [1.0, 0.0],
                }, //15
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    tex_coords: [1.0, 0.0],
                },
                //face 4 16
                Vertex {
                    position: [-0.5, -0.5, -0.5],
                    tex_coords: [0.0, 0.0],
                }, //17
                Vertex {
                    position: [-0.5, -0.5, 0.5],
                    tex_coords: [0.0, 1.0],
                },
                //18
                Vertex {
                    position: [0.5, -0.5, 0.5],
                    tex_coords: [1.0, 1.0],
                }, //19
                Vertex {
                    position: [0.5, -0.5, -0.5],
                    tex_coords: [0.0, 1.0],
                },
                //face 5 20
                Vertex {
                    position: [-0.5, 0.5, -0.5],
                    tex_coords: [0.0, 0.0],
                }, //21
                Vertex {
                    position: [-0.5, 0.5, 0.5],
                    tex_coords: [0.0, 1.0],
                },
                //22
                Vertex {
                    position: [0.5, 0.5, 0.5],
                    tex_coords: [1.0, 1.0],
                }, //23
                Vertex {
                    position: [0.5, 0.5, -0.5],
                    tex_coords: [0.0, 1.0],
                },
            ],
            indices: vec![
                //0
                2, 0, 1, 3, 0, 2, //face 1
                4, 6, 5, 7, 6, 4, //face 2
                8, 10, 9, 8, 11, 10, //face 3
                14, 12, 13, 14, 15, 12, //face 4
                16, 19, 17, 17, 19, 18, //face 5
                23, 20, 21, 23, 21, 22,
            ],
        }
    }
    pub(crate) fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
    pub(crate) fn pentagon() -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: [-0.0868241, 0.49240386, 0.0],
                    tex_coords: [0.4131759, 0.99240386],
                }, // A
                Vertex {
                    position: [-0.49513406, 0.06958647, 0.0],
                    tex_coords: [0.0048659444, 0.56958647],
                }, // B
                Vertex {
                    position: [-0.21918549, -0.44939706, 0.0],
                    tex_coords: [0.28081453, 0.05060294],
                }, // C
                Vertex {
                    position: [0.35966998, -0.3473291, 0.0],
                    tex_coords: [0.85967, 0.1526709],
                }, // D
                Vertex {
                    position: [0.44147372, 0.2347359, 0.0],
                    tex_coords: [0.9414737, 0.7347359],
                },
            ],
            indices: vec![0, 1, 4, 1, 2, 4, 2, 3, 4],
        }
    }
}
