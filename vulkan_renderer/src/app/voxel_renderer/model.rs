use ash::{Device, util::Align, vk};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::mem::offset_of;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RenderModelVertex {
    pub position: [f32; 4],
}
impl RenderModelVertex {
    pub const fn input_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
    pub const fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 1] {
        [vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Self, position) as u32,
        }]
    }
}
pub type Index = u32;
pub struct RenderModel {
    pub index_buffer: vk::Buffer,
    index_allocation: Allocation,
    pub vertex_buffer: vk::Buffer,
    vertex_allocation: Allocation,
    number_indices: usize,
    //  texture: PresentTexture,
}
impl RenderModel {
    pub const VK_INDEX_TYPE: vk::IndexType = vk::IndexType::UINT32;
    const fn index_type_alignment() -> usize {
        align_of::<Index>()
    }
    const fn vertex_alignment() -> usize {
        align_of::<RenderModelVertex>()
    }
    pub fn number_indices(&self) -> usize {
        self.number_indices
    }
    pub fn new_rectangle(device: &Device, allocator: &mut Allocator) -> Self {
        let vertices = [
            RenderModelVertex {
                position: [-1., -1., 0.5, 1.],
            },
            RenderModelVertex {
                position: [-1., 1., 0.5, 1.],
            },
            RenderModelVertex {
                position: [1., 1., 0.5, 1.],
            },
            RenderModelVertex {
                position: [1., -1., 0.5, 1.],
            },
        ];
        let indices = [0, 3, 1, 3, 2, 1];
        Self::new(&vertices, &indices, device, allocator)
    }
    pub fn new(
        vertices: &[RenderModelVertex],
        indices: &[Index],
        device: &Device,
        allocator: &mut Allocator,
    ) -> Self {
        unsafe {
            let index_buffer_info = vk::BufferCreateInfo::default()
                .size(size_of_val(indices) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let index_buffer = device
                .create_buffer(&index_buffer_info, None)
                .expect("failed to get index buffer");
            let index_buffer_memory_requirements =
                device.get_buffer_memory_requirements(index_buffer);
            let index_allocation = allocator
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
            device
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

            let vertex_buffer = device
                .create_buffer(&vertex_input_buffer, None)
                .expect("failed to create vertex buffer");

            let vertex_buffer_memory_requirements =
                device.get_buffer_memory_requirements(vertex_buffer);
            let vertex_allocation = allocator
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

            device
                .bind_buffer_memory(
                    vertex_buffer,
                    vertex_allocation.memory(),
                    vertex_allocation.offset(),
                )
                .expect("failed to bind memory");

            Self {
                index_buffer,
                index_allocation,
                vertex_buffer,
                vertex_allocation,
                number_indices: indices.len(),
            }
        }
    }
    pub fn free(self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.device_wait_idle().expect("failed to wait idle");
            allocator
                .free(self.index_allocation)
                .expect("failed to free index allocation");

            device.destroy_buffer(self.index_buffer, None);
            allocator
                .free(self.vertex_allocation)
                .expect("failed to free vertex buffer");
            device.destroy_buffer(self.vertex_buffer, None);
        }
    }
}
