use ash::{Device, Entry, Instance, ext::debug_utils, khr, vk};
use std::{borrow::Cow, char::MAX, ffi, u32};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::{Window, WindowId},
};
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
unsafe fn record_submit_command_buffer<F: FnOnce(&Device, vk::CommandBuffer)>(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    command_buffer_reuse_fence: vk::Fence,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    unsafe {
        device
            .reset_command_buffer(
                command_buffer,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )
            .expect("reset command buffer failed");
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("failed to begin command buffer");
        f(device, command_buffer);
        device
            .end_command_buffer(command_buffer)
            .expect("failed to create command buffer");
        let command_buffers = vec![command_buffer];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);
        device
            .queue_submit(submit_queue, &[submit_info], command_buffer_reuse_fence)
            .expect("failed to submit queue");
    }
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
/// The maximum number of frames we allow to be in flight at any given time
const MAX_FRAME_LATENCY: usize = 3;
/// Contents are ordered in how they should be freed
struct App {
    draw_commands_reuse_fence: [vk::Fence; MAX_FRAME_LATENCY],
    rendering_complete_semaphores: Vec<vk::Semaphore>,
    present_semaphores: [vk::Semaphore; MAX_FRAME_LATENCY],
    depth_image_view: vk::ImageView,
    depth_image_memory: vk::DeviceMemory,
    depth_image: vk::Image,
    present_image_views: Vec<vk::ImageView>,
    command_pool: vk::CommandPool,
    swapchain: vk::SwapchainKHR,
    device: Device,
    surface: vk::SurfaceKHR,
    surface_instance: khr::surface::Instance,
    debug_call_back: vk::DebugUtilsMessengerEXT,
    debug_utils_loader: debug_utils::Instance,
    instance: Instance,
    // doesnt need to be destroyed
    present_images: Vec<vk::Image>,
    draw_command_buffers: [vk::CommandBuffer; MAX_FRAME_LATENCY],
    setup_command_buffer: vk::CommandBuffer,
    app_setup_command_buffer: vk::CommandBuffer,
    physical_device: vk::PhysicalDevice,
    present_queue: vk::Queue,
    swapchain_device: khr::swapchain::Device,
    window: Window,
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
                .api_version(vk::make_api_version(0, 1, 0, 0));
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
            let mut desired_image_count = surface_capabilities.max_image_count + 1;
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
            Self {
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
        self.window.request_redraw()
    }
}
impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().expect("failed to wait idle");
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
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
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
