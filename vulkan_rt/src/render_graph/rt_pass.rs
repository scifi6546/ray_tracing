use super::{Base, PassBase, RayTracingState, VulkanOutput, VulkanOutputType, VulkanPass};
use crate::{
    prelude::{RenderModel, Vertex},
    record_submit_commandbuffer,
};

use ash::vk;
use generational_arena::Index as ArenaIndex;
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, Allocator},
    MemoryLocation,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
unsafe fn get_device_address(device: &ash::Device, buffer: &vk::Buffer) -> vk::DeviceAddress {
    let buffer_device_address_info = vk::BufferDeviceAddressInfo::builder().buffer(*buffer);
    device.get_buffer_device_address(&buffer_device_address_info)
}
unsafe fn get_addr_const(
    device: &ash::Device,
    buffer: &vk::Buffer,
) -> vk::DeviceOrHostAddressConstKHR {
    vk::DeviceOrHostAddressConstKHR {
        device_address: get_device_address(device, buffer),
    }
}
unsafe fn get_addr(device: &ash::Device, buffer: &vk::Buffer) -> vk::DeviceOrHostAddressKHR {
    vk::DeviceOrHostAddressKHR {
        device_address: get_device_address(device, buffer),
    }
}
struct ModelAccelerationStructure {
    buffer: vk::Buffer,
    allocation: Option<Allocation>,
    acceleration_structure: vk::AccelerationStructureKHR,
}
impl ModelAccelerationStructure {
    fn new(
        base: &Base,
        allocator: Arc<Mutex<Allocator>>,
        raytracing_state: &RayTracingState,
        model: &RenderModel,
    ) -> Self {
        let queue_family_indicies = [base.queue_family_index];
        unsafe {
            let vertex_address = get_addr_const(&base.device, &model.vertex_buffer);
            let index_address = get_addr_const(&base.device, &model.index_buffer);

            let triangles = vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                .vertex_format(vk::Format::R32G32B32_SFLOAT)
                .vertex_data(vertex_address)
                .vertex_stride(Vertex::stride() as vk::DeviceSize)
                .max_vertex(model.max_index as u32)
                .index_type(RenderModel::index_type())
                .index_data(index_address)
                .transform_data(vk::DeviceOrHostAddressConstKHR {
                    host_address: std::ptr::null_mut(),
                })
                .build();

            let geo = [vk::AccelerationStructureGeometryKHR::builder()
                .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
                .geometry(vk::AccelerationStructureGeometryDataKHR { triangles })
                .build()];
            let build_type = [vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .geometries(&geo)
                .build()];
            let build_size = raytracing_state
                .acceleration_structure
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::HOST,
                    &build_type[0],
                    &[1],
                );
            println!(
                "model size: {} scratch size: {}",
                build_size.acceleration_structure_size, build_size.build_scratch_size
            );
            let info = vk::BufferCreateInfo::builder()
                .size(build_size.acceleration_structure_size)
                .queue_family_indices(&queue_family_indicies)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR);
            let buffer = base.device.create_buffer(&info, None).expect("buffer");
            let memory_reqs = base.device.get_buffer_memory_requirements(buffer);
            let allocation = allocator
                .lock()
                .expect("failed to get lock")
                .allocate(&AllocationCreateDesc {
                    name: "",
                    requirements: memory_reqs,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                })
                .expect("failed to get allocation");

            base.device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .expect("failed to bind memory");
            let acceleration_structure_create_info =
                vk::AccelerationStructureCreateInfoKHR::builder()
                    .buffer(buffer)
                    .offset(0)
                    .size(build_size.acceleration_structure_size)
                    .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL);
            let acceleration_structure = raytracing_state
                .acceleration_structure
                .create_acceleration_structure(&acceleration_structure_create_info, None)
                .expect("failed to create acceleration structure");

            let build_range_infos = [vk::AccelerationStructureBuildRangeInfoKHR::builder()
                .primitive_count(model.num_triangles())
                .primitive_offset(0)
                .first_vertex(0)
                .build()];
            let scratch_buffer_info = vk::BufferCreateInfo::builder()
                .size(build_size.build_scratch_size)
                .queue_family_indices(&queue_family_indicies)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(
                    vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                        | vk::BufferUsageFlags::STORAGE_BUFFER
                        | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                );
            let scratch_buffer = base
                .device
                .create_buffer(&scratch_buffer_info, None)
                .expect("failed to create buffer");
            let scratch_memory_reqs = base.device.get_buffer_memory_requirements(scratch_buffer);
            let scratch_memory = allocator
                .lock()
                .expect("failed to get lock")
                .allocate(&AllocationCreateDesc {
                    name: "scratch memory",
                    requirements: scratch_memory_reqs,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                })
                .expect("failed to bind scratch memory");
            base.device
                .bind_buffer_memory(
                    scratch_buffer,
                    scratch_memory.memory(),
                    scratch_memory.offset(),
                )
                .expect("failed to bind memory");
            let range_arr: [&[vk::AccelerationStructureBuildRangeInfoKHR]; 1] =
                [&build_range_infos];

            let build_type = [vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .geometries(&geo)
                .scratch_data(get_addr(&base.device, &scratch_buffer))
                .dst_acceleration_structure(acceleration_structure)
                .build()];
            base.device.device_wait_idle().expect("failed to wait idle");
            record_submit_commandbuffer(
                &base.device,
                base.setup_command_buffer,
                base.setup_commands_reuse_fence,
                base.present_queue,
                &[],
                &[],
                &[],
                |device, command_buffer| {
                    raytracing_state
                        .acceleration_structure
                        .cmd_build_acceleration_structures(command_buffer, &build_type, &range_arr);
                },
            );
            base.device.device_wait_idle().expect("failed to wait idle");
            base.device
                .wait_for_fences(&[base.setup_commands_reuse_fence], true, u64::MAX)
                .expect("failed to wait for fence");
            allocator
                .lock()
                .expect("failed to get allocator")
                .free(scratch_memory)
                .expect("failed to free");
            base.device.destroy_buffer(scratch_buffer, None);
            /*
            raytracing_state
                .acceleration_structure
                .build_acceleration_structures(
                    vk::DeferredOperationKHR::null(),
                    &build_type,
                    &range_arr,
                )
                .expect("failed to build bottom level accceleration structure");

             */
            Self {
                buffer,
                allocation: Some(allocation),
                acceleration_structure,
            }
        }
    }
    fn free(&mut self, base: &PassBase) {
        unsafe {
            base.base
                .device
                .device_wait_idle()
                .expect("failed to wait idle");
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
pub struct RtPass {
    allocation: Option<Allocation>,
    buffer: vk::Buffer,
    acceleration_structure: vk::AccelerationStructureKHR,
    model_acceleration_structures: HashMap<ArenaIndex, ModelAccelerationStructure>,
}
impl RtPass {
    pub fn new(pass_base: &PassBase) -> Self {
        const ALLOC_SIZE: usize = 20 * 256;

        unsafe {
            let queue_family_indicies = [pass_base.base.queue_family_index];
            let model_acceleration_structures = pass_base
                .engine_entities
                .borrow()
                .iter_models()
                .map(|(idx, model)| {
                    (
                        idx,
                        ModelAccelerationStructure::new(
                            &pass_base.base,
                            pass_base.allocator.clone(),
                            &pass_base.raytracing_state,
                            model,
                        ),
                    )
                })
                .collect();

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
                model_acceleration_structures,
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
            for (_idx, accel) in self.model_acceleration_structures.iter_mut() {
                accel.free(base);
            }
        }
    }
}
