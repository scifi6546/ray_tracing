use super::Base;
use crate::render_graph::rt_pass::descriptor_sets::RayTracingDescriptorSets;
use crate::{
    prelude::{RenderModel, Vertex},
    record_submit_commandbuffer,
    render_graph::{PassBase, RayTracingState},
};
use ash::vk;
use cgmath::Matrix4;
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
    MemoryLocation,
};
use std::{
    ffi::c_void,
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
unsafe fn get_device_or_host_address(
    device: &ash::Device,
    buffer: &vk::Buffer,
) -> vk::DeviceOrHostAddressKHR {
    vk::DeviceOrHostAddressKHR {
        device_address: get_device_address(device, buffer),
    }
}

pub struct ModelAccelerationStructure {
    buffer: vk::Buffer,
    allocation: Option<Allocation>,
    acceleration_structure: vk::AccelerationStructureKHR,
    transform_matrix: Matrix4<f32>,
    number_triangles: u32,
}

impl ModelAccelerationStructure {
    pub fn new(
        base: &Base,
        allocator: Arc<Mutex<Allocator>>,
        raytracing_state: &RayTracingState,
        model: &RenderModel,
    ) -> Result<Self, vk::Result> {
        let queue_family_indicies = [base.queue_family_index];
        unsafe {
            let vertex_address = get_addr_const(&base.device, &model.vertex_buffer);
            let index_address = get_addr_const(&base.device, &model.index_buffer);

            let triangles = vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                .vertex_format(Vertex::position_format())
                .vertex_data(vertex_address)
                .vertex_stride(Vertex::stride() as vk::DeviceSize)
                .max_vertex(model.max_index as u32)
                .index_type(RenderModel::index_type())
                .index_data(index_address)
                .transform_data(vk::DeviceOrHostAddressConstKHR {
                    host_address: 0 as *const c_void,
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
                    vk::AccelerationStructureBuildTypeKHR::HOST_OR_DEVICE,
                    &build_type[0],
                    &[model.num_triangles()],
                );

            let info = vk::BufferCreateInfo::builder()
                .size(build_size.acceleration_structure_size)
                .queue_family_indices(&queue_family_indicies)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR);
            let acceleration_structure_buffer =
                base.device.create_buffer(&info, None).expect("buffer");
            let memory_reqs = base
                .device
                .get_buffer_memory_requirements(acceleration_structure_buffer);
            let allocation = allocator
                .lock()
                .expect("failed to get lock")
                .allocate(&AllocationCreateDesc {
                    name: "",
                    requirements: memory_reqs,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                    allocation_scheme: AllocationScheme::DedicatedBuffer(
                        acceleration_structure_buffer,
                    ),
                })
                .expect("failed to get allocation");

            base.device
                .bind_buffer_memory(
                    acceleration_structure_buffer,
                    allocation.memory(),
                    allocation.offset(),
                )
                .expect("failed to bind memory");
            let acceleration_structure_create_info =
                vk::AccelerationStructureCreateInfoKHR::builder()
                    .buffer(acceleration_structure_buffer)
                    .offset(0)
                    .size(build_size.acceleration_structure_size)
                    .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL);
            let acceleration_structure = raytracing_state
                .acceleration_structure
                .create_acceleration_structure(&acceleration_structure_create_info, None)
                .expect("failed to create acceleration structure");

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
                    allocation_scheme: AllocationScheme::DedicatedBuffer(scratch_buffer),
                })
                .expect("failed to bind scratch memory");
            base.device
                .bind_buffer_memory(
                    scratch_buffer,
                    scratch_memory.memory(),
                    scratch_memory.offset(),
                )
                .expect("failed to bind memory");

            base.device.device_wait_idle().expect("failed to wait??");

            record_submit_commandbuffer(
                &base.device,
                base.setup_command_buffer,
                base.setup_commands_reuse_fence,
                base.present_queue,
                &[],
                &[],
                &[],
                |_device, command_buffer| {
                    let geo = [vk::AccelerationStructureGeometryKHR::builder()
                        .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
                        .geometry(vk::AccelerationStructureGeometryDataKHR { triangles })
                        .build()];
                    let build_range_infos = [vk::AccelerationStructureBuildRangeInfoKHR::builder()
                        .primitive_count(model.num_triangles())
                        .primitive_offset(0)
                        .first_vertex(0)
                        .transform_offset(0)
                        .build()];
                    let range_arr: [&[vk::AccelerationStructureBuildRangeInfoKHR]; 1] =
                        [&build_range_infos];
                    let build_type = [vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                        .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                        .geometries(&geo)
                        .scratch_data(get_device_or_host_address(&base.device, &scratch_buffer))
                        .dst_acceleration_structure(acceleration_structure)
                        .build()];

                    raytracing_state
                        .acceleration_structure
                        .cmd_build_acceleration_structures(command_buffer, &build_type, &range_arr);
                },
            );

            base.device
                .wait_for_fences(&[base.setup_commands_reuse_fence], true, u64::MAX)
                .map_err(|e| base.aftermath_state.handle_error(e))
                .expect("failed to wait for fence");

            base.device
                .device_wait_idle()
                .map_err(|e| base.aftermath_state.handle_error(e))
                .expect("failed to wait idle");

            allocator
                .lock()
                .expect("failed to get allocator")
                .free(scratch_memory)
                .expect("failed to free");
            base.device.destroy_buffer(scratch_buffer, None);
            Ok(Self {
                buffer: acceleration_structure_buffer,
                allocation: Some(allocation),
                acceleration_structure,
                transform_matrix: model.animation.build_transform_mat(0),
                number_triangles: model.num_triangles(),
            })
        }
    }
    fn number_triangles(&self) -> u32 {
        self.number_triangles
    }
    pub fn free(&mut self, base: &PassBase) {
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
pub struct TopLevelAccelerationStructure {
    allocation: Option<Allocation>,
    buffer: Option<vk::Buffer>,
    acceleration_structure: Option<vk::AccelerationStructureKHR>,
}
impl TopLevelAccelerationStructure {
    const ALLOC_SIZE: usize = 20 * 256;
    pub fn new<'a>(
        acceleration_structures: impl Iterator<Item = &'a ModelAccelerationStructure>,
        base: &Base,
        allocator: Arc<Mutex<Allocator>>,
        raytracing_state: &RayTracingState,
        descriptor_pool: &RayTracingDescriptorSets,
    ) -> Self {
        let queue_family_indicies = [base.queue_family_index];

        unsafe {
            let (instance_array, number_primitives) = acceleration_structures
                .map(|acceleration_structure| {
                    let info = vk::AccelerationStructureDeviceAddressInfoKHR::builder()
                        .acceleration_structure(acceleration_structure.acceleration_structure)
                        .build();
                    let acceleration_structure_reference = raytracing_state
                        .acceleration_structure
                        .get_acceleration_structure_device_address(&info);
                    (
                        vk::AccelerationStructureInstanceKHR {
                            transform: vk::TransformMatrixKHR {
                                matrix: [
                                    acceleration_structure.transform_matrix.x[0],
                                    acceleration_structure.transform_matrix.y[0],
                                    acceleration_structure.transform_matrix.z[0],
                                    acceleration_structure.transform_matrix.w[0],
                                    acceleration_structure.transform_matrix.x[1],
                                    acceleration_structure.transform_matrix.y[1],
                                    acceleration_structure.transform_matrix.z[1],
                                    acceleration_structure.transform_matrix.w[1],
                                    acceleration_structure.transform_matrix.x[2],
                                    acceleration_structure.transform_matrix.y[2],
                                    acceleration_structure.transform_matrix.z[2],
                                    acceleration_structure.transform_matrix.w[2],
                                ],
                            },
                            instance_custom_index_and_mask: vk::Packed24_8::new(0, u8::MAX),
                            instance_shader_binding_table_record_offset_and_flags:
                                vk::Packed24_8::new(
                                    0,
                                    vk::GeometryInstanceFlagsKHR::TRIANGLE_CULL_DISABLE_NV.as_raw()
                                        as u8,
                                ),
                            acceleration_structure_reference:
                                vk::AccelerationStructureReferenceKHR {
                                    device_handle: acceleration_structure_reference,
                                },
                        },
                        acceleration_structure.number_triangles(),
                    )
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();
            let instances = vk::AccelerationStructureGeometryInstancesDataKHR::builder()
                .array_of_pointers(false)
                .data(vk::DeviceOrHostAddressConstKHR {
                    host_address: instance_array.as_ptr() as *const c_void,
                })
                .build();

            let geometries = vk::AccelerationStructureGeometryKHR::builder()
                .geometry(vk::AccelerationStructureGeometryDataKHR { instances })
                .geometry_type(vk::GeometryTypeKHR::INSTANCES)
                .flags(vk::GeometryFlagsKHR::empty())
                .build();
            let build_type = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL_NV)
                .flags(vk::BuildAccelerationStructureFlagsKHR::empty())
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .geometries(&[geometries])
                .build();
            let number_primitives = [instance_array.len() as u32];
            let build_size = raytracing_state
                .acceleration_structure
                .get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &build_type,
                    &number_primitives,
                );
            let info = vk::BufferCreateInfo::builder()
                .size(build_size.acceleration_structure_size)
                .queue_family_indices(&queue_family_indicies)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR);
            let buffer = base
                .device
                .create_buffer(&info, None)
                .expect("failed to create buffer for top level acceleration structure");

            let memory_reqs = base.device.get_buffer_memory_requirements(buffer);
            let allocation = allocator
                .lock()
                .expect("failed to get lock")
                .allocate(&AllocationCreateDesc {
                    name: "Top Level Acceleration Structure Buffer",
                    requirements: memory_reqs,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                    allocation_scheme: AllocationScheme::DedicatedBuffer(buffer),
                })
                .expect("failed to get allocation");
            base.device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .expect("failed to allocate memory");
            let scratch_info = vk::BufferCreateInfo::builder()
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
                .create_buffer(&scratch_info, None)
                .expect("failed to create scratch buffer for top level acceleration structure");

            let memory_reqs = base.device.get_buffer_memory_requirements(scratch_buffer);
            let scratch_allocation = allocator
                .lock()
                .expect("failed to get lock")
                .allocate(&AllocationCreateDesc {
                    name: "Top Level Acceleration Structure Buffer",
                    requirements: memory_reqs,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                    allocation_scheme: AllocationScheme::DedicatedBuffer(scratch_buffer),
                })
                .expect("failed to get allocation");
            base.device
                .bind_buffer_memory(
                    scratch_buffer,
                    scratch_allocation.memory(),
                    scratch_allocation.offset(),
                )
                .expect("failed to bind memory");
            let info = vk::AccelerationStructureCreateInfoKHR::builder()
                .buffer(buffer)
                .offset(0)
                .size(build_size.acceleration_structure_size)
                .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL);

            let acceleration_structure = raytracing_state
                .acceleration_structure
                .create_acceleration_structure(&info, None)
                .expect("failed to create structure");
            record_submit_commandbuffer(
                &base.device,
                base.setup_command_buffer,
                base.setup_commands_reuse_fence,
                base.present_queue,
                &[],
                &[],
                &[],
                |device, command_buffer| {
                    let build_type = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                        .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL_NV)
                        .flags(vk::BuildAccelerationStructureFlagsKHR::empty())
                        .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                        .dst_acceleration_structure(acceleration_structure)
                        .geometries(&[geometries])
                        .scratch_data(get_device_or_host_address(&base.device, &scratch_buffer))
                        .build();
                    let build_range_infos = [vk::AccelerationStructureBuildRangeInfoKHR::builder()
                        .primitive_count(0)
                        .primitive_offset(0)
                        .transform_offset(0)
                        .build()];

                    raytracing_state
                        .acceleration_structure
                        .cmd_build_acceleration_structures(
                            command_buffer,
                            &[build_type],
                            &[&build_range_infos],
                        );
                },
            );
            base.device
                .wait_for_fences(&[base.setup_commands_reuse_fence], true, u64::MAX)
                .expect("failed to wait");
            base.device.destroy_buffer(scratch_buffer, None);
            allocator
                .lock()
                .expect("failed to lock allocator")
                .free(scratch_allocation)
                .expect("failed to free scratch allocation");
            Self {
                buffer: Some(buffer),
                allocation: Some(allocation),
                acceleration_structure: Some(acceleration_structure),
            }
        }
    }
    pub fn descriptor_set(&self) -> Option<vk::DescriptorSet> {
        todo!()
    }
    pub fn free(
        &mut self,
        base: &Base,
        allocator: Arc<Mutex<Allocator>>,
        raytracing_state: &RayTracingState,
    ) {
        unsafe {
            raytracing_state
                .acceleration_structure
                .destroy_acceleration_structure(self.acceleration_structure.take().unwrap(), None);
        }
        allocator
            .lock()
            .expect("failed to get allocator")
            .free(self.allocation.take().expect("allocator is already freed"));
        unsafe {
            base.device.destroy_buffer(
                self.buffer
                    .take()
                    .expect("top level acceleration structure is already freed"),
                None,
            )
        }
    }
}
