use super::{
    super::{DrawCommandBuffer, ForeignTextureInput, SetupCommandBuffer},
    model::{RenderModel, RenderModelVertex},
};
use ash::{
    Device,
    util::read_spv,
    vk::{self},
};

use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::io::Cursor;

struct FramebufferImageAttachment {
    image: vk::Image,
    view: vk::ImageView,
    allocation: Allocation,
}
impl FramebufferImageAttachment {
    fn free(self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.destroy_image_view(self.view, None);
            allocator
                .free(self.allocation)
                .expect("failed to free allocation");
            device.destroy_image(self.image, None);
        }
    }
}
struct FramebufferImage {
    framebuffer: vk::Framebuffer,
    depth: FramebufferImageAttachment,
    color: FramebufferImageAttachment,
    color_sampler: vk::Sampler,
}
impl FramebufferImage {
    pub fn new_depth_and_color(
        device: &Device,
        allocator: &mut Allocator,
        setup_buffer: &mut SetupCommandBuffer,
        renderpass: &vk::RenderPass,
        present_queue: vk::Queue,
        number_frames: u32,
        output_resolution: vk::Extent2D,
    ) -> Vec<Self> {
        unsafe {
            let mut color_image = (0..number_frames)
                .map(|_| {
                    let color_image_create_info = vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(VoxelPass::COLOR_OUTPUT_FORMAT)
                        .extent(output_resolution.into())
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE);
                    let image = device
                        .create_image(&color_image_create_info, None)
                        .expect("failed to create image");
                    let color_image_requirements = device.get_image_memory_requirements(image);
                    let allocation = allocator
                        .allocate(&AllocationCreateDesc {
                            name: "voxel pass color buffer",
                            requirements: color_image_requirements,
                            location: MemoryLocation::GpuOnly,
                            linear: true,
                            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                        })
                        .expect("failed to allocate memory for color image");
                    device
                        .bind_image_memory(image, allocation.memory(), allocation.offset())
                        .expect("failed to bind color image");

                    let view_info = vk::ImageViewCreateInfo::default()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(VoxelPass::COLOR_OUTPUT_FORMAT)
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
                        .image(image);
                    let view = device
                        .create_image_view(&view_info, None)
                        .expect("failed to create image view");
                    FramebufferImageAttachment {
                        image,
                        view,
                        allocation,
                    }
                })
                .collect::<Vec<_>>();
            let mut depth_images = (0..number_frames)
                .map(|_| {
                    let depth_image_create_info = vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(VoxelPass::DEPTH_FORMAT)
                        .extent(output_resolution.into())
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE);
                    let image = device
                        .create_image(&depth_image_create_info, None)
                        .expect("failed to create depth image");
                    let depth_memory_requirements = device.get_image_memory_requirements(image);
                    let allocation = allocator
                        .allocate(&AllocationCreateDesc {
                            name: "depth image allocation",
                            requirements: depth_memory_requirements,
                            location: MemoryLocation::GpuOnly,
                            linear: true,
                            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                        })
                        .expect("failed to allocate depth image");
                    device
                        .bind_image_memory(image, allocation.memory(), allocation.offset())
                        .expect("failed to bind memory");
                    let depth_image_view_info = vk::ImageViewCreateInfo::default()
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .layer_count(1)
                                .level_count(1),
                        )
                        .image(image)
                        .format(VoxelPass::DEPTH_FORMAT)
                        .view_type(vk::ImageViewType::TYPE_2D);
                    let view = device
                        .create_image_view(&depth_image_view_info, None)
                        .expect("failed to create depth");
                    FramebufferImageAttachment {
                        image,
                        view,
                        allocation,
                    }
                })
                .collect::<Vec<_>>();
            setup_buffer.record_command_buffer(
                device,
                present_queue,
                &[],
                &[],
                &[],
                |device, command_buffer| {
                    let color_layout_transition_barriers = color_image
                        .iter()
                        .map(|image| {
                            vk::ImageMemoryBarrier::default()
                                .image(image.image)
                                .dst_access_mask(
                                    vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                                        | vk::AccessFlags::COLOR_ATTACHMENT_READ,
                                )
                                //  .new_layout(VoxelPass::COLOR_WRITE_ATTACHMENT_LAYOUT /* COLOR_ATTACHMENT_OPTIMAL */)
                                .new_layout(VoxelPass::COLOR_READ_ATTACHMENT_LAYOUT)
                                .old_layout(vk::ImageLayout::UNDEFINED)
                                .subresource_range(
                                    vk::ImageSubresourceRange::default()
                                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                                        .layer_count(1)
                                        .level_count(1),
                                )
                        })
                        .collect::<Vec<_>>();

                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &color_layout_transition_barriers,
                    );
                    let depth_layout_transition_barriers = depth_images
                        .iter()
                        .map(|depth| {
                            vk::ImageMemoryBarrier::default()
                                .image(depth.image)
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
                        command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &depth_layout_transition_barriers,
                    );
                },
            );
            setup_buffer.wait_for_command_completion(device);
            color_image
                .drain(..)
                .zip(depth_images.drain(..))
                .map(|(color, depth)| {
                    let framebuffer_attachments = [color.view, depth.view];
                    let framebuffer_create_info = vk::FramebufferCreateInfo::default()
                        .render_pass(*renderpass)
                        .attachments(&framebuffer_attachments)
                        .width(output_resolution.width)
                        .height(output_resolution.height)
                        .layers(1);
                    let framebuffer = device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .expect("failed to create device");
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
                    let color_sampler = device
                        .create_sampler(&sampler_info, None)
                        .expect("failed to create sampler");
                    FramebufferImage {
                        framebuffer,
                        color,
                        depth,
                        color_sampler,
                    }
                })
                .collect()
        }
    }
    pub fn free(self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.destroy_framebuffer(self.framebuffer, None);
            device.destroy_sampler(self.color_sampler, None);
        }
        self.color.free(device, allocator);
        self.depth.free(device, allocator);
    }
}

