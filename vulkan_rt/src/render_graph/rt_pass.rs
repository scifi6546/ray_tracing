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
/// Contains data that is directly in use by the gpu when rendering a frame.
/// Kept separate from main `RtPass` so that multiple frames can be rendered at the same time
struct FrameData {
    fence: vk::Fence,
    draw_command_buffer: vk::CommandBuffer,
    render_complete_semaphore: vk::Semaphore,
    framebuffer_texture: FramebufferTexture,
}
pub struct RtPass {
    top_level_acceleration_structure: TopLevelAccelerationStructure,
    model_acceleration_structures: HashMap<ArenaIndex, ModelAccelerationStructure>,
    pipeline: RayTracingPipeline,
    descriptor_pool: Option<vk::DescriptorPool>,
    /// index of framebuffer to use
    current_framebuffer_index: usize,

    renderpass: Option<vk::RenderPass>,

    draw_command_buffer: Option<vk::CommandBuffer>,
    frame_data: Vec<FrameData>,
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

        let frame_data = pass_base
            .base
            .present_image_views
            .iter()
            .map(|_| FrameData {
                fence: {
                    let fence_create_info =
                        vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
                    unsafe { pass_base.base.device.create_fence(&fence_create_info, None) }
                        .expect("failed to create draw fence")
                },
                draw_command_buffer: {
                    let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
                        .command_buffer_count(1)
                        .command_pool(pass_base.base.pool)
                        .level(vk::CommandBufferLevel::PRIMARY);

                    unsafe {
                        pass_base
                            .base
                            .device
                            .allocate_command_buffers(&command_buffer_alloc_info)
                            .unwrap()[0]
                    }
                },
                render_complete_semaphore: unsafe {
                    let create_info = vk::SemaphoreCreateInfo::builder();
                    pass_base
                        .base
                        .device
                        .create_semaphore(&create_info, None)
                        .expect("failed to create rendering complete semaphore")
                },
                framebuffer_texture: FramebufferTexture::new(pass_base.clone(), renderpass.clone()),
            })
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
        let descriptor_pool = Some({
            let descriptor_sizes = [vk::DescriptorPoolSize {
                ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                descriptor_count: 1,
            }];
            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(&descriptor_sizes)
                .max_sets(100);
            unsafe {
                pass_base
                    .base
                    .device
                    .create_descriptor_pool(&descriptor_pool_info, None)
                    .expect("failed to crate pool")
            }
        });
        let pipeline =
            RayTracingPipeline::new(pass_base.base.as_ref(), pass_base.raytracing_state.as_ref());
        let top_level_acceleration_structure = TopLevelAccelerationStructure::new(
            model_acceleration_structures.iter().map(|(a, b)| b),
            &pass_base.base,
            pass_base.allocator.clone(),
            pass_base.raytracing_state.as_ref(),
            todo!(),
        );

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

        Ok(Self {
            model_acceleration_structures,
            top_level_acceleration_structure,
            pipeline,
            current_framebuffer_index: 0,

            renderpass: Some(renderpass),

            draw_command_buffer: Some(draw_command_buffer),
            descriptor_pool,
            frame_data,
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
            record_submit_commandbuffer(
                &base.base.device,
                self.frame_data[self.current_framebuffer_index].draw_command_buffer,
                self.frame_data[self.current_framebuffer_index].fence,
                base.base.present_queue,
                &[],
                &[],
                &[self.frame_data[self.current_framebuffer_index].render_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_bind_pipeline(
                        draw_command_buffer,
                        vk::PipelineBindPoint::RAY_TRACING_KHR,
                        self.pipeline.pipeline(),
                    );
                    device.cmd_bind_descriptor_sets(
                        draw_command_buffer,
                        self.pipeline.pipeline_bind_point(),
                        self.pipeline.pipeline_layout(),
                        0,
                        &[
                            self.frame_data[self.current_framebuffer_index]
                                .framebuffer_texture
                                .descriptor_set,
                            self.top_level_acceleration_structure
                                .descriptor_set()
                                .expect(
                                    "descriptor set not found for top level acceleration structure",
                                ),
                        ],
                        &[],
                    )
                },
            );
        }

        let output = vec![VulkanOutput::Framebuffer {
            descriptor_set: self.frame_data[self.current_framebuffer_index]
                .framebuffer_texture
                .descriptor_set,
            write_semaphore: Some(
                self.frame_data[self.current_framebuffer_index].render_complete_semaphore,
            ),
        }];
        self.current_framebuffer_index =
            (self.current_framebuffer_index + 1) % base.base.num_swapchain_images();
        output
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

        unsafe {
            base.base
                .device
                .destroy_render_pass(self.renderpass.take().expect("rt_pass already freed"), None);
        }
        for mut frame_data in self.frame_data.drain(..) {
            unsafe {
                base.base
                    .device
                    .destroy_semaphore(frame_data.render_complete_semaphore, None);
                base.base.device.destroy_fence(frame_data.fence, None);
                frame_data.framebuffer_texture.free_resources(base);
            }
        }
        unsafe {
            base.base.device.destroy_descriptor_pool(
                self.descriptor_pool
                    .take()
                    .expect("descriptor pool already freed"),
                None,
            );
        }
    }
}
