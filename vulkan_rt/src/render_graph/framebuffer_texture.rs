use crate::record_submit_commandbuffer;
use crate::render_graph::PassBase;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use gpu_allocator::MemoryLocation;

pub struct FramebufferTexture {
    pub texture_image: vk::Image,
    pub texture_allocation: Allocation,
    pub texture_image_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub descriptor_set: vk::DescriptorSet,
    pub framebuffer: vk::Framebuffer,
}
impl FramebufferTexture {
    pub const COLOR_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
    pub const FRAMEBUFFER_LAYOUT: vk::ImageLayout = vk::ImageLayout::GENERAL;
    pub fn new(base: PassBase, render_pass: vk::RenderPass) -> Self {
        let width = base.base.window_width;
        let height = base.base.window_height;
        let extent = vk::Extent3D {
            depth: 1,
            height,
            width,
        };
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(Self::COLOR_FORMAT)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let texture_image = unsafe {
            base.base
                .device
                .create_image(&image_create_info, None)
                .expect("failed to create image")
        };
        let texture_memory_req = unsafe {
            base.base
                .device
                .get_image_memory_requirements(texture_image)
        };

        let texture_allocation = base
            .allocator
            .lock()
            .expect("failed to get allocation")
            .allocate(&AllocationCreateDesc {
                name: "image buffer allocation",
                requirements: texture_memory_req,
                location: MemoryLocation::GpuOnly,
                linear: true,
                allocation_scheme: AllocationScheme::DedicatedImage(texture_image),
            })
            .expect("failed to free allocation");
        unsafe {
            base.base
                .device
                .bind_image_memory(
                    texture_image,
                    texture_allocation.memory(),
                    texture_allocation.offset(),
                )
                .expect("failed to bind image memory");
        }
        let tex_image_view_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image_create_info.format)
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
            base.base
                .device
                .create_image_view(&tex_image_view_info, None)
                .expect("failed to get tex image view")
        };
        unsafe {
            record_submit_commandbuffer(
                &base.base.device,
                base.base.setup_command_buffer,
                base.base.setup_commands_reuse_fence,
                base.base.present_queue,
                &[],
                &[],
                &[],
                |device, texture_command_buffer| {
                    let texture_barrier = vk::ImageMemoryBarrier::builder()
                        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .new_layout(Self::FRAMEBUFFER_LAYOUT)
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
                },
            );
            base.base
                .device
                .wait_for_fences(&[base.base.setup_commands_reuse_fence], true, u64::MAX)
                .expect("failed to wait for fence");
        }

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
            base.base
                .device
                .create_sampler(&sampler_info, None)
                .expect("failed to get sampler")
        };
        let scene_state = base.scene_state.as_ref().borrow_mut();
        let desc_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(scene_state.mesh_descriptors.descriptor_pool.clone())
            .set_layouts(&scene_state.mesh_descriptors.descriptor_set_layouts);
        let descriptor_set = unsafe {
            base.base
                .device
                .allocate_descriptor_sets(&desc_alloc_info)
                .expect("failed to allocate desc layout")
        }[0];
        let tex_descriptor = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::GENERAL,
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
            base.base
                .device
                .update_descriptor_sets(&write_desc_sets, &[]);
        }
        let framebuffer = unsafe {
            let attachment = [texture_image_view.clone()];
            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachment)
                .width(width)
                .height(height)
                .layers(1);
            base.base
                .device
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("failed to make framebuffer")
        };
        Self {
            texture_image,
            texture_allocation,
            texture_image_view,
            sampler,
            descriptor_set,
            framebuffer,
        }
    }
    pub unsafe fn free_resources(self, base: &PassBase) {
        base.base
            .device
            .device_wait_idle()
            .expect("failed to wait idle");
        base.base
            .device
            .destroy_image_view(self.texture_image_view, None);
        base.base.device.destroy_sampler(self.sampler, None);
        base.base.device.destroy_image(self.texture_image, None);
        base.allocator
            .lock()
            .expect("failed to get allocator")
            .free(self.texture_allocation)
            .expect("failed to free texture allocation");
        base.base.device.destroy_framebuffer(self.framebuffer, None);
    }
}
