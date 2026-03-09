use super::PresentModelInfo;
use ash::{Device, util::Align, vk};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};

enum TextureData {
    Local(vk::ImageView, vk::Sampler, Allocation, vk::Image),
    Foreign,
}
pub struct ForeignTextureInput {
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub layout: vk::ImageLayout,
}
impl TextureData {
    pub fn free(self, device: &Device, allocator: &mut Allocator) {
        match self {
            Self::Local(image_view, sampler, allocation, image) => unsafe {
                device.destroy_image_view(image_view, None);
                device.destroy_sampler(sampler, None);
                allocator
                    .free(allocation)
                    .expect("failed to free texture allocation");
                device.destroy_image(image, None);
            },
            Self::Foreign => {}
        }
    }
}

pub struct PresentTexture {
    data: TextureData,
    descriptor_sets: Vec<vk::DescriptorSet>,
}
impl PresentTexture {
    pub fn new(vulkan_info: &mut PresentModelInfo, texture_buffer: &[u8]) -> Self {
        unsafe {
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

            let sampler_info = [vk::DescriptorImageInfo::default().sampler(sampler)];
            let texture_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_image_view)];
            let write_descriptor_sets = [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[0])
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .image_info(&sampler_info)
                    .dst_binding(0),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[0])
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .image_info(&texture_info)
                    .dst_binding(1),
            ];
            vulkan_info
                .device
                .update_descriptor_sets(&write_descriptor_sets, &[]);
            vulkan_info
                .setup_command_buffer
                .wait_for_command_completion(vulkan_info.device);
            vulkan_info.device.destroy_buffer(image_buffer, None);
            vulkan_info
                .allocator
                .free(image_allocation)
                .expect("failed to free image");

            PresentTexture {
                data: TextureData::Local(
                    texture_image_view,
                    sampler,
                    texture_allocation,
                    texture_image,
                ),
                descriptor_sets,
            }
        }
    }
    pub fn from_foreign_data(vulkan_info: &PresentModelInfo, data: ForeignTextureInput) -> Self {
        unsafe {
            let layout = [vulkan_info.descriptors.layout];
            let descriptor_allocate_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(vulkan_info.descriptors.pool)
                .set_layouts(&layout);
            let descriptor_sets = vulkan_info
                .device
                .allocate_descriptor_sets(&descriptor_allocate_info)
                .expect("failed to allocate descriptor set");

            let sampler_image_info = [vk::DescriptorImageInfo::default().sampler(data.sampler)];
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(data.layout)
                .image_view(data.image_view)];
            let write_descriptor_sets = [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[0])
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .image_info(&sampler_image_info)
                    .dst_binding(0),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[0])
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .image_info(&image_info)
                    .dst_binding(1),
            ];
            vulkan_info
                .device
                .update_descriptor_sets(&write_descriptor_sets, &[]);
            PresentTexture {
                data: TextureData::Foreign,
                descriptor_sets,
            }
        }
    }
    pub fn descriptor_sets(&self) -> &[vk::DescriptorSet] {
        &self.descriptor_sets
    }
    pub fn free(self, device: &Device, allocator: &mut Allocator) {
        self.data.free(device, allocator);
    }
}
