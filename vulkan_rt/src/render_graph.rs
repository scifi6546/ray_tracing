mod graph;
use super::{prelude::*, record_submit_commandbuffer, Base, GraphicsApp};
use ash::{util::read_spv, vk};
use gpu_allocator::{vulkan::*, AllocatorDebugSettings};
use graph::{RenderGraph, RenderPass};
use std::{
    ffi::CStr,
    io::Cursor,
    mem::size_of,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};
use winit::event::Event;

enum VulkanOutput {}
#[derive(PartialEq, Eq, Clone)]
enum VulkanOutputType {}
struct VulkanPass {
    frame_number: u32,
    imgui_context: imgui::Context,
    imgui_renderer: imgui_rs_vulkan_renderer::Renderer,
    imgui_platform: imgui_winit_support::WinitPlatform,
    allocator: Arc<Mutex<Allocator>>,
    engine_entities: EngineEntities,
    framebuffers: Vec<vk::Framebuffer>,
    renderpass: vk::RenderPass,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layouts: [vk::DescriptorSetLayout; 1],
    fragment_shader_module: vk::ShaderModule,
    vertex_shader_module: vk::ShaderModule,
    graphics_pipeline: vk::Pipeline,
    mesh_list: Vec<RenderModel>,
    camera: Camera,
    pipeline_layout: vk::PipelineLayout,
    scissors: [vk::Rect2D; 1],

    viewports: [vk::Viewport; 1],
}
impl VulkanPass {
    pub fn new(base: &Base) -> Self {
        let mut imgui_context = imgui::Context::create();
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
        let hidipi_factor = imgui_platform.hidpi_factor();
        imgui_platform.attach_window(
            imgui_context.io_mut(),
            &base.window,
            imgui_winit_support::HiDpiMode::Rounded,
        );
        imgui_context.io_mut().font_global_scale = (1.0 / hidipi_factor as f32);
        let mut allocator = Arc::new(Mutex::new(
            Allocator::new(&AllocatorCreateDesc {
                instance: base.instance.clone(),
                device: base.device.clone(),
                physical_device: base.p_device.clone(),
                debug_settings: AllocatorDebugSettings::default(),
                buffer_device_address: false,
            })
            .expect("created allocator"),
        ));

        let renderpass_attachments = [
            vk::AttachmentDescription::builder()
                .format(base.surface_format.format)
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
            base.device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap()
        };
        let framebuffers = unsafe {
            base.present_image_views
                .iter()
                .map(|&present_image_view| {
                    let framebuffer_attachments = [present_image_view, base.depth_image_view];
                    let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                        .render_pass(renderpass)
                        .attachments(&framebuffer_attachments)
                        .width(base.surface_resolution.width)
                        .height(base.surface_resolution.height)
                        .layers(1);
                    base.device
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
            base.device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("failed to get descriptor pool")
        };
        let desc_layout_bindings = unsafe {
            [vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()]
        };
        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&desc_layout_bindings);
        let descriptor_set_layouts = unsafe {
            [base
                .device
                .create_descriptor_set_layout(&descriptor_info, None)
                .expect("failed to create descriptor set layout")]
        };
        let (meshes, camera) = make_meshes();

        println!("camera: {:#?}", camera);

        let mut mesh_list = meshes
            .iter()
            .map(|m| {
                m.build_render_model(
                    base,
                    &mut allocator.lock().expect("failed to lock"),
                    &descriptor_pool,
                    &descriptor_set_layouts,
                )
            })
            .collect::<Vec<_>>();

        let engine_entities = EngineEntities::new(
            base,
            &mut allocator.lock().expect("failed to lock"),
            &descriptor_pool,
            &descriptor_set_layouts,
        );
        let mut vertex_spv_file = Cursor::new(include_bytes!("../shaders/bin/push.vert.glsl"));
        let mut frag_spv_file = Cursor::new(include_bytes!("../shaders/bin/push.frag.glsl"));
        let vertex_code =
            read_spv(&mut vertex_spv_file).expect("failed tp read vertex shader code");
        let vert_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);
        let frag_code = read_spv(&mut frag_spv_file).expect("failed to read fragment spv file");
        let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);
        let vertex_shader_module = unsafe {
            base.device
                .create_shader_module(&vert_shader_info, None)
                .expect("vertex shader compile info")
        };
        let fragment_shader_module = unsafe {
            base.device
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
            base.device
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
            width: base.surface_resolution.width as f32,
            height: base.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [base.surface_resolution.into()];
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
            base.device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphics_pipeline_info],
                    None,
                )
                .expect("failed to create graphics_pipeline")[0]
        };
        let imgui_renderer = imgui_rs_vulkan_renderer::Renderer::with_gpu_allocator(
            allocator.clone(),
            base.device.clone(),
            base.present_queue,
            base.pool,
            renderpass,
            &mut imgui_context,
            Some(imgui_rs_vulkan_renderer::Options {
                in_flight_frames: framebuffers.len(),
                ..Default::default()
            }),
        )
        .expect("failed to make renderer");
        Self {
            frame_number: 0,
            imgui_context,
            imgui_renderer,
            imgui_platform,
            engine_entities,
            camera,
            allocator,
            framebuffers,
            renderpass,
            descriptor_pool,
            descriptor_set_layouts,
            fragment_shader_module,
            vertex_shader_module,
            graphics_pipeline,
            pipeline_layout,
            mesh_list,
            viewports,
            scissors,
        }
    }
}
struct PassBase {
    base: Rc<Base>,
}
impl RenderPass for VulkanPass {
    type Base = PassBase;
    type RenderPassOutputMarker = VulkanOutputType;
    type RenderPassOutput = VulkanOutput;

