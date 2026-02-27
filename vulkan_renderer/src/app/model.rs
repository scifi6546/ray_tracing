use super::Vertex;
use crate::utils::find_memorytype_index;
use ash::{Device, util::Align, vk};
use std::mem::size_of_val;
pub struct Model {
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub vertex_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
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
        device_memory_properties: vk::PhysicalDeviceMemoryProperties,
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
            let index_buffer_memory_index = find_memorytype_index(
                &index_buffer_memory_requirements,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("failed to get memory type index");
            let index_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(index_buffer_memory_requirements.size)
                .memory_type_index(index_buffer_memory_index);

            let index_buffer_memory = device
                .allocate_memory(&index_allocate_info, None)
                .expect("failed to allocate memory");
            let index_ptr = device
                .map_memory(
                    index_buffer_memory,
                    0,
                    index_buffer_memory_requirements.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map");
            let mut index_slice = Align::new(
                index_ptr,
                Self::index_type_alignment() as u64,
                index_buffer_memory_requirements.size,
            );
            index_slice.copy_from_slice(indices);
            device.unmap_memory(index_buffer_memory);
            device
                .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
                .expect("failed to bind index buffer memory to index buffer");

            let vertex_input_buffer = vk::BufferCreateInfo::default()
                .size(size_of_val(vertices) as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            println!("vertex size: {}", size_of::<Vertex>());
            let vertex_buffer = device
                .create_buffer(&vertex_input_buffer, None)
                .expect("failed to create vertex buffer");
            let vertex_buffer_memory_requirements =
                device.get_buffer_memory_requirements(vertex_buffer);
            let vertex_buffer_memory_index = find_memorytype_index(
                &vertex_buffer_memory_requirements,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("failed to get memory requirements");
            let vertex_buffer_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(vertex_buffer_memory_requirements.size)
                .memory_type_index(vertex_buffer_memory_index);
            let vertex_buffer_memory = device
                .allocate_memory(&vertex_buffer_allocate_info, None)
                .expect("failed to allocate memory for vertex buffer");
            let vertex_ptr = device
                .map_memory(
                    vertex_buffer_memory,
                    0,
                    vertex_buffer_memory_requirements.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map vertex buffer to device space");
            let mut vertex_align = Align::new(
                vertex_ptr,
                align_of::<Vertex>() as u64,
                vertex_buffer_memory_requirements.size,
            );
            vertex_align.copy_from_slice(vertices);
            device.unmap_memory(vertex_buffer_memory);
            device
                .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
                .expect("failed to bind memory");
            Self {
                vertex_buffer_memory,
                vertex_buffer,
                index_buffer_memory,
                index_buffer,
            }
        }
    }
    pub fn free(&mut self, device: &Device) {
        unsafe {
            device.device_wait_idle().expect("failed to free");
            device.free_memory(self.index_buffer_memory, None);
            device.destroy_buffer(self.index_buffer, None);

            device.free_memory(self.vertex_buffer_memory, None);
            device.destroy_buffer(self.vertex_buffer, None);
        }
    }
}