pub struct VoxelPass {
    render_model: RenderModel,
    framebuffers: Vec<FramebufferImage>,

    graphics_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    renderpass: vk::RenderPass,
    fragment_shader_module: vk::ShaderModule,
    vertex_shader_module: vk::ShaderModule,

    //does not need to be freed
    viewport: vk::Viewport,
    output_resolution: vk::Extent2D,
}
impl VoxelPass {
    const COLOR_OUTPUT_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
    pub const COLOR_WRITE_ATTACHMENT_LAYOUT: vk::ImageLayout =
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
    pub const COLOR_READ_ATTACHMENT_LAYOUT: vk::ImageLayout =
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    const DEPTH_FORMAT: vk::Format = vk::Format::D16_UNORM;
    pub fn new(
        device: &Device,
        allocator: &mut Allocator,
        setup_buffer: &mut SetupCommandBuffer,
        number_frames: u32,
        present_queue: vk::Queue,
        output_resolution: vk::Extent2D,
    ) -> Self {
        unsafe {
            let renderpass_attachments = [
                vk::AttachmentDescription2::default()
                    .format(Self::COLOR_OUTPUT_FORMAT)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
                vk::AttachmentDescription2::default()
                    .format(Self::DEPTH_FORMAT)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            ];

            let color_attachments = [vk::AttachmentReference2::default()
                .attachment(0)
                .layout(
                    Self::COLOR_WRITE_ATTACHMENT_LAYOUT, /* COLOR_ATTACHMENT_OPTIMAL */
                )
                .aspect_mask(vk::ImageAspectFlags::COLOR)];

            let depth_stencil_attachment = vk::AttachmentReference2::default()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL);

            let subpass_dependencies = [vk::SubpassDependency2::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)];

