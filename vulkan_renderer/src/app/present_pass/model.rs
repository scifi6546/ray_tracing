use super::{super::SetupCommandBuffer, descriptors::PresentDescriptors, texture::PresentTexture};
use ash::{Device, util::Align, vk};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::mem::{offset_of, size_of_val};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct PresentVertex {
    pub pos: [f32; 4],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}
impl PresentVertex {
    pub const fn input_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
    pub const fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Self, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, uv) as u32,
            },
        ]
    }
}

pub struct PresentModelInfo<'a> {
    pub device: &'a Device,
    pub allocator: &'a mut Allocator,
    pub setup_command_buffer: &'a mut SetupCommandBuffer,
    pub present_queue: &'a vk::Queue,
    pub descriptors: &'a PresentDescriptors,
}

pub struct PresentModel {
    texture: PresentTexture,
    pub vertex_allocation: Allocation,
    pub vertex_buffer: vk::Buffer,
    pub index_allocation: Allocation,
    pub index_buffer: vk::Buffer,
    // does not need to get freed
    //pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub num_indices: usize,
}
#[derive(Clone, Copy)]
pub struct PresentRectangle {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub z_index: f32,
}
type Index = u32;
impl PresentModel {
    const fn index_type_alignment() -> usize {
        align_of::<Index>()
    }
    const fn vertex_alignment() -> usize {
        align_of::<PresentVertex>()
    }
    pub fn new_rectangle(
        rectangle: PresentRectangle,
        texture: PresentTexture,
        vulkan_info: &mut PresentModelInfo,
    ) -> Self {
        let vertices = [
            PresentVertex {
                pos: [rectangle.min_x, rectangle.min_y, rectangle.z_index, 1.],
                color: [1., 1., 1., 1.],
                uv: [0., 0.],
            },
            PresentVertex {
                pos: [rectangle.min_x, rectangle.max_y, rectangle.z_index, 1.],
                color: [1., 1., 1., 1.],
                uv: [0., 1.],
            },
            PresentVertex {
                pos: [rectangle.max_x, rectangle.max_y, rectangle.z_index, 1.],
                color: [1., 1., 1., 1.],
                uv: [1., 1.],
            },
            PresentVertex {
                pos: [rectangle.max_x, rectangle.min_y, rectangle.z_index, 1.],
                color: [1., 1., 0., 1.],
                uv: [1., 0.],
            },
        ];

        let indices = [0, 3, 1, 3, 2, 1];
        Self::new(&vertices, &indices, texture, vulkan_info)
    }
    pub fn new_rectangle_with_buffer(
        rectangle: PresentRectangle,
        texture_buffer: &[u8],
        vulkan_info: &mut PresentModelInfo,
    ) -> Self {
        let texture = PresentTexture::new(vulkan_info, texture_buffer);
        Self::new_rectangle(rectangle, texture, vulkan_info)
    }
    pub fn new(
        vertices: &[PresentVertex],
        indices: &[u32],
        texture: PresentTexture,
        vulkan_info: &mut PresentModelInfo,
    ) -> Self {
        unsafe {
            let index_buffer_info = vk::BufferCreateInfo::default()
                .size(size_of_val(indices) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let index_buffer = vulkan_info
                .device
                .create_buffer(&index_buffer_info, None)
                .expect("failed to get index buffer");
            let index_buffer_memory_requirements = vulkan_info
                .device
                .get_buffer_memory_requirements(index_buffer);
            let index_allocation = vulkan_info
                .allocator
                .allocate(&AllocationCreateDesc {
                    name: "Model Index Buffer",
                    requirements: index_buffer_memory_requirements,
                    location: MemoryLocation::CpuToGpu,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .expect("failed to allocate");
            {
                let index_ptr = index_allocation.mapped_ptr().unwrap();

                let mut index_slice = Align::new(
                    index_ptr.as_ptr(),
                    Self::index_type_alignment() as u64,
                    index_buffer_memory_requirements.size,
                );
                index_slice.copy_from_slice(indices);
            }

            vulkan_info
                .device
                .bind_buffer_memory(
                    index_buffer,
                    index_allocation.memory(),
                    index_allocation.offset(),
                )
                .expect("failed to bind index buffer memory to index buffer");

            let vertex_input_buffer = vk::BufferCreateInfo::default()
                .size(size_of_val(vertices) as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let vertex_buffer = vulkan_info
                .device
                .create_buffer(&vertex_input_buffer, None)
                .expect("failed to create vertex buffer");

            let vertex_buffer_memory_requirements = vulkan_info
                .device
                .get_buffer_memory_requirements(vertex_buffer);
            let vertex_allocation = vulkan_info
                .allocator
                .allocate(&AllocationCreateDesc {
                    name: "vertex buffer allocation",
                    requirements: vertex_buffer_memory_requirements,
                    location: MemoryLocation::CpuToGpu,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .expect("failed to create allocation");

            {
                let vertex_ptr = vertex_allocation
                    .mapped_ptr()
                    .expect("failed to get a vertex pointer");
                let mut vertex_slice = Align::new(
                    vertex_ptr.as_ptr(),
                    Self::vertex_alignment() as u64,
                    index_buffer_memory_requirements.size,
                );
                vertex_slice.copy_from_slice(vertices);
            }

            vulkan_info
                .device
                .bind_buffer_memory(
                    vertex_buffer,
                    vertex_allocation.memory(),
                    vertex_allocation.offset(),
                )
                .expect("failed to bind memory");

            Self {
                texture,
                vertex_allocation,
                num_indices: indices.len(),
                vertex_buffer,
                index_allocation,
                index_buffer,
            }
        }
    }
    pub fn descriptor_sets(&self) -> &[vk::DescriptorSet] {
        self.texture.descriptor_sets()
    }
    pub fn free(self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.device_wait_idle().expect("failed to free");

            self.texture.free(device, allocator);

            allocator
                .free(self.vertex_allocation)
                .expect("failed to free vertex allocation");
            allocator
                .free(self.index_allocation)
                .expect("failed to free index allocation");

            device.destroy_buffer(self.index_buffer, None);

            device.destroy_buffer(self.vertex_buffer, None);
        }
    }
}
