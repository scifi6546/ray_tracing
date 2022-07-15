use super::{AnimationList, Mesh, Vertex};
use crate::{find_memory_type_index, record_submit_commandbuffer, Base};
use ash::{util::Align, vk};
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, Allocator},
    MemoryLocation,
};
use std::mem::{align_of, size_of};
/// Texture used by render meshes
pub struct RenderTexture {
    pub texture_image: vk::Image,
    pub texture_allocation: Allocation,
    pub texture_image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}
impl RenderTexture {
    pub fn new(
        texture: &image::RgbaImage,
        base: &Base,
        allocator: &mut Allocator,
        descriptor_pool: &vk::DescriptorPool,
        descriptor_layouts: &[vk::DescriptorSetLayout],
    ) -> Self {
        let desc_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool.clone())
            .set_layouts(descriptor_layouts);
        let descriptor_set = unsafe {
            base.device
                .allocate_descriptor_sets(&desc_alloc_info)
                .expect("failed to allocate desc layout")
        }[0];
        let (width, height) = texture.dimensions();
        let image_extent = vk::Extent2D { width, height };
        let image_data = texture.clone().into_raw();
        let image_buffer_info = vk::BufferCreateInfo::builder()
            .size(size_of::<u8>() as u64 * image_data.len() as u64)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let image_buffer = unsafe {
            base.device
                .create_buffer(&image_buffer_info, None)
                .expect("failed to get buffer")
        };
        let image_buffer_memory_req =
            unsafe { base.device.get_buffer_memory_requirements(image_buffer) };

        let image_buffer_allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "image buffer allocation",
                requirements: image_buffer_memory_req,
                location: MemoryLocation::CpuToGpu,
                linear: true,
            })
            .expect("failed to make allocation");

        let img_ptr = image_buffer_allocation
            .mapped_ptr()
            .expect("failed to map image pointer");
        unsafe {
            let mut image_slice = Align::new(
                img_ptr.as_ptr(),
                align_of::<u8>() as u64,
                image_buffer_memory_req.size,
            );
            image_slice.copy_from_slice(&image_data);

            base.device
                .bind_buffer_memory(
                    image_buffer,
                    image_buffer_allocation.memory(),
                    image_buffer_allocation.offset(),
                )
                .expect("failed to bind texture memory");
        }
        let texture_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(image_extent.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let texture_image = unsafe {
            base.device
                .create_image(&texture_create_info, None)
                .expect("failed to create image")
        };
        let texture_memory_req =
            unsafe { base.device.get_image_memory_requirements(texture_image) };

        let texture_allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "image buffer allocation",
                requirements: texture_memory_req,
                location: MemoryLocation::GpuOnly,
                linear: true,
            })
            .expect("failed to free allocation");

        unsafe {
            base.device
                .bind_image_memory(
                    texture_image,
                    texture_allocation.memory(),
                    texture_allocation.offset(),
                )
                .expect("failed to bind image memory");
        }

        unsafe {
            record_submit_commandbuffer(
                &base.device,
                base.setup_command_buffer,
                base.setup_commands_reuse_fence,
                base.present_queue,
                &[],
                &[],
                &[],
                |device, texture_command_buffer| {
                    let texture_barrier = vk::ImageMemoryBarrier::builder()
                        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .image(texture_image)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .level_count(1)
                                .layer_count(1)
                                .build(),
                        )
                        .build();
                    device.cmd_pipeline_barrier(
                        texture_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier],
                    );
                    let buffer_copy_region = vk::BufferImageCopy::builder()
                        .image_subresource(
                            vk::ImageSubresourceLayers::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .layer_count(1)
                                .build(),
                        )
                        .image_extent(image_extent.into())
                        .build();
                    device.cmd_copy_buffer_to_image(
                        texture_command_buffer,
                        image_buffer,
                        texture_image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[buffer_copy_region],
                    );
                    let texture_barrier_end = vk::ImageMemoryBarrier::builder()
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image(texture_image)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .level_count(1)
                                .layer_count(1)
                                .build(),
                        )
                        .build();
                    device.cmd_pipeline_barrier(
                        texture_command_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier_end],
                    );
                },
            )
        }
        allocator
            .free(image_buffer_allocation)
            .expect("failed to free");

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_v(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_w(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .compare_op(vk::CompareOp::NEVER);
        let sampler = unsafe {
            base.device
                .create_sampler(&sampler_info, None)
                .expect("failed to get sampler")
        };
        let tex_image_view_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(texture_create_info.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            })
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(texture_image);
        let texture_image_view = unsafe {
            base.device
                .create_image_view(&tex_image_view_info, None)
                .expect("failed to get tex image view")
        };

        let tex_descriptor = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: texture_image_view,
            sampler,
        };
        let write_desc_sets = [vk::WriteDescriptorSet {
            dst_set: descriptor_set,
            dst_binding: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &tex_descriptor,
            ..Default::default()
        }];

        unsafe {
            base.device.update_descriptor_sets(&write_desc_sets, &[]);
            base.device
                .wait_for_fences(&[base.setup_commands_reuse_fence], true, u64::MAX);

            base.device.destroy_buffer(image_buffer, None);
        }
        Self {
            sampler,
            texture_allocation,
            texture_image,
            texture_image_view,
        }
    }
}
pub struct RenderModel {
    pub index_buffer: vk::Buffer,
    pub index_allocation: Allocation,
    pub vertex_buffer: vk::Buffer,
    pub descriptor_set: vk::DescriptorSet,
    pub vertex_allocation: Allocation,
    pub texture_image: vk::Image,
    pub texture_allocation: Allocation,
    pub texture_image_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub num_indices: u32,
    pub animation: AnimationList,
}
impl RenderModel {
    pub unsafe fn free_resources(mut self, base: &Base, allocator: &mut Allocator) {
        base.device.device_wait_idle().expect("failed to wait idle");
        base.device
            .destroy_image_view(self.texture_image_view, None);
        base.device.destroy_sampler(self.sampler, None);
        base.device.destroy_image(self.texture_image, None);
        allocator
            .free(self.texture_allocation)
            .expect("failed to free texture allocation");

        base.device.destroy_buffer(self.vertex_buffer, None);
        allocator
            .free(self.vertex_allocation)
            .expect("failed to free allocation");
        base.device.destroy_buffer(self.index_buffer, None);
        allocator
            .free(self.index_allocation)
            .expect("failed to destroy index allocation");
    }
}
pub struct Model {
    pub animation: AnimationList,
    pub mesh: Mesh,
    pub texture: image::RgbaImage,
}
impl Model {
    pub fn build_render_model(
        &self,
        base: &Base,
        allocator: &mut Allocator,
        descriptor_pool: &vk::DescriptorPool,
        descriptor_layouts: &[vk::DescriptorSetLayout],
    ) -> RenderModel {
        let index_buffer_info = vk::BufferCreateInfo::builder()
            .size(size_of::<u32>() as u64 * self.mesh.indices.len() as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let index_buffer = unsafe {
            base.device
                .create_buffer(&index_buffer_info, None)
                .expect("failed to create index buffer")
        };
        let index_buffer_memory_req =
            unsafe { base.device.get_buffer_memory_requirements(index_buffer) };
        let index_allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "index buffer memory",
                requirements: index_buffer_memory_req,
                location: MemoryLocation::CpuToGpu,
                linear: true,
            })
            .expect("failed to allocate");
        let index_ptr = index_allocation.mapped_ptr().expect("failed to map ptr");

