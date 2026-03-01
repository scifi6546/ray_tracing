use super::{Descriptors, SetupCommandBuffer};
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
    pub descriptors: &'a Descriptors,
}
pub struct PresentModel {
    texture_image_view: vk::ImageView,
    sampler: vk::Sampler,
    texture_allocation: Allocation,
    texture_image: vk::Image,
    image_buffer: vk::Buffer,
    image_allocation: Allocation,
    pub vertex_allocation: Allocation,
    pub vertex_buffer: vk::Buffer,
    pub index_allocation: Allocation,
    pub index_buffer: vk::Buffer,
    // does not need to get freed
    pub descriptor_sets: Vec<vk::DescriptorSet>,
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
    pub fn new_rectangle(
        width: f32,
        height: f32,
        z_index: f32,
        texture_buffer: &[u8],
        vulkan_info: &mut PresentModelInfo,
    ) -> Self {
        let vertices = [
            PresentVertex {
                pos: [-width, -height, z_index, 1.],
                color: [1., 1., 1., 1.],
                uv: [0., 0.],
            },
            PresentVertex {
                pos: [-width, height, z_index, 1.],
                color: [1., 1., 1., 1.],
                uv: [0., 1.],
            },
            PresentVertex {
                pos: [width, height, z_index, 1.],
                color: [1., 1., 1., 1.],
                uv: [1., 1.],
            },
            PresentVertex {
                pos: [width, -height, z_index, 1.],
                color: [1., 1., 0., 1.],
                uv: [1., 0.],
            },
        ];

        let indices = [0, 3, 1, 3, 2, 1];

        Self::new(&vertices, &indices, texture_buffer, vulkan_info)
    }
    pub fn new(
        vertices: &[PresentVertex],
        indices: &[u32],
        texture_buffer: &[u8],
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
            let image = image::load_from_memory(texture_buffer)
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
            let image_buffer = vulkan_info
                .device
                .create_buffer(&image_buffer_info, None)
                .unwrap();
            let image_buffer_memory_requirements = vulkan_info
                .device
                .get_buffer_memory_requirements(image_buffer);
            let image_allocation = vulkan_info
                .allocator
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
            vulkan_info
                .device
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
            let texture_image = vulkan_info
                .device
                .create_image(&texture_create_info, None)
                .unwrap();
            let texture_memory_req = vulkan_info
                .device
                .get_image_memory_requirements(texture_image);
            let texture_allocation = vulkan_info
                .allocator
                .allocate(&AllocationCreateDesc {
                    name: "texture allocation",
                    requirements: texture_memory_req,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .expect("failed to allocate");
            vulkan_info
                .device
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
            vulkan_info.setup_command_buffer.record_command_buffer(
                vulkan_info.device,
                *vulkan_info.present_queue,
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
            let sampler_info = vk::SamplerCreateInfo {
                mag_filter: vk::Filter::LINEAR,
                min_filter: vk::Filter::LINEAR,
                mipmap_mode: vk::SamplerMipmapMode::LINEAR,
                address_mode_u: vk::SamplerAddressMode::MIRRORED_REPEAT,
                address_mode_v: vk::SamplerAddressMode::MIRRORED_REPEAT,
                address_mode_w: vk::SamplerAddressMode::MIRRORED_REPEAT,
                max_anisotropy: 1.0,
                border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
                compare_op: vk::CompareOp::NEVER,
                ..Default::default()
            };
            let sampler = vulkan_info
                .device
                .create_sampler(&sampler_info, None)
                .unwrap();
            let tex_image_view_info = vk::ImageViewCreateInfo {
                view_type: vk::ImageViewType::TYPE_2D,
                format: texture_create_info.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                },
                image: texture_image,
                ..Default::default()
            };
            let texture_image_view = vulkan_info
                .device
                .create_image_view(&tex_image_view_info, None)
                .unwrap();
            let layout = [vulkan_info.descriptors.layout];
            let descriptor_allocate_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(vulkan_info.descriptors.pool)
                .set_layouts(&layout);
            let descriptor_sets = vulkan_info
                .device
                .allocate_descriptor_sets(&descriptor_allocate_info)
                .expect("failed to allocate descriptor set");
            let texture_descriptor = [vk::DescriptorImageInfo {
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image_view: texture_image_view,
                sampler,
            }];
            let write_descriptor_sets = [vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[0])
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&texture_descriptor)];
            vulkan_info
                .device
                .update_descriptor_sets(&write_descriptor_sets, &[]);
            Self {
                descriptor_sets,
                texture_image_view,
                sampler,
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

            device.destroy_image_view(self.texture_image_view, None);
            device.destroy_sampler(self.sampler, None);
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
