use super::{PassBase, VulkanOutput, VulkanOutputType, VulkanPass};
use crate::prelude::{RenderModel, Vertex};
use ash::vk;
use ash::vk::DeviceSize;
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
            unsafe fn get_addr(
                device: &ash::Device,
                buffer: &vk::Buffer,
            ) -> vk::DeviceOrHostAddressConstKHR {
                let buffer_device_address_info =
                    vk::BufferDeviceAddressInfo::builder().buffer(*buffer);
                let device_address = device.get_buffer_device_address(&buffer_device_address_info);
                vk::DeviceOrHostAddressConstKHR { device_address }
            }
            for (idx, model) in pass_base.engine_entities.borrow().iter_models() {
                let vertex_address = get_addr(&pass_base.base.device, &model.vertex_buffer);
                let index_address = get_addr(&pass_base.base.device, &model.index_buffer);

                let triangles = vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                    .vertex_format(vk::Format::R32G32B32_SFLOAT)
                    .vertex_data(vertex_address)
                    .vertex_stride(Vertex::stride() as DeviceSize)
                    .max_vertex(model.max_index as u32)
                    .index_type(RenderModel::index_type())
                    .index_data(index_address)
                    .build();
                /*
                vk::AccelerationStructureGeometryDataKHR {
                                        triangles: vk::AccelerationStructureGeometryTrianglesDataKHR {
                                            s_type: Default::default(),
                                            p_next: (),
                                            vertex_format: Default::default(),
                                            vertex_data: Default::default(),
                                            vertex_stride: 0,
                                            max_vertex: 0,
                                            index_type: Default::default(),
                                            index_data: Default::default(),
                                            transform_data: Default::default(),
                                        },
                                    }
                 */
                let geo = [vk::AccelerationStructureGeometryKHR::builder()
                    .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
                    .geometry(vk::AccelerationStructureGeometryDataKHR { triangles })
                    .build()];
                let build_type = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                    .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                    .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                    .geometries(&geo);
                let num = pass_base
                    .raytracing_state
                    .acceleration_structure
                    .get_acceleration_structure_build_sizes(
                        vk::AccelerationStructureBuildTypeKHR::HOST,
                        &build_type,
                        &[1],
                    );
                println!("model size: {}", num.acceleration_structure_size);
            }
            /*
            let geometries = [vk::AccelerationStructureGeometryKHR::builder()
                .geometry(vk::AccelerationStructureGeometryDataKHR {})
                .build()];
            let build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .geometries(&geometries);
            let sizes = pass_base
                .raytracing_state
                .acceleration_structure
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::HOST,
                    &build_info,
                    &[10],
                );
            println!("sizes: {:?}", sizes);

             */
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