        let mut index_slice = unsafe {
            Align::new(
                index_ptr.as_ptr(),
                align_of::<u32>() as u64,
                index_buffer_memory_req.size,
            )
        };
        unsafe {
            index_slice.copy_from_slice(&self.mesh.indices);

            base.device
                .bind_buffer_memory(
                    index_buffer,
                    index_allocation.memory(),
                    index_allocation.offset(),
                )
                .unwrap();
        }

        let vertex_input_buffer_info = vk::BufferCreateInfo::builder()
            .size(size_of::<Vertex>() as u64 * self.mesh.vertices.len() as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let vertex_buffer = unsafe {
            base.device
                .create_buffer(&vertex_input_buffer_info, None)
                .expect("failed to create vertex input buffer")
        };
        let vertex_input_buffer_memory_req =
            unsafe { base.device.get_buffer_memory_requirements(vertex_buffer) };
        let vertex_allocation = allocator
            .allocate(&AllocationCreateDesc {
                requirements: vertex_input_buffer_memory_req,
                location: MemoryLocation::CpuToGpu,
                linear: true,
                name: "Vertex Buffer",
            })
            .expect("failed to allocate");
        let vert_ptr = vertex_allocation
            .mapped_ptr()
            .expect("failed to map vertex ptr");

        unsafe {
            let mut slice = Align::new(
                vert_ptr.as_ptr(),
                align_of::<Vertex>() as u64,
                vertex_input_buffer_memory_req.size,
            );
            slice.copy_from_slice(&self.mesh.vertices);

            base.device
                .bind_buffer_memory(
                    vertex_buffer,
                    vertex_allocation.memory(),
                    vertex_allocation.offset(),
                )
                .expect("failed to bind vertex buffer memory");
        }
        let desc_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool.clone())
            .set_layouts(descriptor_layouts);
        let descriptor_set = unsafe {
            base.device
                .allocate_descriptor_sets(&desc_alloc_info)
                .expect("failed to allocate desc layout")
        }[0];
        let (width, height) = self.texture.dimensions();
        let image_extent = vk::Extent2D { width, height };
        let image_data = self.texture.clone().into_raw();
        let image_buffer_info = vk::BufferCreateInfo::builder()
            .size(size_of::<u8>() as u64 * image_data.len() as u64)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let image_buffer = unsafe {
            base.device
                .create_buffer(&image_buffer_info, None)
                .expect("failed to get buffer")
        };
        let image_buffer_memory_req =
            unsafe { base.device.get_buffer_memory_requirements(image_buffer) };

