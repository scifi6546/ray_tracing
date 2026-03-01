use super::{
    Descriptors, PresentModel, PresentVertex, SetupCommandBuffer, record_submit_command_buffer,
};
use ash::{Device, Instance, khr, util::read_spv, vk};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::io::Cursor;
struct DepthImageData {
    image: vk::Image,
    allocation: Allocation,
    view: vk::ImageView,
}
impl DepthImageData {
    pub fn free(self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
            allocator
                .free(self.allocation)
                .expect("failed to deallocate");
        }
    }
}
pub struct PresentPass {
    pub graphics_pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub renderpass: vk::RenderPass,
    pub fragment_shader_module: vk::ShaderModule,
    pub vertex_shader_module: vk::ShaderModule,
    pub framebuffers: Vec<vk::Framebuffer>,

    pub present_image_views: Vec<vk::ImageView>,
    depth_images: Vec<DepthImageData>,

    //does not need to be freed
    pub viewport: vk::Viewport,
    #[allow(dead_code)]
    pub present_images: Vec<vk::Image>,
    pub surface_resolution: vk::Extent2D,
}
impl PresentPass {
    const DEPTH_FORMAT: vk::Format = vk::Format::D16_UNORM;
    pub fn new(
        device: &Device,
        setup_command_buffer: &mut SetupCommandBuffer,
        instance: &Instance,
        swapchain: vk::SwapchainKHR,
        present_queue: vk::Queue,
        allocator: &mut Allocator,
        descriptors: &Descriptors,
        surface_resolution: vk::Extent2D,
        surface_format: vk::SurfaceFormatKHR,
    ) -> Self {
        unsafe {
            let swapchain_device = khr::swapchain::Device::new(instance, device);
            let renderpass_attachments = [
                vk::AttachmentDescription::default()
                    .format(surface_format.format)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
                vk::AttachmentDescription::default()
                    .format(Self::DEPTH_FORMAT)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            ];
            let color_attachments = [vk::AttachmentReference::default()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
            let depth_stencil_attachment = vk::AttachmentReference::default()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
            let subpass_dependencies = [vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)];
            let subpass = [vk::SubpassDescription::default()
                .color_attachments(&color_attachments)
                .depth_stencil_attachment(&depth_stencil_attachment)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)];
            let renderpass_create_info = vk::RenderPassCreateInfo::default()
                .attachments(&renderpass_attachments)
                .subpasses(&subpass)
                .dependencies(&subpass_dependencies);
            let renderpass = device
                .create_render_pass(&renderpass_create_info, None)
                .expect("failed to crate renderpass");
            let present_images = swapchain_device
                .get_swapchain_images(swapchain)
                .expect("failed to get present images");
            let mut depth_images = present_images
                .iter()
                .map(|_| {
                    let depth_image_create_info = vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(Self::DEPTH_FORMAT)
                        .extent(surface_resolution.into())
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE);
                    let depth_image = device
                        .create_image(&depth_image_create_info, None)
                        .expect("failed to create depth image");
                    let depth_memory_requirements =
                        device.get_image_memory_requirements(depth_image);

                    let depth_image_allocation = allocator
                        .allocate(&AllocationCreateDesc {
                            name: "depth image allocation",
                            requirements: depth_memory_requirements,
                            location: MemoryLocation::GpuOnly,
                            linear: true,
                            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                        })
                        .expect("failed to allocate depth image");

                    device
                        .bind_image_memory(
                            depth_image,
                            depth_image_allocation.memory(),
                            depth_image_allocation.offset(),
                        )
                        .expect("failed to bind memory");
                    (depth_image, depth_image_allocation)
                })
                .collect::<Vec<_>>();

