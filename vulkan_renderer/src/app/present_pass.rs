use super::Vertex;
use ash::{Device, util::read_spv, vk};
use std::{io::Cursor, mem::offset_of};
pub struct PresentPass {
    pub graphics_pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub renderpass: vk::RenderPass,
    pub fragment_shader_module: vk::ShaderModule,
    pub vertex_shader_module: vk::ShaderModule,
}
impl PresentPass {
    pub fn new(
        device: &Device,
        surface_resolution: vk::Extent2D,
        surface_format: vk::SurfaceFormatKHR,
    ) -> Self {
        unsafe {
            let renderpass_attachments = [
                vk::AttachmentDescription::default()
                    .format(surface_format.format)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
                vk::AttachmentDescription::default()
                    .format(vk::Format::D16_UNORM)
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
            let layout_create_info = vk::PipelineLayoutCreateInfo::default();
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
            let vertex_input_binding_description = [vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(size_of::<Vertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)];
            let vertex_input_attribute_descriptions = [
                vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(offset_of!(Vertex, pos) as u32),
                vk::VertexInputAttributeDescription::default()
                    .location(1)
                    .binding(0)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .offset(offset_of!(Vertex, color) as u32),
            ];
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
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::ZERO,
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
            }
        }
    }
    pub fn free(&self, device: &Device) {
        unsafe {
            device.device_wait_idle().expect("failed to wait idle");

            device.destroy_pipeline(self.graphics_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_shader_module(self.fragment_shader_module, None);
            device.destroy_shader_module(self.vertex_shader_module, None);
            device.destroy_render_pass(self.renderpass, None);
        }
    }
}
