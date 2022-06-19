use super::{find_memory_type_index, prelude::*, Base};
use crate::prelude::Animation;
use crate::record_submit_commandbuffer;
use ash::vk::SemaphoreWaitInfo;
use ash::{
    util::{read_spv, Align},
    vk,
};
use cgmath::{SquareMatrix, Vector3};
use gpu_allocator::vulkan::*;
use gpu_allocator::{AllocatorDebugSettings, MemoryLocation};
use image::RgbaImage;
use std::ffi::c_void;
use std::{
    default::Default,
    ffi::CStr,
    io::Cursor,
    mem::{align_of, size_of, size_of_val},
    rc::Rc,
};

struct RenderModel {
    index_buffer: vk::Buffer,
    index_allocation: Allocation,
    vertex_buffer: vk::Buffer,
    descriptor_set: vk::DescriptorSet,
    vertex_allocation: Allocation,
    texture_image: vk::Image,
    texture_allocation: Allocation,
    texture_image_view: vk::ImageView,
    sampler: vk::Sampler,
    num_indices: u32,
    animation: AnimationList,
}
impl RenderModel {
    pub unsafe fn free_resources(self, base: &Base, allocator: &mut Allocator) {
        base.device.device_wait_idle().expect("failed to wait idle");
        base.device
            .destroy_image_view(self.texture_image_view, None);
        base.device.destroy_sampler(self.sampler, None);
        base.device.destroy_image(self.texture_image, None);
        allocator
            .free(self.texture_allocation)
            .expect("failed to free texture allocation");

        base.device.destroy_buffer(self.vertex_buffer, None);
        allocator
            .free(self.vertex_allocation)
            .expect("failed to free allocation");
        base.device.destroy_buffer(self.index_buffer, None);
        allocator
            .free(self.index_allocation)
            .expect("failed to destroy index allocation");
    }
}
struct Model {
    animation: AnimationList,
    mesh: Mesh,
    texture: image::RgbaImage,
}
impl Model {
    pub fn build_render_model(
        &self,
        base: &Base,
        allocator: &mut Allocator,
        descriptor_pool: &vk::DescriptorPool,
        descriptor_layouts: &[vk::DescriptorSetLayout],
    ) -> RenderModel {
        let index_buffer_info = vk::BufferCreateInfo::builder()
            .size(size_of::<u32>() as u64 * self.mesh.indices.len() as u64)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let index_buffer = unsafe {
            base.device
                .create_buffer(&index_buffer_info, None)
                .expect("failed to create index buffer")
        };
        let index_buffer_memory_req =
            unsafe { base.device.get_buffer_memory_requirements(index_buffer) };
        let index_allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "index buffer memory",
                requirements: index_buffer_memory_req,
                location: MemoryLocation::CpuToGpu,
                linear: true,
            })
            .expect("failed to allocate");
        let index_ptr = index_allocation.mapped_ptr().expect("failed to map ptr");

        let mut index_slice = unsafe {
            Align::new(
                index_ptr.as_ptr(),
                align_of::<u32>() as u64,
                index_buffer_memory_req.size,
            )
        };
        unsafe {
            index_slice.copy_from_slice(&self.mesh.indices);

            base.device
                .bind_buffer_memory(
                    index_buffer,
                    index_allocation.memory(),
                    index_allocation.offset(),
                )
                .unwrap();
        }

        let vertex_input_buffer_info = vk::BufferCreateInfo::builder()
            .size(size_of::<Vertex>() as u64 * self.mesh.vertices.len() as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let vertex_buffer = unsafe {
            base.device
                .create_buffer(&vertex_input_buffer_info, None)
                .expect("failed to create vertex input buffer")
        };
        let vertex_input_buffer_memory_req =
            unsafe { base.device.get_buffer_memory_requirements(vertex_buffer) };
        let vertex_allocation = allocator
            .allocate(&AllocationCreateDesc {
                requirements: vertex_input_buffer_memory_req,
                location: MemoryLocation::CpuToGpu,
                linear: true,
                name: "Vertex Buffer",
            })
            .expect("failed to allocate");
        let vert_ptr = vertex_allocation
            .mapped_ptr()
            .expect("failed to map vertex ptr");

        unsafe {
            let mut slice = Align::new(
                vert_ptr.as_ptr(),
                align_of::<Vertex>() as u64,
                vertex_input_buffer_memory_req.size,
            );
            slice.copy_from_slice(&self.mesh.vertices);

            base.device
                .bind_buffer_memory(
                    vertex_buffer,
                    vertex_allocation.memory(),
                    vertex_allocation.offset(),
                )
                .expect("failed to bind vertex buffer memory");
        }
        let desc_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool.clone())
            .set_layouts(descriptor_layouts);
        let descriptor_set = unsafe {
            base.device
                .allocate_descriptor_sets(&desc_alloc_info)
                .expect("failed to allocate desc layout")
        }[0];
        let (width, height) = self.texture.dimensions();
        let image_extent = vk::Extent2D { width, height };
        let image_data = self.texture.clone().into_raw();
        let image_buffer_info = vk::BufferCreateInfo::builder()
            .size(size_of::<u8>() as u64 * image_data.len() as u64)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let image_buffer = unsafe {
            base.device
                .create_buffer(&image_buffer_info, None)
                .expect("failed to get buffer")
        };
        let image_buffer_memory_req =
            unsafe { base.device.get_buffer_memory_requirements(image_buffer) };

        let image_buffer_allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "image buffer allocation",
                requirements: image_buffer_memory_req,
                location: MemoryLocation::CpuToGpu,
                linear: true,
            })
            .expect("failed to make allocation");

        let img_ptr = image_buffer_allocation
            .mapped_ptr()
            .expect("failed to map image pointer");
        unsafe {
            let mut image_slice = Align::new(
                img_ptr.as_ptr(),
                align_of::<u8>() as u64,
                image_buffer_memory_req.size,
            );
            image_slice.copy_from_slice(&image_data);

            base.device
                .bind_buffer_memory(
                    image_buffer,
                    image_buffer_allocation.memory(),
                    image_buffer_allocation.offset(),
                )
                .expect("failed to bind texture memory");
        }
        let texture_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(image_extent.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let texture_image = unsafe {
            base.device
                .create_image(&texture_create_info, None)
                .expect("failed to create image")
        };
        let texture_memory_req =
            unsafe { base.device.get_image_memory_requirements(texture_image) };
        let texture_memory_index = unsafe {
            find_memory_type_index(
                &texture_memory_req,
                &base.device_memory_properties,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("failed to get memory index")
        };
        let texture_allocation = allocator
            .allocate(&AllocationCreateDesc {
                name: "image buffer allocation",
                requirements: texture_memory_req,
                location: MemoryLocation::GpuOnly,
                linear: true,
            })
            .expect("failed to free allocation");

        unsafe {
            base.device
                .bind_image_memory(
                    texture_image,
                    texture_allocation.memory(),
                    texture_allocation.offset(),
                )
                .expect("failed to bind image memory");
        }

        unsafe {
            record_submit_commandbuffer(
                &base.device,
                base.setup_command_buffer,
                base.setup_commands_reuse_fence,
                base.present_queue,
                &[],
                &[],
                &[],
                |device, texture_command_buffer| {
                    let texture_barrier = vk::ImageMemoryBarrier::builder()
                        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .image(texture_image)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .level_count(1)
                                .layer_count(1)
                                .build(),
                        )
                        .build();
                    device.cmd_pipeline_barrier(
                        texture_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier],
                    );
                    let buffer_copy_region = vk::BufferImageCopy::builder()
                        .image_subresource(
                            vk::ImageSubresourceLayers::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .layer_count(1)
                                .build(),
                        )
                        .image_extent(image_extent.into())
                        .build();
                    device.cmd_copy_buffer_to_image(
                        texture_command_buffer,
                        image_buffer,
                        texture_image,
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        &[buffer_copy_region],
                    );
                    let texture_barrier_end = vk::ImageMemoryBarrier::builder()
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image(texture_image)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .level_count(1)
                                .layer_count(1)
                                .build(),
                        )
                        .build();
                    device.cmd_pipeline_barrier(
                        texture_command_buffer,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[texture_barrier_end],
                    );
                },
            )
        }
        allocator
            .free(image_buffer_allocation)
            .expect("failed to free");
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_v(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .address_mode_w(vk::SamplerAddressMode::MIRRORED_REPEAT)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .compare_op(vk::CompareOp::NEVER);
        let sampler = unsafe {
            base.device
                .create_sampler(&sampler_info, None)
                .expect("failed to get sampler")
        };
        let tex_image_view_info = vk::ImageViewCreateInfo::builder()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(texture_create_info.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            })
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(texture_image);
        let texture_image_view = unsafe {
            base.device
                .create_image_view(&tex_image_view_info, None)
                .expect("failed to get tex image view")
        };

        let tex_descriptor = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: texture_image_view,
            sampler,
        };
        let write_desc_sets = [vk::WriteDescriptorSet {
            dst_set: descriptor_set,
            dst_binding: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &tex_descriptor,
            ..Default::default()
        }];

        unsafe {
            base.device.update_descriptor_sets(&write_desc_sets, &[]);
            base.device
                .wait_for_fences(&[base.setup_commands_reuse_fence], true, u64::MAX);

            base.device.destroy_buffer(image_buffer, None);
        }
        RenderModel {
            index_buffer,
            index_allocation,
            vertex_buffer,
            vertex_allocation,
            descriptor_set,
            texture_image,
            texture_allocation,
            texture_image_view,
            sampler,
            num_indices: self.mesh.indices.len() as u32,
            animation: self.animation.clone(),
        }
    }
}

