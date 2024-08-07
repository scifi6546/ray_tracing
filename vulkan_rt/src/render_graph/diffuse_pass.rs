use super::framebuffer_texture::FramebufferTexture;
use super::{PassBase, VulkanOutput, VulkanOutputType, VulkanPass};
use crate::{prelude::*, record_submit_commandbuffer};
use ash::{util::read_spv, vk};
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme},
    MemoryLocation,
};

use std::{ffi::CStr, io::Cursor, mem::size_of};

pub struct DiffusePass {
    render_textures: Option<Vec<FramebufferTexture>>,
    renderpass: vk::RenderPass,
    descriptor_set_layouts: [vk::DescriptorSetLayout; 1],
    fragment_shader_module: vk::ShaderModule,
    vertex_shader_module: vk::ShaderModule,
    graphics_pipeline: vk::Pipeline,
    rendering_complete_semaphore: vk::Semaphore,
    pipeline_layout: vk::PipelineLayout,
    scissors: [vk::Rect2D; 1],
    viewports: [vk::Viewport; 1],
    framebuffer_idx: u32,
    //semaphore_buffer: SemaphoreBuffer,
    draw_command_buffer: vk::CommandBuffer,
    draw_fence: vk::Fence,
    frame_number: usize,
}
const COLOR_FORMAT: vk::Format = FramebufferTexture::COLOR_FORMAT;
impl DiffusePass {
    pub fn new(base: PassBase) -> Self {
        let desc_layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build()];
        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&desc_layout_bindings);
        let descriptor_set_layouts = unsafe {
            [base
                .base
                .device
                .create_descriptor_set_layout(&descriptor_info, None)
                .expect("failed to create descriptor set layout")]
        };
        let renderpass_attachments = [vk::AttachmentDescription::builder()
            .format(COLOR_FORMAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .final_layout(FramebufferTexture::FRAMEBUFFER_LAYOUT)
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
        let mut vertex_spv_file =
            Cursor::new(load_bytes("./vulkan_rt/shaders/bin/push/push.vert.spv"));
        let mut frag_spv_file =
            Cursor::new(load_bytes("./vulkan_rt/shaders/bin/push/push.frag.spv"));
        let vertex_code =
            read_spv(&mut vertex_spv_file).expect("failed tp read vertex shader code");
        let vert_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);
        let frag_code = read_spv(&mut frag_spv_file).expect("failed to read fragment spv file");
        let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);
        let vertex_shader_module = unsafe {
            base.base
                .device
                .create_shader_module(&vert_shader_info, None)
                .expect("vertex shader compile info")
        };
        let fragment_shader_module = unsafe {
            base.base
                .device
                .create_shader_module(&frag_shader_info, None)
                .expect("failed tp compile fragment shader")
        };
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(size_of::<cgmath::Matrix4<f32>>() as u32);
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(std::slice::from_ref(&push_constant_range))
            .set_layouts(&descriptor_set_layouts);
        let pipeline_layout = unsafe {
            base.base
                .device
                .create_pipeline_layout(&layout_create_info, None)
                .expect("failed to get pipeline layout")
        };
        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(vertex_shader_module)
                .name(shader_entry_name)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(fragment_shader_module)
                .name(shader_entry_name)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 4 * size_of::<f32>() as u32,
            },
        ];
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: base.base.surface_resolution.width as f32,
            height: base.base.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [base.base.surface_resolution.into()];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL);
        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let noop_stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .build();
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .front(noop_stencil_state)
            .back(noop_stencil_state)
            .max_depth_bounds(1.0);
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: false.into(),
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);
        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);
        let graphics_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(renderpass)
            .build();
        let graphics_pipeline = unsafe {
            base.base
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphics_pipeline_info],
                    None,
                )
                .expect("failed to create graphics_pipeline")[0]
        };
        let rendering_complete_semaphore = unsafe {
            let create_info = vk::SemaphoreCreateInfo::builder();
            base.base
                .device
                .create_semaphore(&create_info, None)
                .expect("failed to create rendering complete semaphore")
        };
        //     let semaphore_buffer = SemaphoreBuffer::new(base.clone());
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(base.base.pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let draw_command_buffer = unsafe {
            base.base
                .device
                .allocate_command_buffers(&command_buffer_alloc_info)
                .unwrap()
        }[0];

        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let draw_fence = unsafe { base.base.device.create_fence(&fence_create_info, None) }
            .expect("failed to create draw fence");

        Self {
            render_textures,
            renderpass,
            descriptor_set_layouts,
            fragment_shader_module,
            vertex_shader_module,
            graphics_pipeline,
            pipeline_layout,
            scissors,
            viewports,
            rendering_complete_semaphore,

            framebuffer_idx: 0,
            draw_command_buffer,
            draw_fence,
            frame_number: 0,
        }
    }
}
impl VulkanPass for DiffusePass {
    fn get_dependencies(&self) -> Vec<VulkanOutputType> {
        Vec::new()
    }

