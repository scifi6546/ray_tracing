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
unsafe fn get_addr(device: &ash::Device, buffer: &vk::Buffer) -> vk::DeviceOrHostAddressConstKHR {
    let buffer_device_address_info = vk::BufferDeviceAddressInfo::builder().buffer(*buffer);
    let device_address = device.get_buffer_device_address(&buffer_device_address_info);
    vk::DeviceOrHostAddressConstKHR { device_address }
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
            let vertex_address = get_addr(&base.device, &model.vertex_buffer);
            let index_address = get_addr(&base.device, &model.index_buffer);

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
            println!("triangles\n{:#?}", triangles);
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
            let range_arr: [&[vk::AccelerationStructureBuildRangeInfoKHR]; 1] =
                [&build_range_infos];
            let scratch_buffer_info = vk::BufferCreateInfo::builder()
                .size(build_size.build_scratch_size)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(
                    vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                        | vk::BufferUsageFlags::STORAGE_BUFFER,
                );
            let scratch_buffer = base
                .device
                .create_buffer(&scratch_buffer_info, None)
                .expect("failed to create scratch buffer");
            let scratch_reqs = base.device.get_buffer_memory_requirements(scratch_buffer);
            let scratch_allocation = allocator
                .lock()
                .expect("failed to get alloc")
                .allocate(&AllocationCreateDesc {
                    name: "scratch allocation",
                    requirements: scratch_reqs,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                })
                .expect("failed to get alloc");
            base.device
                .bind_buffer_memory(
                    scratch_buffer,
                    scratch_allocation.memory(),
                    scratch_allocation.offset(),
                )
                .expect("failed to bind memory");
            let scratch_addres_info = vk::BufferDeviceAddressInfo::builder().buffer(scratch_buffer);
            let scratch_address = base.device.get_buffer_device_address(&scratch_addres_info);
            let build_type = [vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .geometries(&geo)
                .scratch_data(vk::DeviceOrHostAddressKHR {
                    device_address: scratch_address,
                })
                .dst_acceleration_structure(acceleration_structure)
                .build()];
            let t_fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
            let fence = base
                .device
                .create_fence(&t_fence_info, None)
                .expect("failed to create fence");
            let command_buffer_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(1)
                .command_pool(base.pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let command_buffer = base
                .device
                .allocate_command_buffers(&command_buffer_info)
                .expect("failed to get buffer")[0];
            base.device
                .device_wait_idle()
                .expect("failed to wait idle before cmd_buffer");
            record_submit_commandbuffer(
                &base.device,
                command_buffer,
                fence,
                base.present_queue,
                &[],
                &[],
                &[],
                |device, command_buffer| {
                    println!("{:#?}", build_type);
                    println!("{:#?}", *build_type[0].p_geometries);
                    println!("{:#?}", range_arr);
                    let mut barrier = [vk::MemoryBarrier2::builder()
                        .dst_access_mask(vk::AccessFlags2::ACCELERATION_STRUCTURE_WRITE_KHR)
                        .dst_stage_mask(vk::PipelineStageFlags2::ACCELERATION_STRUCTURE_BUILD_KHR)
                        .build()];

                    let dep_info = vk::DependencyInfo::builder().memory_barriers(&barrier);
                    device.cmd_pipeline_barrier2(command_buffer, &dep_info);

                    raytracing_state
                        .acceleration_structure
                        .cmd_build_acceleration_structures(command_buffer, &build_type, &range_arr);
                },
            );
            base.device.device_wait_idle().expect("failed to wait idle");
            base.device
                .wait_for_fences(&[fence], true, u64::MAX)
                .expect("failed to wait for fence");
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
            allocator
                .lock()
                .expect("failed to get alloc lock")
                .free(scratch_allocation)
                .expect("failed to free");
            base.device.destroy_buffer(scratch_buffer, None);
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
