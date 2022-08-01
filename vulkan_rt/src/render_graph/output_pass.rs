use super::{get_semaphores, PassBase, VulkanOutput, VulkanOutputType, VulkanPass};
use crate::{prelude::*, record_submit_commandbuffer};
use ash::{util::read_spv, vk};

use std::{ffi::CStr, io::Cursor, mem::size_of};

/// Describes renderpass that render a framebuffer to screen
pub struct OutputPass {
    imgui_renderer: imgui_rs_vulkan_renderer::Renderer,
    imgui_platform: imgui_winit_support::WinitPlatform,
    render_plane: Option<RenderModel>,
    framebuffers: Vec<vk::Framebuffer>,
    renderpass: vk::RenderPass,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layouts: [vk::DescriptorSetLayout; 1],
    fragment_shader_module: vk::ShaderModule,
    vertex_shader_module: vk::ShaderModule,
    graphics_pipeline: vk::Pipeline,

    pipeline_layout: vk::PipelineLayout,
    scissors: [vk::Rect2D; 1],
    viewports: [vk::Viewport; 1],
    present_index: Option<u32>,
}
impl OutputPass {
    pub fn new(base: &mut PassBase) -> Self {
        let mut scene_state = base.scene_state.as_ref().borrow_mut();

        let imgui_context = &mut scene_state.imgui_context;
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(imgui_context);

        let hidipi_factor = imgui_platform.hidpi_factor();

        imgui_platform.attach_window(
            imgui_context.io_mut(),
            &base.base.window,
            imgui_winit_support::HiDpiMode::Rounded,
        );
        scene_state.imgui_context.io_mut().font_global_scale = 1.0 / hidipi_factor as f32;

        let renderpass_attachments = [
            vk::AttachmentDescription::builder()
                .format(base.base.surface_format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build(),
            vk::AttachmentDescription::builder()
                .format(vk::Format::D16_UNORM)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build(),
        ];
        let color_attachment_refs = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
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
            .depth_stencil_attachment(&depth_attachment_ref)
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
        let framebuffers = unsafe {
            base.base
                .present_image_views
                .iter()
                .map(|&present_image_view| {
                    let framebuffer_attachments = [present_image_view, base.base.depth_image_view];
                    let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(renderpass)
                        .attachments(&framebuffer_attachments)
                        .width(base.base.surface_resolution.width)
                        .height(base.base.surface_resolution.height)
                        .layers(1);
                    base.base
                        .device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .expect("failed to create framebuffer")
                })
                .collect::<Vec<_>>()
        };
        let descriptor_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
        }];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&descriptor_sizes)
            .max_sets(100);
        let descriptor_pool = unsafe {
            base.base
                .device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("failed to get descriptor pool")
        };
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

        let render_plane = {
            let render_plane_mesh = Mesh::plane();
            let render_plane_model = Model {
                animation: AnimationList::new(vec![]),
                mesh: render_plane_mesh,
                texture: image::RgbaImage::from_pixel(100, 100, image::Rgba([100, 100, 100, 0xff])),
            };
            render_plane_model.build_render_model(
                base.base.as_ref(),
                &mut base.allocator.lock().expect("failed to lock"),
                &descriptor_pool,
                &descriptor_set_layouts,
            )
        };
        let mut vertex_spv_file = Cursor::new(include_bytes!("../../shaders/bin/push.vert.glsl"));
        let mut frag_spv_file = Cursor::new(include_bytes!("../../shaders/bin/push.frag.glsl"));
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
            .size(size_of::<Mat4>() as u32);
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
        let imgui_renderer = imgui_rs_vulkan_renderer::Renderer::with_gpu_allocator(
            base.allocator.clone(),
            base.base.device.clone(),
            base.base.present_queue,
            base.base.pool,
            renderpass,
            &mut scene_state.imgui_context,
            Some(imgui_rs_vulkan_renderer::Options {
                in_flight_frames: framebuffers.len(),
                ..Default::default()
            }),
        )
        .expect("failed to make renderer");
        Self {
            imgui_renderer,
            imgui_platform,
            framebuffers,
            renderpass,
            descriptor_pool,
            descriptor_set_layouts,
            fragment_shader_module,
            vertex_shader_module,
            graphics_pipeline,
            pipeline_layout,
            render_plane: Some(render_plane),
            viewports,
            scissors,
            present_index: None,
        }
    }
}
impl VulkanPass for OutputPass {
    fn handle_event(&mut self, base: &PassBase, event: &winit::event::Event<()>) {
        let mut scene_state = base.scene_state.as_ref().borrow_mut();
        self.imgui_platform.handle_event(
            scene_state.imgui_context.io_mut(),
            &base.base.window,
            event,
        )
    }