            setup_command_buffer.record_command_buffer(
                device,
                present_queue,
                &[],
                &[],
                &[],
                |device, setup_command_buffer| {
                    let layout_transition_barriers = depth_images
                        .iter()
                        .map(|(image, _)| {
                            vk::ImageMemoryBarrier::default()
                                .image(*image)
                                .dst_access_mask(
                                    vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                                        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                                )
                                .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                                .old_layout(vk::ImageLayout::UNDEFINED)
                                .subresource_range(
                                    vk::ImageSubresourceRange::default()
                                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                        .layer_count(1)
                                        .level_count(1),
                                )
                        })
                        .collect::<Vec<_>>();

                    device.cmd_pipeline_barrier(
                        setup_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &layout_transition_barriers,
                    );
                },
            );
            let depth_images = depth_images
                .drain(..)
                .map(|(image, allocation)| {
                    let depth_image_view_info = vk::ImageViewCreateInfo::default()
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .layer_count(1)
                                .level_count(1),
                        )
                        .image(image)
                        .format(Self::DEPTH_FORMAT)
                        .view_type(vk::ImageViewType::TYPE_2D);
                    let view = device
                        .create_image_view(&depth_image_view_info, None)
                        .expect("failed to create depth");
                    DepthImageData {
                        image,
                        allocation,
                        view,
                    }
                })
                .collect::<Vec<_>>();

            let present_image_views = present_images
                .iter()
                .map(|image| {
                    let create_view_info = vk::ImageViewCreateInfo::default()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(*image);
                    device
                        .create_image_view(&create_view_info, None)
                        .expect("failed to create image view")
                })
                .collect::<Vec<_>>();
            let framebuffers = present_image_views
                .iter()
                .zip(depth_images.iter())
                .map(|(present_image_view, depth_image)| {
                    let framebuffer_attachments = [*present_image_view, depth_image.view];
                    let framebuffer_create_info = vk::FramebufferCreateInfo::default()
                        .render_pass(renderpass)
                        .attachments(&framebuffer_attachments)
                        .width(surface_resolution.width)
                        .height(surface_resolution.height)
                        .layers(1);
                    device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .expect("failed to create device")
                })
                .collect();

            let mut vertex_spv_file = Cursor::new(include_bytes!("../../shaders/vert.spv"));
            let mut fragment_spv_file = Cursor::new(include_bytes!("../../shaders/frag.spv"));
            let vertex_code = read_spv(&mut vertex_spv_file).expect("failed to read vertex shader");

            let vertex_shader_info = vk::ShaderModuleCreateInfo::default().code(&vertex_code);
            let vertex_shader_module = device
                .create_shader_module(&vertex_shader_info, None)
                .expect("Failed to create vertex shader");
            let fragment_code =
                read_spv(&mut fragment_spv_file).expect("failed to read fragment shader");
            let fragment_shader_info = vk::ShaderModuleCreateInfo::default().code(&fragment_code);
            let fragment_shader_module = device
                .create_shader_module(&fragment_shader_info, None)
                .expect("failed to create fragment shader module");
            let set_layouts = [descriptors.layout];
            let layout_create_info =
                vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
            let pipeline_layout = device
                .create_pipeline_layout(&layout_create_info, None)
                .expect("failed to create layout");
            let shader_entry_name = c"main";
            let shader_stage_create_infos = [
                vk::PipelineShaderStageCreateInfo::default()
                    .module(vertex_shader_module)
                    .name(&shader_entry_name)
                    .stage(vk::ShaderStageFlags::VERTEX),
                vk::PipelineShaderStageCreateInfo::default()
                    .module(fragment_shader_module)
                    .name(shader_entry_name)
                    .stage(vk::ShaderStageFlags::FRAGMENT),
            ];

            let vertex_input_binding_description = PresentVertex::input_binding_description();

            let vertex_input_attribute_descriptions = PresentVertex::attribute_descriptions();
            let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
                .vertex_binding_descriptions(&vertex_input_binding_description);
            let vertex_input_assembly_state_info =
                vk::PipelineInputAssemblyStateCreateInfo::default()
                    .topology(vk::PrimitiveTopology::TRIANGLE_FAN);
            let viewports = [vk::Viewport {
                x: 0.,
                y: 0.,
                width: surface_resolution.width as f32,
                height: surface_resolution.height as f32,
                min_depth: 0.,
                max_depth: 1.,
            }];
            let scissors = [surface_resolution.into()];
            let viewport_state_info = vk::PipelineViewportStateCreateInfo::default()
                .viewports(&viewports)
                .scissors(&scissors);
            let rasterization_info = vk::PipelineRasterizationStateCreateInfo::default()
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .line_width(1.)
                .polygon_mode(vk::PolygonMode::FILL);
            let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let noop_stencil_state = vk::StencilOpState::default()
                .fail_op(vk::StencilOp::KEEP)
                .pass_op(vk::StencilOp::KEEP)
                .depth_fail_op(vk::StencilOp::KEEP)
                .compare_op(vk::CompareOp::ALWAYS);
            let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
                .front(noop_stencil_state)
                .back(noop_stencil_state)
                .max_depth_bounds(1.);
            let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
                blend_enable: 1,
                src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ONE,
                dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::RGBA,
            }];
            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op(vk::LogicOp::CLEAR)
                .attachments(&color_blend_attachment_states);
            let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state_info =
                vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
            let graphics_pipeline_infos = [vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stage_create_infos)
                .vertex_input_state(&vertex_input_state_info)
                .input_assembly_state(&vertex_input_assembly_state_info)
                .viewport_state(&viewport_state_info)
                .rasterization_state(&rasterization_info)
                .multisample_state(&multisample_state_info)
                .depth_stencil_state(&depth_state_info)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state_info)
                .layout(pipeline_layout)
                .render_pass(renderpass)];
            let graphics_pipeline = device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphics_pipeline_infos,
                    None,
                )
                .expect("failed to get graphics pipeline")[0];
            Self {
                graphics_pipeline,
                pipeline_layout,
                renderpass,
                fragment_shader_module,
                vertex_shader_module,
                framebuffers,
                present_image_views,
                present_images,
                depth_images,
                viewport: viewports[0],
                surface_resolution,
            }
        }
    }
    pub fn draw(
        &self,
        device: &Device,
        models: &[PresentModel],
        draw_command_buffer: vk::CommandBuffer,
        present_queue: &vk::Queue,
        draw_commandbuffer_reuse_fence: vk::Fence,
        present_complete_semaphore: vk::Semaphore,
        rendering_complete_semaphore: vk::Semaphore,
        current_frame_index: usize,
    ) {
        unsafe {
            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0., 0.5, 0., 0.],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.,
                        stencil: 0,
                    },
                },
            ];

            let renderpass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.renderpass)
                .framebuffer(self.framebuffers[current_frame_index])
                .render_area(self.surface_resolution.into())
                .clear_values(&clear_values);
            device
                .wait_for_fences(&[draw_commandbuffer_reuse_fence], true, u64::MAX)
                .expect("failed to wait for fence");
            device
                .reset_fences(&[draw_commandbuffer_reuse_fence])
                .expect("failed to reset fence");

            record_submit_command_buffer(
                device,
                draw_command_buffer,
                draw_commandbuffer_reuse_fence,
                *present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[present_complete_semaphore],
                &[rendering_complete_semaphore],
                |device, command_buffer| {
                    device.cmd_begin_render_pass(
                        command_buffer,
                        &renderpass_begin_info,
                        vk::SubpassContents::INLINE,
                    );

                    device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.graphics_pipeline,
                    );

                    device.cmd_set_viewport(command_buffer, 0, &[self.viewport]);
                    device.cmd_set_scissor(command_buffer, 0, &[self.surface_resolution.into()]);
                    for model in models {
                        device.cmd_bind_descriptor_sets(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.pipeline_layout,
                            0,
                            &model.descriptor_sets,
                            &[],
                        );
                        device.cmd_bind_index_buffer(
                            command_buffer,
                            model.index_buffer,
                            0,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[model.vertex_buffer],
                            &[0],
                        );
                        device.cmd_draw_indexed(
                            command_buffer,
                            model.num_indices as u32,
                            1,
                            0,
                            0,
                            1,
                        );
                    }

                    device.cmd_end_render_pass(command_buffer);
                },
            );
        }
    }
    pub fn free(mut self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.device_wait_idle().expect("failed to wait idle");

            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_shader_module(self.fragment_shader_module, None);
            device.destroy_shader_module(self.vertex_shader_module, None);
            device.destroy_render_pass(self.renderpass, None);
            for framebuffer in self.framebuffers.drain(..) {
                device.destroy_framebuffer(framebuffer, None);
            }
            for view in self.present_image_views.drain(..) {
                device.destroy_image_view(view, None);
            }
            for image in self.depth_images.drain(..) {
                image.free(device, allocator);
            }
        }
    }
}
