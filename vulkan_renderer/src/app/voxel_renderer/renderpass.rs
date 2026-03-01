use super::super::SetupCommandBuffer;
use ash::{
    Device,
    util::read_spv,
    vk::{self},
};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};
use std::{
    io::Cursor,
    mem::{offset_of, size_of},
};
#[repr(C)]
struct VoxelPassVertex {
    pub position: [f32; 4],
}
impl VoxelPassVertex {
    const fn input_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
    const fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 1] {
        [vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: offset_of!(Self, position) as u32,
        }]
    }
}
pub struct VoxelPass {
    color_image_allocation: Allocation,
    color_image: vk::Image,
    graphics_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    renderpass: vk::RenderPass,
    fragment_shader_module: vk::ShaderModule,
    vertex_shader_module: vk::ShaderModule,
}
impl VoxelPass {
    //const COLOR_OUTPUT_FORMAT: vk::Format = vk::Format::R8G8B8A8_SRGB;
    const COLOR_OUTPUT_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
    const DEPTH_FORMAT: vk::Format = vk::Format::D16_UNORM;
    pub fn new(
        device: &Device,
        allocator: &mut Allocator,
        setup_buffer: &mut SetupCommandBuffer,
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
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
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
            let vertex_input_binding_description = VoxelPassVertex::input_binding_description();
            let vertex_input_attribute_descriptions = VoxelPassVertex::attribute_descriptions();
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

            let color_image_create_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(Self::COLOR_OUTPUT_FORMAT)
                .extent(output_resolution.into())
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let color_image = device
                .create_image(&color_image_create_info, None)
                .expect("failed to create image");
            let color_image_requirements = device.get_image_memory_requirements(color_image);
            let color_image_allocation = allocator
                .allocate(&AllocationCreateDesc {
                    name: "voxel pass color buffer",
                    requirements: color_image_requirements,
                    location: MemoryLocation::GpuOnly,
                    linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .expect("failed to allocate memory for color image");
            device
                .bind_image_memory(
                    color_image,
                    color_image_allocation.memory(),
                    color_image_allocation.offset(),
                )
                .expect("failed to bind color image");
            setup_buffer.record_command_buffer(
                device,
                present_queue,
                &[],
                &[],
                &[],
                |device, command_buffer| {
                    let layout_transition_barrier = vk::ImageMemoryBarrier::default()
                        .image(color_image)
                        .dst_access_mask(
                            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                                | vk::AccessFlags::COLOR_ATTACHMENT_READ,
                        )
                        .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .layer_count(1)
                                .level_count(1),
                        );
                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barrier],
                    );
                },
            );
            Self {
                color_image_allocation,
                color_image,
                graphics_pipeline,
                pipeline_layout,
                renderpass,
                fragment_shader_module,
                vertex_shader_module,
            }
        }
    }
    pub fn draw(&self) {
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
    }
    pub fn free(self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            device.device_wait_idle().expect("failed to wait idle");
            allocator
                .free(self.color_image_allocation)
                .expect("failed to free color image allocation");
            device.destroy_image(self.color_image, None);
            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_render_pass(self.renderpass, None);
            device.destroy_shader_module(self.fragment_shader_module, None);
            device.destroy_shader_module(self.vertex_shader_module, None);
        }
    }
}