    fn get_dependencies(&self) -> Vec<VulkanOutputType> {
        vec![VulkanOutputType::FrameBuffer, VulkanOutputType::Empty]
    }

    fn get_output(&self) -> Vec<VulkanOutputType> {
        Vec::new()
    }

    fn process(&mut self, base: &PassBase, input: Vec<&VulkanOutput>) -> Vec<VulkanOutput> {
        let mut depedency_semaphores = get_semaphores(&input);
        let (present_index, _) = unsafe {
            base.base
                .swapchain_loader
                .acquire_next_image(
                    base.base.swapchain,
                    u64::MAX,
                    base.base.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .expect("failed to acquire image")
        };
        self.present_index = Some(present_index);
        let diffuse_descriptor_set = match input[1] {
            &VulkanOutput::Framebuffer { descriptor_set, .. } => descriptor_set,
            _ => panic!("invalid dependency"),
        };

        let mut scene_state = base.scene_state.as_ref().borrow_mut();
        self.imgui_platform
            .prepare_frame(scene_state.imgui_context.io_mut(), &base.base.window)
            .expect("failed to prepare frame");

        let ui = scene_state.imgui_context.frame();

        let mut engine_entities = base.engine_entities.as_ref().borrow_mut();
        let mut new_active_scene: Option<String> = None;

        for (i, scene_name) in engine_entities.names().iter().enumerate() {
            ui.text(format!("{}", i));

            let button_clicked = ui.button(scene_name.to_string());
            if button_clicked {
                println!("set new active scene, {}", scene_name);
                new_active_scene = Some(scene_name.to_string());
            }
        }
        if let Some(name) = new_active_scene {
            engine_entities.set_name(name);
        }
        if ui.is_any_item_hovered() {
            // println!("hovered!!!");
        }
        /*
        if ui.is_any_item_active() {
            println!("item active")
        }

         */
        if ui.button("foo!!!!") {
            println!("clicked foo????");
        }
        self.imgui_platform.prepare_render(&ui, &base.base.window);
        let draw_data = ui.render();
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
                self.framebuffers[self.present_index.expect("failed to do process stage") as usize],
            )
            .render_area(base.base.surface_resolution.into())
            .clear_values(&clear_values);
        depedency_semaphores.push(base.base.present_complete_semaphore);
        let wait_mask = depedency_semaphores
            .iter()
            .map(|_| vk::PipelineStageFlags::BOTTOM_OF_PIPE)
            .collect::<Vec<_>>();
        unsafe {
            record_submit_commandbuffer(
                &base.base.device,
                base.base.draw_command_buffer,
                base.base.draw_commands_reuse_fence,
                base.base.present_queue,
                &wait_mask,
                &depedency_semaphores,
                &[base.base.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &renderpass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    let transform_mat = self
                        .render_plane
                        .as_ref()
                        .unwrap()
                        .animation
                        .build_transform_mat(0);
                    /*
                    device.cmd_bind_descriptor_sets(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_layout,
                        0,
                        &[self.render_plane.as_ref().unwrap().texture.descriptor_set],
                        &[],
                    );
                    */
                    device.cmd_bind_descriptor_sets(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_layout,
                        0,
                        &[diffuse_descriptor_set],
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
                        &[self.render_plane.as_ref().unwrap().vertex_buffer],
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
                        self.render_plane.as_ref().unwrap().index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.cmd_draw_indexed(
                        draw_command_buffer,
                        self.render_plane.as_ref().unwrap().num_indices,
                        1,
                        0,
                        0,
                        1,
                    );

                    self.imgui_renderer
                        .cmd_draw(draw_command_buffer, draw_data)
                        .expect("failed to draw");
                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );
            let present_index = self.present_index.unwrap();
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(std::slice::from_ref(
                    &base.base.rendering_complete_semaphore,
                ))
                .swapchains(std::slice::from_ref(&base.base.swapchain))
                .image_indices(std::slice::from_ref(&present_index));
            base.base
                .swapchain_loader
                .queue_present(base.base.present_queue, &present_info)
                .expect("failed to present render");
            self.present_index = None;
        }

        Vec::new()
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
            let plane = self.render_plane.take().expect("resource already freed");
            plane.free_resources(
                base.base.as_ref(),
                &mut base.allocator.lock().expect("failed to get lock"),
            );

            for &descriptor_set_layout in self.descriptor_set_layouts.iter() {
                base.base
                    .device
                    .destroy_descriptor_set_layout(descriptor_set_layout, None);
            }
            base.base
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            for framebuffer in self.framebuffers.drain(..) {
                base.base.device.destroy_framebuffer(framebuffer, None);
            }
            base.base.device.destroy_render_pass(self.renderpass, None);
        }
    }
}