            let subpass = [vk::SubpassDescription2::default()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachments)
                .depth_stencil_attachment(&depth_stencil_attachment)
                .color_attachments(&color_attachments)];
            let renderpass_create_info = vk::RenderPassCreateInfo2::default()
                .attachments(&renderpass_attachments)
                .subpasses(&subpass)
                .dependencies(&subpass_dependencies);
            let renderpass = device
                .create_render_pass2(&renderpass_create_info, None)
                .expect("failed to create renderpass");

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
            let pipeline_layout = device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("failed to create pipeline layout");

            let mut fragment_spv_file =
                Cursor::new(include_bytes!("../../../shaders/voxel_frag.spv"));
            let fragment_code =
                read_spv(&mut fragment_spv_file).expect("failed to parse fragment shader file");

            let fragment_shader_info = vk::ShaderModuleCreateInfo::default().code(&fragment_code);
            let fragment_shader_module = device
                .create_shader_module(&fragment_shader_info, None)
                .expect("failed to create fragment shader module");
            let mut vertex_spv_file =
                Cursor::new(include_bytes!("../../../shaders/voxel_vert.spv"));
            let vertex_code =
                read_spv(&mut vertex_spv_file).expect("failed to parse vertex shader file");
            let vertex_shader_module_info =
                vk::ShaderModuleCreateInfo::default().code(&vertex_code);
            let vertex_shader_module = device
                .create_shader_module(&vertex_shader_module_info, None)
                .expect("failed to create voxel vertex shader module");
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
            let vertex_input_binding_description = RenderModelVertex::input_binding_description();
            let vertex_input_attribute_descriptions = RenderModelVertex::attribute_descriptions();
            let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
                .vertex_binding_descriptions(&vertex_input_binding_description);
            let vertex_input_assembly_state_info =
                vk::PipelineInputAssemblyStateCreateInfo::default()
                    .topology(vk::PrimitiveTopology::TRIANGLE_FAN);
            let viewports = [vk::Viewport {
                x: 0.,
                y: 0.,
                width: output_resolution.width as f32,
                height: output_resolution.height as f32,
                min_depth: 0.,
                max_depth: 1.,
            }];
            let scissors = [output_resolution.into()];
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

            let framebuffers = FramebufferImage::new_depth_and_color(
                device,
                allocator,
                setup_buffer,
                &renderpass,
                present_queue,
                number_frames,
                output_resolution,
            );
            let render_model = RenderModel::new_rectangle(device, allocator);
            Self {
                render_model,
                framebuffers,
                viewport: viewports[0],
                graphics_pipeline,
                pipeline_layout,
                renderpass,
                fragment_shader_module,
                vertex_shader_module,
                output_resolution,
            }
        }
    }

    pub fn draw(
        &mut self,
        device: &Device,
        draw_command_buffer: &DrawCommandBuffer,

        current_frame_index: usize,
    ) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0.5, 0., 1.],
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
            .framebuffer(self.framebuffers[current_frame_index].framebuffer)
            .render_area(self.output_resolution.into())
            .clear_values(&clear_values);
        unsafe {
            draw_command_buffer.record_command_buffer(device, |device, command_buffer| {
                let texture_barrier = vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::MEMORY_READ)
                    .dst_access_mask(vk::AccessFlags::MEMORY_WRITE)
                    .old_layout(Self::COLOR_READ_ATTACHMENT_LAYOUT)
                    .new_layout(Self::COLOR_WRITE_ATTACHMENT_LAYOUT)
                    .image(self.framebuffers[current_frame_index].color.image)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        level_count: 1,
                        layer_count: 1,
                        ..Default::default()
                    });

                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::ALL_COMMANDS,
                    vk::PipelineStageFlags::ALL_COMMANDS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[texture_barrier],
                );

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
                device.cmd_set_scissor(command_buffer, 0, &[self.output_resolution.into()]);
                device.cmd_bind_index_buffer(
                    command_buffer,
                    self.render_model.index_buffer,
                    0,
                    RenderModel::VK_INDEX_TYPE,
                );
                device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &[self.render_model.vertex_buffer],
                    &[0],
                );
                device.cmd_draw_indexed(
                    command_buffer,
                    self.render_model.number_indices() as u32,
                    1,
                    0,
                    0,
                    1,
                );
                device.cmd_end_render_pass(command_buffer);
                let texture_barrier = vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
                    .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                    .old_layout(Self::COLOR_WRITE_ATTACHMENT_LAYOUT)
                    .new_layout(Self::COLOR_READ_ATTACHMENT_LAYOUT)
                    .image(self.framebuffers[current_frame_index].color.image)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        level_count: 1,
                        layer_count: 1,
                        ..Default::default()
                    });

                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::ALL_COMMANDS,
                    vk::PipelineStageFlags::ALL_COMMANDS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[texture_barrier],
                );
            });
        }
    }
    pub fn textures(&self) -> Vec<ForeignTextureInput> {
        self.framebuffers
            .iter()
            .map(|frame_buffer| ForeignTextureInput {
                image_view: frame_buffer.color.view,
                sampler: frame_buffer.color_sampler,
                layout: Self::COLOR_READ_ATTACHMENT_LAYOUT,
            })
            .collect()
    }
    pub fn free(mut self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.device_wait_idle().expect("failed to wait idle");
            self.render_model.free(device, allocator);
            for framebuffer in self.framebuffers.drain(..) {
                framebuffer.free(device, allocator);
            }

            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_render_pass(self.renderpass, None);
            device.destroy_shader_module(self.fragment_shader_module, None);
            device.destroy_shader_module(self.vertex_shader_module, None);
        }
    }
}
