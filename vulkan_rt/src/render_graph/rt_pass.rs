use super::{PassBase, VulkanOutput, VulkanOutputType, VulkanPass};
use ash::vk;
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc},
    MemoryLocation,
};

pub struct RtPass {
    allocation: Option<Allocation>,
    buffer: vk::Buffer,
    acceleration_structure: vk::AccelerationStructureKHR,
}
impl RtPass {
    pub fn new(pass_base: &PassBase) -> Self {
        const ALLOC_SIZE: usize = 20 * 256;
        unsafe {
            let queue_family_indicies = [pass_base.base.queue_family_index];
            let info = vk::BufferCreateInfo::builder()
                .size(ALLOC_SIZE as vk::DeviceSize)
                .queue_family_indices(&queue_family_indicies)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR);
            let buffer = pass_base
                .base
                .device
                .create_buffer(&info, None)
                .expect("buffer");
            let memory_reqs = pass_base.base.device.get_buffer_memory_requirements(buffer);
            let allocation = pass_base
                .allocator
                .lock()
                .expect("failed to get lock")
                .allocate(&AllocationCreateDesc {
                    name: "",
                    requirements: memory_reqs,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                })
                .expect("failed to get allocation");

            pass_base
                .base
                .device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .expect("failed to bind memory");
            let info = vk::AccelerationStructureCreateInfoKHR::builder()
                .buffer(buffer)
                .offset(0)
                .size(allocation.size())
                .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL);
            let acceleration_structure = pass_base
                .raytracing_state
                .acceleration_structure
                .create_acceleration_structure(&info, None)
                .expect("failed to create structure");

            Self {
                allocation: Some(allocation),
                buffer,
                acceleration_structure,
            }
        }
    }
}
impl VulkanPass for RtPass {
    fn get_dependencies(&self) -> Vec<VulkanOutputType> {
        vec![]
    }

    fn get_output(&self) -> Vec<VulkanOutputType> {
        vec![VulkanOutputType::Empty]
    }

    fn process(&mut self, base: &PassBase, input: Vec<&VulkanOutput>) -> Vec<VulkanOutput> {
        vec![VulkanOutput::Empty]
    }

    fn free(&mut self, base: &PassBase) {
        unsafe {
            base.raytracing_state
                .acceleration_structure
                .destroy_acceleration_structure(self.acceleration_structure, None);
            base.base.device.destroy_buffer(self.buffer, None);
            base.allocator
                .lock()
                .expect("failed to get allocator")
                .free(self.allocation.take().unwrap())
                .expect("failed to free memory");
        }
    }
}
