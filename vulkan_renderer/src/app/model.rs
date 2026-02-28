use super::Vertex;

use ash::{Device, util::Align, vk};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::mem::size_of_val;
pub struct Model {
    pub vertex_allocation: Allocation,
    pub vertex_buffer: vk::Buffer,
    pub index_allocation: Allocation,
    pub index_buffer: vk::Buffer,
}
type Index = u32;
impl Model {
    const fn index_type_alignment() -> usize {
        align_of::<Index>()
    }
    pub fn new(
        vertices: &[Vertex],
        indices: &[u32],
        device: &Device,
        allocator: &mut Allocator,
    ) -> Self {
        println!(
            "indices size_of_val: {}, vertices size_of_val: {}",
            size_of_val(indices),
            size_of_val(vertices)
        );
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
                    Self::index_type_alignment() as u64,
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
                vertex_allocation,

                vertex_buffer,
                index_allocation,
                index_buffer,
            }
        }
    }
    pub fn free(self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.device_wait_idle().expect("failed to free");
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
