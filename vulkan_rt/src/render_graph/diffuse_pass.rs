use super::{PassBase, VulkanOutput, VulkanOutputType, VulkanPass};
use ash::vk;
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc},
    MemoryLocation,
};
use std::borrow::BorrowMut;
struct FramebufferTexture {
    pub texture_image: vk::Image,
    pub texture_allocation: Allocation,
    pub texture_image_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub descriptor_set: vk::DescriptorSet,
    /// todo: make several frame buffers
    framebuffer: vk::Framebuffer,
}
impl FramebufferTexture {
    pub fn new(base: PassBase, render_pass: vk::RenderPass) -> Self {
        let width = base.base.window_width;
        let height = base.base.window_height;
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(COLOR_FORMAT)
            .extent(vk::Extent3D {
                depth: 1,
                height,
                width,
            })
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
    pub unsafe fn free_resources(mut self, base: &PassBase) {
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
pub struct DiffusePass {
    render_textures: Option<Vec<FramebufferTexture>>,
    renderpass: vk::RenderPass,
}
const COLOR_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
impl DiffusePass {
    pub fn new(base: PassBase) -> Self {
        let renderpass_attachments = [vk::AttachmentDescription::builder()
            .format(COLOR_FORMAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let dependencies = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_READ,
            )
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .build()];
        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);
        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);
        let renderpass = unsafe {
            base.base
                .device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };

        let render_textures = base
            .base
            .present_image_views
            .iter()
            .map(|_| FramebufferTexture::new(base.clone(), renderpass.clone()))
            .collect::<Vec<_>>();
        let render_textures = Some(render_textures);
        Self {
            render_textures,
            renderpass,
        }
    }
}
impl VulkanPass for DiffusePass {
    fn get_dependencies(&self) -> Vec<VulkanOutputType> {
        Vec::new()
    }

    fn get_output(&self) -> Vec<VulkanOutputType> {
        vec![VulkanOutputType::Empty]
    }

    fn process(&mut self, base: &PassBase, input: Vec<&VulkanOutput>) -> Vec<VulkanOutput> {
        vec![VulkanOutput::Empty]
    }

    fn free(&mut self, base: &PassBase) {
        let mut render_textures = self
            .render_textures
            .take()
            .expect("diffuse pass has already been freed");
        unsafe {
            for tex in render_textures.drain(..) {
                tex.free_resources(base)
            }
            base.base.device.destroy_render_pass(self.renderpass, None);
        }
    }
}