pub fn run(base: &Base) {
    let mut allocator = Allocator::new(&AllocatorCreateDesc {
        instance: base.instance.clone(),
        device: base.device.clone(),
        physical_device: base.p_device.clone(),
        debug_settings: AllocatorDebugSettings::default(),
        buffer_device_address: false,
    })
    .expect("created allocator");
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
        .max_sets(2);
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
    let desc_set_layouts = unsafe {
        [base
            .device
            .create_descriptor_set_layout(&descriptor_info, None)
            .expect("failed to create descriptor set layout")]
    };
    let sun = Model {
        mesh: Mesh::sphere(64, 32),
        texture: image::RgbaImage::from_pixel(100, 100, image::Rgba([255, 255, 255, 255])),
        animation: AnimationList::new(vec![Rc::new(StaticPosition {
            position: cgmath::Point3::new(0.0, 1.0, -4.0),
        })]),
    }
    .build_render_model(base, &mut allocator, &descriptor_pool, &desc_set_layouts);
    let planet = Model {
        mesh: Mesh::sphere(64, 32),
        texture: image::load_from_memory(include_bytes!("../../assets/earthmap.jpg"))
            .unwrap()
            .to_rgba8(),
        animation: AnimationList::new(vec![
            Rc::new(StaticPosition {
                position: cgmath::Point3::new(0.0, 1.0, -4.0),
            }),
            Rc::new(Orbit {
                radius: 2.0,
                orbit_period: 10000.0,
            }),
            Rc::new(RotateX { rotate_rate: 0.01 }),
            Rc::new(Scale {
                scale: Vector3::new(0.1, 0.1, 0.1),
            }),
        ]),
    }
    .build_render_model(base, &mut allocator, &descriptor_pool, &desc_set_layouts);
    let mut mesh_list = vec![sun, planet];

    let mut vertex_spv_file = Cursor::new(include_bytes!("../shaders/bin/push.vert.glsl"));
    let mut frag_spv_file = Cursor::new(include_bytes!("../shaders/bin/push.frag.glsl"));
    let vertex_code = read_spv(&mut vertex_spv_file).expect("failed tp read vertex shader code");
    let vert_shader_info = vk::ShaderModuleCreateInfo::builder().code(&vertex_code);
    let frag_code = read_spv(&mut frag_spv_file).expect("failed to read fragment spv file");
    let frag_shader_info = vk::ShaderModuleCreateInfo::builder().code(&frag_code);
    let vertex_shader_module = unsafe {
        base.device
            .create_shader_module(&vert_shader_info, None)
            .expect("vertex shader compile info")
    };
    let frag_shader_module = unsafe {
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
        .set_layouts(&desc_set_layouts);
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
            .module(frag_shader_module)
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
    let graphics_pipelines = unsafe {
        base.device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[graphics_pipeline_info], None)
            .expect("failed to create graphics_pipeline")[0]
    };

    base.render_loop(|frame_counter| {
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
                &[vk::PipelineStageFlags::BOTTOM_OF_PIPE],
                &[base.present_complete_semaphore],
                &[base.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &renderpass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    for mesh in mesh_list.iter() {
                        let transform_mat =
                            cgmath::perspective(cgmath::Rad(3.14 / 2.0), 1.0, 0.1, 10.0)
                                * mesh.animation.build_transform_mat(frame_counter);

                        device.cmd_bind_descriptor_sets(
                            draw_command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            pipeline_layout,
                            0,
                            &[mesh.descriptor_set],
                            &[],
                        );
                        device.cmd_bind_pipeline(
                            draw_command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            graphics_pipelines,
                        );
                        device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                        device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                        device.cmd_bind_vertex_buffers(
                            draw_command_buffer,
                            0,
                            &[mesh.vertex_buffer],
                            &[0],
                        );
                        device.cmd_push_constants(
                            draw_command_buffer,
                            pipeline_layout,
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

                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );
            let present_index_arr = [present_index];
            let render_complete_sem_arr = [base.rendering_complete_semaphore];

            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(std::slice::from_ref(&base.rendering_complete_semaphore))
                .swapchains(std::slice::from_ref(&base.swapchain))
                .image_indices(std::slice::from_ref(&present_index));
            base.swapchain_loader
                .queue_present(base.present_queue, &present_info)
                .expect("failed to present render");
        }
    });
    unsafe {
        base.device.device_wait_idle().expect("failed to wait idle");
        base.device.destroy_pipeline(graphics_pipelines, None);
        base.device.destroy_pipeline_layout(pipeline_layout, None);
        base.device
            .destroy_shader_module(vertex_shader_module, None);
        base.device.destroy_shader_module(frag_shader_module, None);

        for mesh in mesh_list.drain(..) {
            mesh.free_resources(base, &mut allocator)
        }
        for &descriptor_set_layout in desc_set_layouts.iter() {
            base.device
                .destroy_descriptor_set_layout(descriptor_set_layout, None);
        }
        base.device.destroy_descriptor_pool(descriptor_pool, None);

        for framebuffer in framebuffers {
            base.device.destroy_framebuffer(framebuffer, None);
        }
        base.device.destroy_render_pass(renderpass, None);
        drop(allocator);
    }
}