    fn get_dependencies(&self) -> Vec<Self::RenderPassOutputMarker> {
        Vec::new()
    }

    fn get_output(&self) -> Vec<Self::RenderPassOutputMarker> {
        Vec::new()
    }

    fn process(
        &mut self,
        base: &Self::Base,
        input: Vec<&Self::RenderPassOutput>,
    ) -> Vec<Self::RenderPassOutput> {
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
        self.imgui_platform
            .prepare_frame(self.imgui_context.io_mut(), &base.base.window)
            .expect("failed to prepare frame");

        let ui = self.imgui_context.frame();
        let mut set_name: Option<String> = None;
        self.imgui_platform.prepare_render(&ui, &base.base.window);
        for scene_name in self.engine_entities.names().iter() {
            let button_res = ui.button(scene_name);
            if button_res {
                set_name = Some(scene_name.to_string());

                println!("pressed button {} for scene: {}", button_res, scene_name)
            }
        }
        if let Some(n) = set_name {
            self.engine_entities.set_name(n);
        }

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
            .framebuffer(self.framebuffers[present_index as usize])
            .render_area(base.base.surface_resolution.into())
            .clear_values(&clear_values);

        unsafe {
            record_submit_commandbuffer(
                &base.base.device,
                base.base.draw_command_buffer,
                base.base.draw_commands_reuse_fence,
                base.base.present_queue,
                &[vk::PipelineStageFlags::BOTTOM_OF_PIPE],
                &[base.base.present_complete_semaphore],
                &[base.base.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &renderpass_begin_info,
                        vk::SubpassContents::INLINE,
                    );

                    let (camera, mesh_list) = self.engine_entities.get_selected_meshes();
                    for mesh in mesh_list.iter() {
                        let transform_mat = camera.make_transform_mat()
                            * mesh
                                .animation
                                .build_transform_mat(self.frame_number as usize);
                        device.cmd_bind_descriptor_sets(
                            draw_command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.pipeline_layout,
                            0,
                            &[mesh.descriptor_set],
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
                            Mat4ToBytes(&transform_mat),
                        );
                        device.cmd_bind_index_buffer(
                            draw_command_buffer,
                            mesh.index_buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_draw_indexed(draw_command_buffer, mesh.num_indices, 1, 0, 0, 1);
                    }
                    self.imgui_renderer
                        .cmd_draw(draw_command_buffer, draw_data)
                        .expect("fai;ed to draw");
                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );

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
        }
        self.frame_number += 1;
        Vec::new()
    }

    fn free(mut self, base: &Self::Base) {
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
            self.engine_entities.free_resources(
                base.base.as_ref(),
                &mut self.allocator.lock().expect("failed to get lock"),
            );
            for mesh in self.mesh_list.drain(..) {
                mesh.free_resources(
                    base.base.as_ref(),
                    &mut self.allocator.lock().expect("failed to get lock"),
                )
            }
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
            drop(self.allocator);
        }
    }
}
pub struct RenderPassApp {
    graph: RenderGraph<VulkanPass>,
}
impl RenderPassApp {
    pub fn new(base: &Base) -> Self {
        let mut graph = RenderGraph::new();
        graph.insert_output_pass(VulkanPass::new(base), Vec::new());
        Self { graph }
    }
}
impl GraphicsApp for RenderPassApp {
    fn run_frame(&mut self, base: Rc<Base>, frame_number: u32) {
        let pass_base = PassBase { base };
        self.graph.run_graph(&pass_base);
    }

    fn process_event(&mut self, elapsed_time: Duration) {}

    fn handle_event(&mut self, base: Rc<Base>, event: &Event<()>) {}

    fn free_resources(self, base: Rc<Base>) {
        let pass_base = PassBase { base };
        self.graph.free_passes(&pass_base);
    }
}
