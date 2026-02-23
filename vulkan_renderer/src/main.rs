use app::record_submit_command_buffer;
use ash::{
    Device, Entry, Instance,
    ext::debug_utils,
    khr,
    util::{Align, read_spv},
    vk,
};
use std::{
    borrow::Cow,
    ffi,
    io::Cursor,
    mem::{offset_of, size_of, size_of_val},
    u32, u64,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};
mod app;
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    unsafe {
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            Cow::from("")
        } else {
            ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            Cow::from("")
        } else {
            ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        println!(
            "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
        );
    }

    vk::FALSE
}

fn find_memorytype_index(
    memory_requirements: &vk::MemoryRequirements,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_properties
        .memory_types
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_requirements.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
#[repr(C)]
#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}
/// The maximum number of frames we allow to be in flight at any given time
const MAX_FRAME_LATENCY: usize = 3;
/// Contents are ordered in how they should be freed
struct App {
    graphics_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    fragment_shader_module: vk::ShaderModule,
    vertex_shader_module: vk::ShaderModule,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    framebuffers: Vec<vk::Framebuffer>,
    renderpass: vk::RenderPass,
    draw_commands_reuse_fence: [vk::Fence; MAX_FRAME_LATENCY],
    rendering_complete_semaphores: Vec<vk::Semaphore>,
    present_semaphores: [vk::Semaphore; MAX_FRAME_LATENCY],
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,
    depth_image: vk::Image,
    present_image_views: Vec<vk::ImageView>,
    present_images: Vec<vk::Image>,
    command_pool: vk::CommandPool,
    swapchain: vk::SwapchainKHR,
    device: Device,
    surface: vk::SurfaceKHR,
    surface_instance: khr::surface::Instance,
    debug_call_back: vk::DebugUtilsMessengerEXT,
    debug_utils_loader: debug_utils::Instance,
    instance: Instance,
    // doesnt need to be destroyed
    viewport: vk::Viewport,
    surface_resolution: vk::Extent2D,
    frame_index: usize,

    draw_command_buffers: [vk::CommandBuffer; MAX_FRAME_LATENCY],
    setup_command_buffer: vk::CommandBuffer,
    #[allow(dead_code)]
    app_setup_command_buffer: vk::CommandBuffer,
    #[allow(dead_code)]
    physical_device: vk::PhysicalDevice,
    present_queue: vk::Queue,
    swapchain_device: khr::swapchain::Device,
    #[allow(dead_code)]
    window: Window,
    #[allow(dead_code)]
    entry: Entry,
}
impl App {
    fn new(event_loop: &ActiveEventLoop, window_width: u32, window_height: u32) -> Self {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height)),
            )
            .unwrap();
        unsafe {
            let entry = Entry::load().unwrap();
            let app_name = c"Vulkan Voxel";
            let layer_names = [c"VK_LAYER_KHRONOS_validation"];
            let layer_names_raw: Vec<_> = layer_names.iter().map(|name| name.as_ptr()).collect();

            let mut extension_names = ash_window::enumerate_required_extensions(
                event_loop.display_handle().unwrap().as_raw(),
            )
            .unwrap()
            .to_vec();
            extension_names.push(debug_utils::NAME.as_ptr());
            let app_info = vk::ApplicationInfo::default()
                .application_name(app_name)
                .application_version(0)
                .engine_name(app_name)
                .engine_version(0)
                .api_version(vk::make_api_version(0, 1, 3, 0));
            let create_flags = vk::InstanceCreateFlags::default();
            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_layer_names(&layer_names_raw)
                .enabled_extension_names(&extension_names)
                .flags(create_flags);
            let instance = entry
                .create_instance(&create_info, None)
                .expect("failed to create instance");
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));
            let debug_utils_loader = debug_utils::Instance::new(&entry, &instance);
            let debug_call_back = debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap();
            let surface = ash_window::create_surface(
                &entry,
                &instance,
                window
                    .display_handle()
                    .expect("failed to get display handle")
                    .as_raw(),
                window
                    .window_handle()
                    .expect("failed to get handle")
                    .as_raw(),
                None,
            )
            .expect("failed to get surface");
            let physical_device_list = instance
                .enumerate_physical_devices()
                .expect("failed to get devices");
            let surface_instance = khr::surface::Instance::new(&entry, &instance);
            let (physical_device, queue_family_index) = physical_device_list
                .iter()
                .find_map(|physical_device| {
                    instance
                        .get_physical_device_queue_family_properties(*physical_device)
                        .iter()
                        .enumerate()
                        .find_map(|(queue_family_index, info)| {
                            let supports_graphics_and_surface =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_instance
                                        .get_physical_device_surface_support(
                                            *physical_device,
                                            queue_family_index as u32,
                                            surface,
                                        )
                                        .expect("failed to get instance support");
                            if supports_graphics_and_surface {
                                Some((*physical_device, queue_family_index as u32))
                            } else {
                                None
                            }
                        })
                })
                .expect("failed to get phusical device");
            let device_extension_names_raw = [khr::swapchain::NAME.as_ptr()];
            let priorities = [1.0];
            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);
            let device = instance
                .create_device(physical_device, &device_create_info, None)
                .expect("failed to create device");
            let present_queue = device.get_device_queue(queue_family_index, 0);
            let surface_format = surface_instance
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("failed to get formats")[0];
            let surface_capabilities = surface_instance
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("failed to get capabilities");
            let mut desired_image_count = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0
                && desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = surface_capabilities.max_image_count;
            }
            let surface_resolution = match surface_capabilities.current_extent.width {
                u32::MAX => vk::Extent2D {
                    width: window_width,
                    height: window_height,
                },
                _ => surface_capabilities.current_extent,
            };
            let pre_transform = if surface_capabilities
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_capabilities.current_transform
            };
            let present_modes = surface_instance
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("failed to get present modes");
            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|mode| *mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);
            let swapchain_device = khr::swapchain::Device::new(&instance, &device);
            let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_resolution)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);
            let swapchain = swapchain_device
                .create_swapchain(&swapchain_create_info, None)
                .expect("failed to create swapchain");
            let pool_create_info = vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index);
            let command_pool = device
                .create_command_pool(&pool_create_info, None)
                .expect("failed to create command pool");
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_buffer_count(2 + MAX_FRAME_LATENCY as u32)
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let command_buffers = device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("failed to create command buffers");
            let setup_command_buffer = command_buffers[0];
            let app_setup_command_buffer = command_buffers[1];
            let draw_command_buffers = command_buffers[2..][..MAX_FRAME_LATENCY]
                .try_into()
                .unwrap();
            let present_images = swapchain_device
                .get_swapchain_images(swapchain)
                .expect("failed to get present images");
            println!("swapchain images len: {}", present_images.len());
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
            let device_memory_properties =
                instance.get_physical_device_memory_properties(physical_device);
            let depth_image_create_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::D16_UNORM)
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
            let depth_memory_requirements = device.get_image_memory_requirements(depth_image);
            let depth_image_memory_index = find_memorytype_index(
                &depth_memory_requirements,
                &device_memory_properties,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("failed to find sutible index for depth image");
            let depth_image_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(depth_memory_requirements.size)
                .memory_type_index(depth_image_memory_index);
            let depth_image_memory = device
                .allocate_memory(&depth_image_allocate_info, None)
                .expect("failed to allocate");
            device
                .bind_image_memory(depth_image, depth_image_memory, 0)
                .expect("failed to bind memory");
            record_submit_command_buffer(
                &device,
                setup_command_buffer,
                vk::Fence::null(),
                present_queue,
                &[],
                &[],
                &[],
                |device, setup_command_buffer| {
                    let layout_transition_barriers = vk::ImageMemoryBarrier::default()
                        .image(depth_image)
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
                        );
                    device.cmd_pipeline_barrier(
                        setup_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barriers],
                    );
                },
            );
            let depth_image_view_info = vk::ImageViewCreateInfo::default()
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .layer_count(1)
                        .level_count(1),
                )
                .image(depth_image)
                .format(depth_image_create_info.format)
                .view_type(vk::ImageViewType::TYPE_2D);
            let depth_image_view = device
                .create_image_view(&depth_image_view_info, None)
                .expect("failed to create depth");
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_semaphores = std::array::from_fn(|_| {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("failed to create present complete semaphore")
            });
            let rendering_complete_semaphores = (0..present_images.len())
                .map(|_| {
                    device
                        .create_semaphore(&semaphore_create_info, None)
                        .expect("failed to create present complete semaphore")
                })
                .collect();
            let fence_create_info =
                vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            let draw_commands_reuse_fence = std::array::from_fn(|_| {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("failed to create fence")
            });
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
            let framebuffers = present_image_views
                .iter()
                .map(|present_image_view| {
                    let framebuffer_attachments = [*present_image_view, depth_image_view];
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
            let index_buffer_data = [0u32, 1, 2];
            let index_buffer_info = vk::BufferCreateInfo::default()
                .size(size_of_val(&index_buffer_data) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let index_buffer = device
                .create_buffer(&index_buffer_info, None)
                .expect("failed to get index buffer");
            let index_buffer_memory_requirements =
                device.get_buffer_memory_requirements(index_buffer);
            let index_buffer_memory_index = find_memorytype_index(
                &index_buffer_memory_requirements,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("failed to get memory type index");
            let index_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(index_buffer_memory_requirements.size)
                .memory_type_index(index_buffer_memory_index);

            let index_buffer_memory = device
                .allocate_memory(&index_allocate_info, None)
                .expect("failed to allocate memory");
            let index_ptr = device
                .map_memory(
                    index_buffer_memory,
                    0,
                    index_buffer_memory_requirements.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map");
            let mut index_slice = Align::new(
                index_ptr,
                align_of::<u32>() as u64,
                index_buffer_memory_requirements.size,
            );
            index_slice.copy_from_slice(&index_buffer_data);
            device.unmap_memory(index_buffer_memory);
            device
                .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
                .expect("failed to bind index buffer memory to index buffer");
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
            let vertex_input_buffer = vk::BufferCreateInfo::default()
                .size(size_of_val(&vertices) as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let vertex_buffer = device
                .create_buffer(&vertex_input_buffer, None)
                .expect("failed to create vertex buffer");
            let vertex_buffer_memory_requirements =
                device.get_buffer_memory_requirements(vertex_buffer);
            let vertex_buffer_memory_index = find_memorytype_index(
                &vertex_buffer_memory_requirements,
                &device_memory_properties,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
            .expect("failed to get memory requirements");
            let vertex_buffer_allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(vertex_buffer_memory_requirements.size)
                .memory_type_index(vertex_buffer_memory_index);
            let vertex_buffer_memory = device
                .allocate_memory(&vertex_buffer_allocate_info, None)
                .expect("failed to allocate memory for vertex buffer");
            let vertex_ptr = device
                .map_memory(
                    vertex_buffer_memory,
                    0,
                    vertex_buffer_memory_requirements.size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map vertex buffer to device space");
            let mut vertex_align = Align::new(
                vertex_ptr,
                align_of::<Vertex>() as u64,
                vertex_buffer_memory_requirements.size,
            );
            vertex_align.copy_from_slice(&vertices);
            device.unmap_memory(vertex_buffer_memory);
            device
                .bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)
                .expect("failed to bind memory");
            let mut vertex_spv_file = Cursor::new(include_bytes!("../shaders/vert.spv"));
            let mut fragment_spv_file = Cursor::new(include_bytes!("../shaders/frag.spv"));

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
                viewport: viewports[0],
                surface_resolution,
                frame_index: 0,
                graphics_pipeline,
                pipeline_layout,
                fragment_shader_module,
                vertex_shader_module,
                vertex_buffer_memory,
                vertex_buffer,
                index_buffer_memory,
                index_buffer,
                framebuffers,
                renderpass,
                draw_commands_reuse_fence,
                rendering_complete_semaphores,
                depth_image_view,
                depth_image_memory,
                depth_image,
                present_image_views,
                present_images,
                command_pool,
                swapchain,
                device,
                window,
                entry,
                instance,
                surface,
                debug_utils_loader,
                debug_call_back,
                surface_instance,
                physical_device,
                present_queue,
                swapchain_device,
                setup_command_buffer,
                app_setup_command_buffer,
                draw_command_buffers,
                present_semaphores,
            }
        }
    }
    fn request_redraw(&mut self) {
        let current_frame_index = self.frame_index % MAX_FRAME_LATENCY;
        let draw_commands_reuse_fence = self.draw_commands_reuse_fence[current_frame_index];
        unsafe {
            self.device
                .wait_for_fences(&[draw_commands_reuse_fence], true, u64::MAX)
                .expect("failed to wait for fence");
            self.device
                .reset_fences(&[draw_commands_reuse_fence])
                .expect("failed to reset fence");
        }

        unsafe {
            let present_complete_semaphore = self.present_semaphores[current_frame_index];
            let draw_command_buffer = self.draw_command_buffers[current_frame_index];
            let (present_index, _) = self
                .swapchain_device
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    present_complete_semaphore,
                    vk::Fence::null(),
                )
                .expect("failed to acquire next image");
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
            let rendering_complete_semaphore =
                self.rendering_complete_semaphores[current_frame_index];
            let renderpass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.renderpass)
                .framebuffer(self.framebuffers[current_frame_index])
                .render_area(self.surface_resolution.into())
                .clear_values(&clear_values);
            record_submit_command_buffer(
                &self.device,
                draw_command_buffer,
                draw_commands_reuse_fence,
                self.present_queue,
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
                    device.cmd_bind_index_buffer(
                        command_buffer,
                        self.index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.vertex_buffer], &[0]);
                    device.cmd_draw_indexed(command_buffer, 3, 1, 0, 0, 1);
                    device.cmd_end_render_pass(command_buffer);
                },
            );
            let wait_semaphore = [rendering_complete_semaphore];
            let swapchains = [self.swapchain];
            let present_indices = [present_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&wait_semaphore)
                .swapchains(&swapchains)
                .image_indices(&present_indices);
            self.swapchain_device
                .queue_present(self.present_queue, &present_info)
                .expect("failed to present queue");
        }

        self.frame_index += 1;

        //self.window.request_redraw()
    }
}
impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().expect("failed to wait idle");
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_shader_module(self.fragment_shader_module, None);
            self.device
                .destroy_shader_module(self.vertex_shader_module, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.index_buffer_memory, None);
            self.device.destroy_buffer(self.index_buffer, None);
            for framebuffer in self.framebuffers.drain(..) {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            self.device.destroy_render_pass(self.renderpass, None);
            for fence in self.draw_commands_reuse_fence {
                self.device.destroy_fence(fence, None);
            }
            for semaphore in self.rendering_complete_semaphores.iter().copied() {
                self.device.destroy_semaphore(semaphore, None);
            }
            for semaphore in self.present_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.free_memory(self.depth_image_memory, None);
            self.device.destroy_image(self.depth_image, None);
            for view in self.present_image_views.iter() {
                self.device.destroy_image_view(*view, None);
            }
            for image in self.present_images.drain(..) {
                self.device.destroy_image(image, None);
            }
            self.device
                .reset_command_buffer(
                    self.setup_command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("failed to reset buffer");
            self.device.destroy_command_pool(self.command_pool, None);
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_instance.destroy_surface(self.surface, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Default)]
struct WindowContainer {
    app: Option<App>,
}
impl ApplicationHandler for WindowContainer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.app = Some(App::new(event_loop, 1024, 800));
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(app) = self.app.as_mut() {
            app.request_redraw();
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            _ => (),
        }
    }
}
fn main() {
    let event_loop = EventLoop::new().unwrap();
    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = WindowContainer::default();

    event_loop
        .run_app(&mut app)
        .expect("failed to start event loop");
}