    fn get_output(&self) -> Vec<VulkanOutputType> {
        vec![VulkanOutputType::FrameBuffer]
    }

    fn process(&mut self, base: &PassBase, _input: Vec<&VulkanOutput>) -> Vec<VulkanOutput> {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.3, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass)
            .framebuffer(
                self.render_textures.as_ref().expect("already freed")
                    [self.framebuffer_idx as usize]
                    .framebuffer
                    .clone(),
            )
            .render_area(base.base.surface_resolution.into())
            .clear_values(&clear_values);

        unsafe {
            base.base
                .device
                .device_wait_idle()
                .expect("failed to wait idle");
        }
        let engine_entities: std::cell::Ref<EngineEntities> = base.engine_entities.borrow();
        unsafe {
            record_submit_commandbuffer(
                &base.base.device,
                self.draw_command_buffer,
                self.draw_fence,
                base.base.present_queue,
                &[],
                &[],
                &[self.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &renderpass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    let (camera, mesh_list) = engine_entities.get_selected_meshes();
                    let camera_mat = camera.make_transform_mat();
                    for mesh in mesh_list.iter() {
                        let transform_mat =
                            camera_mat * mesh.animation.build_transform_mat(self.frame_number);
                        device.cmd_bind_descriptor_sets(
                            draw_command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.pipeline_layout,
                            0,
                            &[mesh.texture.descriptor_set],
                            &[],
                        );
                        device.cmd_bind_pipeline(
                            draw_command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.graphics_pipeline,
                        );
                        device.cmd_set_viewport(draw_command_buffer, 0, &self.viewports);
                        device.cmd_set_scissor(draw_command_buffer, 0, &self.scissors);
                        device.cmd_bind_vertex_buffers(
                            draw_command_buffer,
                            0,
                            &[mesh.vertex_buffer],
                            &[0],
                        );
                        device.cmd_push_constants(
                            draw_command_buffer,
                            self.pipeline_layout,
                            vk::ShaderStageFlags::VERTEX,
                            0,
                            mat4_to_bytes(&transform_mat),
                        );
                        device.cmd_bind_index_buffer(
                            draw_command_buffer,
                            mesh.index_buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_draw_indexed(draw_command_buffer, mesh.num_indices, 1, 0, 0, 1);
                    }
                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );
        }
        self.framebuffer_idx = (self.framebuffer_idx + 1) % base.base.num_swapchain_images() as u32;
        self.frame_number += 1;
        vec![VulkanOutput::Framebuffer {
            descriptor_set: self.render_textures.as_ref().unwrap()[self.framebuffer_idx as usize]
                .descriptor_set,
            write_semaphore: Some(self.rendering_complete_semaphore),
        }]
    }

    fn free(&mut self, base: &PassBase) {
        unsafe {
            base.base
                .device
                .device_wait_idle()
                .expect("failed to wait idle");
            base.base
                .device
                .destroy_pipeline(self.graphics_pipeline, None);
            base.base
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            base.base
                .device
                .destroy_shader_module(self.vertex_shader_module, None);
            base.base
                .device
                .destroy_shader_module(self.fragment_shader_module, None);
            base.base
                .device
                .destroy_semaphore(self.rendering_complete_semaphore, None);
            base.base.device.destroy_fence(self.draw_fence, None);
            let mut render_textures = self
                .render_textures
                .take()
                .expect("diffuse pass has already been freed");

            for tex in render_textures.drain(..) {
                tex.free_resources(base)
            }
            base.base.device.destroy_render_pass(self.renderpass, None);
            for &descriptor_set_layout in self.descriptor_set_layouts.iter() {
                base.base
                    .device
                    .destroy_descriptor_set_layout(descriptor_set_layout, None);
            }
        }
    }
}
