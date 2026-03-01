use super::SetupCommandBuffer;
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
}
impl PresentVertex {
    pub const fn input_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
    pub const fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
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
        ]
    }
}

pub struct PresentModel {
    texture_allocation: Allocation,
    texture_image: vk::Image,
    image_buffer: vk::Buffer,
    image_allocation: Allocation,
    pub vertex_allocation: Allocation,
    pub vertex_buffer: vk::Buffer,
    pub index_allocation: Allocation,
    pub index_buffer: vk::Buffer,
    pub num_indices: usize,
}
type Index = u32;
impl PresentModel {
    const fn index_type_alignment() -> usize {
        align_of::<Index>()
    }
    const fn vertex_alignment() -> usize {
        align_of::<PresentVertex>()
    }

    pub fn new(
        vertices: &[PresentVertex],
        indices: &[u32],
        device: &Device,
        allocator: &mut Allocator,
        setup_command_buffer: &mut SetupCommandBuffer,
        present_queue: &vk::Queue,
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
            let image = image::load_from_memory(include_bytes!("../../temp_assets/rocket.png"))
                .expect("failed to load")
                .to_rgba8();
            let (width, height) = image.dimensions();
            let image_extent = vk::Extent2D { width, height };
            let image_data = image.into_raw();
            let image_buffer_info = vk::BufferCreateInfo {
                size: (size_of::<u8>() * image_data.len()) as u64,
                usage: vk::BufferUsageFlags::TRANSFER_SRC,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };
            let image_buffer = device.create_buffer(&image_buffer_info, None).unwrap();
            let image_buffer_memory_requirements =
                device.get_buffer_memory_requirements(image_buffer);
            let image_allocation = allocator
                .allocate(&AllocationCreateDesc {
                    name: "image texture",
                    requirements: image_buffer_memory_requirements,
                    location: MemoryLocation::CpuToGpu,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .expect("failed to create allocation");
            {
                let image_ptr = image_allocation
                    .mapped_ptr()
                    .expect("failed to get pointer");
                let raw_ptr = image_ptr.as_ptr();
                let mut image_slice = Align::new(
                    raw_ptr,
                    align_of::<u8>() as u64,
                    image_buffer_memory_requirements.size,
                );
                image_slice.copy_from_slice(&image_data);
            }
            device
                .bind_buffer_memory(
                    image_buffer,
                    image_allocation.memory(),
                    image_allocation.offset(),
                )
                .expect("failed to bind image buffer to memory");
            let texture_create_info = vk::ImageCreateInfo {
                image_type: vk::ImageType::TYPE_2D,
                format: vk::Format::R8G8B8A8_UNORM,
                extent: image_extent.into(),
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };
            let texture_image = device.create_image(&texture_create_info, None).unwrap();
            let texture_memory_req = device.get_image_memory_requirements(texture_image);
            let texture_allocation = allocator
                .allocate(&AllocationCreateDesc {
                    name: "texture allocation",
                    requirements: texture_memory_req,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .expect("failed to allocate");
            device
                .bind_image_memory(
                    texture_image,
                    texture_allocation.memory(),
                    texture_allocation.offset(),
                )
                .expect("failed to bind image to texture");
            let buffer_copy_regions = vk::BufferImageCopy::default()
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1),
                )
                .image_extent(image_extent.into());
            setup_command_buffer.record_command_buffer(
                device,
                *present_queue,
                &[],
                &[],
                &[],
                |device, command_buffer| {
                    let texture_barrier = vk::ImageMemoryBarrier {
                        dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                        new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        image: texture_image,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            level_count: 1,
                            layer_count: 1,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier],
                    );
                    device.cmd_copy_buffer_to_image(
                        command_buffer,
                        image_buffer,
                        texture_image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[buffer_copy_regions],
                    );
                    let texture_barrier_end = vk::ImageMemoryBarrier {
                        src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                        dst_access_mask: vk::AccessFlags::SHADER_READ,
                        old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        image: texture_image,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            level_count: 1,
                            layer_count: 1,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier_end],
                    );
                },
            );

            Self {
                texture_allocation,
                texture_image,
                image_allocation,
                image_buffer,
                vertex_allocation,
                num_indices: indices.len(),
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
                .free(self.texture_allocation)
                .expect("failed to free texture allocation");
            device.destroy_image(self.texture_image, None);
            allocator
                .free(self.image_allocation)
                .expect("failed to free image");
            device.destroy_buffer(self.image_buffer, None);

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
