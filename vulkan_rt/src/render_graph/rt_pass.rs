mod acceleration_structure;
mod descriptor_sets;
mod pipeline;
use super::{
    framebuffer_texture::FramebufferTexture, Base, PassBase, RayTracingState, VulkanOutput,
    VulkanOutputType, VulkanPass,
};

use acceleration_structure::{ModelAccelerationStructure, TopLevelAccelerationStructure};

use pipeline::RayTracingPipeline;

use ash::vk;
use generational_arena::Index as ArenaIndex;

use crate::prelude::{mat4_to_bytes, EngineEntities};
use crate::record_submit_commandbuffer;
use std::collections::HashMap;

pub struct RtPass {
    top_level_acceleration_structure: TopLevelAccelerationStructure,
    model_acceleration_structures: HashMap<ArenaIndex, ModelAccelerationStructure>,
    pipeline: RayTracingPipeline,
    framebuffer_textures: Vec<FramebufferTexture>,
    /// index of framebuffer to use
    current_framebuffer_index: usize,
    render_complete_semaphore: Option<vk::Semaphore>,
    renderpass: Option<vk::RenderPass>,
    draw_fence: Option<vk::Fence>,
    draw_command_buffer: Option<vk::CommandBuffer>,
}
impl RtPass {
    pub fn new(pass_base: &PassBase) -> Result<Self, vk::Result> {
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let renderpass_attachments = [vk::AttachmentDescription::builder()
            .format(FramebufferTexture::COLOR_FORMAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .final_layout(FramebufferTexture::FRAMEBUFFER_LAYOUT)
            .build()];
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
            pass_base
                .base
                .device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };
        let framebuffer_textures = pass_base
            .base
            .present_image_views
            .iter()
            .map(|_| FramebufferTexture::new(pass_base.clone(), renderpass.clone()))
            .collect::<Vec<_>>();

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
                    )
                    .expect("failed to build acceleration structure"),
                )
            })
            .collect::<HashMap<_, _>>();

        let top_level_acceleration_structure = TopLevelAccelerationStructure::new(
            model_acceleration_structures.iter().map(|(a, b)| b),
            &pass_base.base,
            pass_base.allocator.clone(),
            pass_base.raytracing_state.as_ref(),
        );

        let pipeline =
            RayTracingPipeline::new(pass_base.base.as_ref(), pass_base.raytracing_state.as_ref());
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(pass_base.base.pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let draw_command_buffer = unsafe {
            pass_base
                .base
                .device
                .allocate_command_buffers(&command_buffer_alloc_info)
                .unwrap()
        }[0];
        let rendering_complete_semaphore = unsafe {
            let create_info = vk::SemaphoreCreateInfo::builder();
            pass_base
                .base
                .device
                .create_semaphore(&create_info, None)
                .expect("failed to create rendering complete semaphore")
        };
        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let draw_fence = unsafe { pass_base.base.device.create_fence(&fence_create_info, None) }
            .expect("failed to create draw fence");
        Ok(Self {
            model_acceleration_structures,
            top_level_acceleration_structure,
            pipeline,
            framebuffer_textures,
            current_framebuffer_index: 0,
            render_complete_semaphore: Some(rendering_complete_semaphore),
            renderpass: Some(renderpass),
            draw_fence: Some(draw_fence),
            draw_command_buffer: Some(draw_command_buffer),
        })
    }
}
impl VulkanPass for RtPass {
    fn get_dependencies(&self) -> Vec<VulkanOutputType> {
        vec![]
    }

    fn get_output(&self) -> Vec<VulkanOutputType> {
        vec![VulkanOutputType::FrameBuffer]
    }

    fn process(&mut self, base: &PassBase, _input: Vec<&VulkanOutput>) -> Vec<VulkanOutput> {
        unsafe {
            base.base
                .device
                .device_wait_idle()
                .expect("failed to wait idle");
        }
        unsafe {
            record_submit_commandbuffer(
                &base.base.device,
                self.draw_command_buffer.unwrap(),
                self.draw_fence.unwrap(),
                base.base.present_queue,
                &[],
                &[],
                &[self.render_complete_semaphore.unwrap()],
                |device, draw_command_buffer| {},
            );
        }
        let engine_entities: std::cell::Ref<EngineEntities> = base.engine_entities.borrow();
        self.current_framebuffer_index =
            (self.current_framebuffer_index + 1) % base.base.num_swapchain_images();
        vec![VulkanOutput::Framebuffer {
            descriptor_set: self.framebuffer_textures[self.current_framebuffer_index]
                .descriptor_set,
            write_semaphore: Some(self.render_complete_semaphore.unwrap()),
        }]
    }

    fn free(&mut self, base: &PassBase) {
        unsafe {
            self.top_level_acceleration_structure.free(
                base.base.as_ref(),
                base.allocator.clone(),
                base.raytracing_state.as_ref(),
            );

            for (_idx, accel) in self.model_acceleration_structures.iter_mut() {
                accel.free(base);
            }
        }
        self.pipeline.free(&base.base);
        for mut texture in self.framebuffer_textures.drain(..) {
            unsafe {
                texture.free_resources(base);
            }
        }
        unsafe {
            base.base.device.destroy_semaphore(
                self.render_complete_semaphore
                    .take()
                    .expect("rt_pass already freed once"),
                None,
            );
            base.base
                .device
                .destroy_render_pass(self.renderpass.take().expect("rt_pass already freed"), None);
            base.base
                .device
                .destroy_fence(self.draw_fence.take().expect("rt_pass already freed"), None);
        }
    }
}
