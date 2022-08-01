use super::{find_memory_type_index, record_submit_commandbuffer, Base};
use ash::{
    util::{read_spv, Align},
    vk,
};

use std::{
    ffi::CStr,
    io::Cursor,
    mem::{align_of, size_of},
};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}
pub fn run(base: &Base) {
    let render_pass_attachments = [
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
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build(),
    ];
    let color_attachment_ref = [vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build()];
    let depth_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();
    let dependencies = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .build()];
    let subpass = vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_ref)
        .depth_stencil_attachment(&depth_attachment_ref)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);
    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&render_pass_attachments)
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(&dependencies);
    let renderpass = unsafe {
        base.device
            .create_render_pass(&renderpass_create_info, None)
            .expect("failed to create renderpass")
    };
    let framebuffers = base
        .present_image_views
        .iter()
        .map(|&present_image_view| {
            let framebuffer_attachments = [present_image_view, base.depth_image_view];
            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&framebuffer_attachments)
                .width(base.surface_resolution.width)
                .height(base.surface_resolution.height)
                .layers(1);
            unsafe {
                base.device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("failed to create framebuffer")
            }
        })
        .collect::<Vec<_>>();
    let index_buffer_data = [0u32, 1, 2];
    let index_buffer_info = vk::BufferCreateInfo::builder()
        .size(std::mem::size_of_val(&index_buffer_data) as u64)
        .usage(vk::BufferUsageFlags::INDEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let index_buffer = unsafe {
        base.device
            .create_buffer(&index_buffer_info, None)
            .expect("failed to create buffer")
    };
    let index_buffer_req = unsafe { base.device.get_buffer_memory_requirements(index_buffer) };
    let index_buffer_memory_index = find_memory_type_index(
        &index_buffer_req,
        &base.device_memory_properties,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .expect("failed to find sutible memory type");
    let index_alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(index_buffer_req.size)
        .memory_type_index(index_buffer_memory_index);
    let index_buffer_memory = unsafe {
        base.device
            .allocate_memory(&index_alloc_info, None)
            .unwrap()
    };
    let index_ptr = unsafe {
        base.device
            .map_memory(
                index_buffer_memory,
                0,
                index_buffer_req.size,
                vk::MemoryMapFlags::empty(),
            )
            .unwrap()
    };
    let mut index_slice =
        unsafe { Align::new(index_ptr, align_of::<u32>() as u64, index_buffer_req.size) };
    index_slice.copy_from_slice(&index_buffer_data);
    unsafe {
        base.device.unmap_memory(index_buffer_memory);
        base.device
            .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
            .unwrap();
    }

    let vertex_input_buffer_info = vk::BufferCreateInfo::builder()
        .size(3 * std::mem::size_of::<Vertex>() as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let vertex_input_buffer = unsafe {
        base.device
            .create_buffer(&vertex_input_buffer_info, None)
            .expect("failed to make vertex buffer")
    };

    let vertex_input_buffer_memory_req = unsafe {
        base.device
            .get_buffer_memory_requirements(vertex_input_buffer)
    };
    let vertex_input_buffer_memory_index = find_memory_type_index(
        &vertex_input_buffer_memory_req,
        &base.device_memory_properties,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .expect("failed to find suitable memory");
    let vertex_buffer_allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(vertex_input_buffer_memory_req.size)
        .memory_type_index(vertex_input_buffer_memory_index);
    let vertex_input_buffer_memory = unsafe {
        base.device
            .allocate_memory(&vertex_buffer_allocate_info, None)
            .expect("failed to allocate memory")
    };
    let vertices = [
        Vertex {
            pos: [-1.0, 1.0, 0.0, 1.0],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [1.0, 1.0, 0.0, 1.0],
            color: [0.0, 0.0, 1.0, 1.0],
        },
        Vertex {
            pos: [0.0, -1.0, 0.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
    ];
    let vert_ptr = unsafe {
        base.device
            .map_memory(
                vertex_input_buffer_memory,
                0,
                vertex_input_buffer_memory_req.size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("failed to map memory")
    };
    let mut vert_align = unsafe {
        Align::new(
            vert_ptr,
            align_of::<Vertex>() as u64,
            vertex_input_buffer_memory_req.size,
        )
    };
    vert_align.copy_from_slice(&vertices);
    unsafe {
        base.device.unmap_memory(vertex_input_buffer_memory);
        base.device
            .bind_buffer_memory(vertex_input_buffer, vertex_input_buffer_memory, 0)
            .expect("failed to bind memory");
    }
    let mut vertex_spv_file = Cursor::new(&include_bytes!("../shaders/bin/triangle.vert.glsl"));
    let mut frag_spv_file = Cursor::new(&include_bytes!("../shaders/bin/triangle.frag.glsl"));
    let vertex_code = read_spv(&mut vertex_spv_file).expect("failed to read vertex code");
    let vertex_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);
    let frag_code = read_spv(&mut frag_spv_file).expect("failed to read file");
    let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);
    let vertex_shader_module = unsafe {
        base.device
            .create_shader_module(&vertex_shader_info, None)
            .expect("failed to create vertex shader module")
    };
    let frag_shader_module = unsafe {
        base.device
            .create_shader_module(&frag_shader_info, None)
            .expect("failed to create frag shader")
    };
    let layout_create_info = vk::PipelineLayoutCreateInfo::default();
    let pipeline_layout = unsafe {
        base.device
            .create_pipeline_layout(&layout_create_info, None)
            .expect("failed to create layout")
    };
    let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
    let shader_stage_create_infos = [
        vk::PipelineShaderStageCreateInfo::builder()
            .module(vertex_shader_module)
            .name(shader_entry_name)
            .stage(vk::ShaderStageFlags::VERTEX)
            .build(),
        vk::PipelineShaderStageCreateInfo::builder()
            .module(frag_shader_module)
            .name(shader_entry_name)
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .build(),
    ];
    let vertex_input_binding_description = [vk::VertexInputBindingDescription::builder()
        .binding(0)
        .stride(size_of::<Vertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX)
        .build()];
    let vertex_input_attribute_descriptions = [
        vk::VertexInputAttributeDescription::builder()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(0)
            .build(),
        vk::VertexInputAttributeDescription::builder()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(4 * size_of::<f32>() as u32)
            .build(),
    ];
    let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
        .vertex_binding_descriptions(&vertex_input_binding_description);
    let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let viewports = [vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(base.surface_resolution.width as f32)
        .height(base.surface_resolution.height as f32)
        .min_depth(0.0)
        .max_depth(1.0)
        .build()];
    let scissors = [base.surface_resolution.into()];
    let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports)
        .build();
    let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0)
        .polygon_mode(vk::PolygonMode::FILL);
    let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
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
        blend_enable: 0,
        src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::RGBA,
    }];
    let color_blend_states = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&color_blend_attachment_states);
    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);
    let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stage_create_infos)
        .vertex_input_state(&vertex_input_state_info)
        .input_assembly_state(&vertex_input_assembly_state_info)
        .viewport_state(&viewport_state_info)
        .rasterization_state(&rasterization_info)
        .multisample_state(&multisample_state_info)
        .depth_stencil_state(&depth_state_info)
        .color_blend_state(&color_blend_states)
        .dynamic_state(&dynamic_state_info)
        .layout(pipeline_layout)
        .render_pass(renderpass)
        .build();
    let graphics_pipeline = unsafe {
        base.device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info], None)
            .expect("failed to create graphics pipeline")[0]
    };
    base.render_loop(|_| {
        let (present_index, _) = unsafe {
            base.swapchain_loader
                .acquire_next_image(
                    base.swapchain,
                    u64::MAX,
                    base.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .expect("failed to acquire image")
        };
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass)
            .framebuffer(framebuffers[present_index as usize])
            .render_area(base.surface_resolution.into())
            .clear_values(&clear_values);
        unsafe {
            record_submit_commandbuffer(
                &base.device,
                base.draw_command_buffer,
                base.draw_commands_reuse_fence,
                base.present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[base.present_complete_semaphore],
                &[base.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    device.cmd_bind_pipeline(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        graphics_pipeline,
                    );
                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    device.cmd_bind_vertex_buffers(
                        draw_command_buffer,
                        0,
                        &[vertex_input_buffer],
                        &[0],
                    );

                    device.cmd_bind_index_buffer(
                        draw_command_buffer,
                        index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.cmd_draw_indexed(
                        draw_command_buffer,
                        index_buffer_data.len() as u32,
                        1,
                        0,
                        0,
                        1,
                    );
                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );
        }
        let wait_semaphors = [base.rendering_complete_semaphore];
        let swapchains = [base.swapchain];
        let image_indices = [present_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphors)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        unsafe {
            base.swapchain_loader
                .queue_present(base.present_queue, &present_info)
                .expect("failed to present image");
        }
    });
    unsafe {
        base.device.device_wait_idle().expect("failed to wait idle");
        base.device.destroy_pipeline(graphics_pipeline, None);
        base.device.destroy_pipeline_layout(pipeline_layout, None);
        base.device
            .destroy_shader_module(vertex_shader_module, None);
        base.device.destroy_shader_module(frag_shader_module, None);
        base.device.free_memory(index_buffer_memory, None);
        base.device.destroy_buffer(index_buffer, None);
        base.device.free_memory(vertex_input_buffer_memory, None);
        base.device.destroy_buffer(vertex_input_buffer, None);
        for fb in framebuffers {
            base.device.destroy_framebuffer(fb, None)
        }
        base.device.destroy_render_pass(renderpass, None);
    }
}