        let image_buffer_allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "image buffer allocation",
                requirements: image_buffer_memory_req,
                location: MemoryLocation::CpuToGpu,
                linear: true,
            })
            .expect("failed to make allocation");

        let img_ptr = image_buffer_allocation
            .mapped_ptr()
            .expect("failed to map image pointer");
        unsafe {
            let mut image_slice = Align::new(
                img_ptr.as_ptr(),
                align_of::<u8>() as u64,
                image_buffer_memory_req.size,
            );
            image_slice.copy_from_slice(&image_data);

            base.device
                .bind_buffer_memory(
                    image_buffer,
                    image_buffer_allocation.memory(),
                    image_buffer_allocation.offset(),
                )
                .expect("failed to bind texture memory");
        }
        let texture_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(image_extent.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let texture_image = unsafe {
            base.device
                .create_image(&texture_create_info, None)
                .expect("failed to create image")
        };
        let texture_memory_req =
            unsafe { base.device.get_image_memory_requirements(texture_image) };

        let texture_allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "image buffer allocation",
                requirements: texture_memory_req,
                location: MemoryLocation::GpuOnly,
                linear: true,
            })
            .expect("failed to free allocation");

        unsafe {
            base.device
                .bind_image_memory(
                    texture_image,
                    texture_allocation.memory(),
                    texture_allocation.offset(),
                )
                .expect("failed to bind image memory");
        }

        unsafe {
            record_submit_commandbuffer(
                &base.device,
                base.setup_command_buffer,
                base.setup_commands_reuse_fence,
                base.present_queue,
                &[],
                &[],
                &[],
                |device, texture_command_buffer| {
                    let texture_barrier = vk::ImageMemoryBarrier::builder()
                        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .image(texture_image)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .level_count(1)
                                .layer_count(1)
                                .build(),
                        )
                        .build();
                    device.cmd_pipeline_barrier(
                        texture_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier],
                    );
                    let buffer_copy_region = vk::BufferImageCopy::builder()
                        .image_subresource(
                            vk::ImageSubresourceLayers::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .layer_count(1)
                                .build(),
                        )
                        .image_extent(image_extent.into())
                        .build();
                    device.cmd_copy_buffer_to_image(
                        texture_command_buffer,
                        image_buffer,
                        texture_image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[buffer_copy_region],
                    );
                    let texture_barrier_end = vk::ImageMemoryBarrier::builder()
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image(texture_image)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .level_count(1)
                                .layer_count(1)
                                .build(),
                        )
                        .build();
                    device.cmd_pipeline_barrier(
                        texture_command_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier_end],
                    );
                },
            )
        }
        allocator
            .free(image_buffer_allocation)
            .expect("failed to free");

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_v(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_w(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .compare_op(vk::CompareOp::NEVER);
        let sampler = unsafe {
            base.device
                .create_sampler(&sampler_info, None)
                .expect("failed to get sampler")
        };
        let tex_image_view_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(texture_create_info.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            })
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(texture_image);
        let texture_image_view = unsafe {
            base.device
                .create_image_view(&tex_image_view_info, None)
                .expect("failed to get tex image view")
        };

        let tex_descriptor = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: texture_image_view,
            sampler,
        };
        let write_desc_sets = [vk::WriteDescriptorSet {
            dst_set: descriptor_set,
            dst_binding: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &tex_descriptor,
            ..Default::default()
        }];

        unsafe {
            base.device.update_descriptor_sets(&write_desc_sets, &[]);
            base.device
                .wait_for_fences(&[base.setup_commands_reuse_fence], true, u64::MAX);

            base.device.destroy_buffer(image_buffer, None);
        }
        RenderModel {
            index_buffer,
            index_allocation,
            vertex_buffer,
            vertex_allocation,
            descriptor_set,
            texture_image,
            texture_allocation,
            texture_image_view,
            sampler,
            num_indices: self.mesh.indices.len() as u32,
            animation: self.animation.clone(),
        }
    }
}
