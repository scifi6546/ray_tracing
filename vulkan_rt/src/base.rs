mod extension_manager;
use super::{find_memory_type_index, record_submit_commandbuffer};
use ash::{
    extensions::{
        ext::DebugUtils,
        khr::{Surface, Swapchain},
    },
    vk, Device, Entry, Instance,
};
use extension_manager::ExtensionManager;
use std::{borrow::Cow, cell::RefCell, ffi::CStr, os::raw::c_char};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};
pub struct Base {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub surface_loader: Surface,
    pub swapchain_loader: Swapchain,
    pub debug_utils_loader: DebugUtils,
    pub window: winit::window::Window,
    pub event_loop: RefCell<EventLoop<()>>,
    pub debug_callback: vk::DebugUtilsMessengerEXT,

    pub p_device: vk::PhysicalDevice,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,

    pub pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,

    pub draw_commands_reuse_fence: vk::Fence,
    pub setup_commands_reuse_fence: vk::Fence,

    pub window_width: u32,
    pub window_height: u32,
    pub extension_manager: ExtensionManager,
}
impl Base {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Ray Tracing Example")
            .with_inner_size(winit::dpi::LogicalSize::new(
                window_width as f32,
                window_height as f32,
            ))
            .build(&event_loop)
            .unwrap();

        let app_info = vk::ApplicationInfo::builder().api_version(vk::make_api_version(0, 1, 3, 0));
        let mut layer_names: Vec<&CStr> = vec![];
        {
            #[cfg(feature = "validation_layers")]
            layer_names.push(unsafe {
                CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0")
            });
        }
        let layer_names_raw: Vec<*const c_char> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();
        let mut extension_manager = ExtensionManager::new();

        for name in ash_window::enumerate_required_extensions(&window).unwrap() {
            unsafe {
                extension_manager.add_extension(*name);
            }
        }
        unsafe {
            extension_manager.add_extension(DebugUtils::name().as_ptr());
        }
        let base_extensions = unsafe {
            [CStr::from_bytes_with_nul_unchecked(
                b"VK_EXT_descriptor_indexing\0",
            )]
        };
        for name in base_extensions {
            unsafe { extension_manager.add_extension(name.as_ptr()) }
        }
        for name in layer_names_raw.iter() {
            let name_cstr = unsafe { CStr::from_ptr(*name) };
            let name_str = name_cstr.to_str().unwrap();
            println!("enabled layer: {}", name_str);
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(extension_manager.extensions());

        let entry = Entry::linked();
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
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

        let instance =
            unsafe { entry.create_instance(&create_info, None) }.expect("failed to create");
        let debug_utils_loader = DebugUtils::new(&entry, &instance);
        let debug_callback = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .expect("failed to create debug callback")
        };
        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, &window, None)
                .expect("failed to create render surface")
        };
        let surface_loader = Surface::new(&entry, &instance);
        let p_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Error getting physical devices")
        };
        let (p_device, queue_family_index) = p_devices
            .iter()
            .map(|dev| unsafe {
                instance
                    .get_physical_device_queue_family_properties(*dev)
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        let supports_graphic_and_surface = info
                            .queue_flags
                            .contains(vk::QueueFlags::GRAPHICS)
                            && surface_loader
                                .get_physical_device_surface_support(*dev, index as u32, surface)
                                .expect("failed to get device_support");
                        if supports_graphic_and_surface {
                            Some((*dev, index))
                        } else {
                            None
                        }
                    })
            })
            .find_map(|i| i)
            .expect("no sutible device");
        let device_extension_names_raw = [Swapchain::name().as_ptr()];
        let queue_family_index = queue_family_index as u32;
        let features = vk::PhysicalDeviceFeatures::builder().shader_clip_distance(true);
        let priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);
        let device = unsafe {
            instance
                .create_device(p_device, &device_create_info, None)
                .expect("failed to create device")
        };
        let present_queue = unsafe { device.get_device_queue(queue_family_index as u32, 0) };
        let surface_format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(p_device, surface)
                .unwrap()[0]
        };
        let surface_capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(p_device, surface)
                .unwrap()
        };
        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        };
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
        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(p_device, surface)
                .unwrap()
        };
        let present_mode = present_modes
            .iter()
            .find(|mode| **mode == vk::PresentModeKHR::MAILBOX)
            .cloned()
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let swapchain_loader = Swapchain::new(&instance, &device);
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
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
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        };
        let pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);
        let pool = unsafe { device.create_command_pool(&pool_create_info, None).unwrap() };
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(2)
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY);
        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_alloc_info)
                .unwrap()
        };
        let setup_command_buffer = command_buffers[0];
        let draw_command_buffer = command_buffers[1];
        let present_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let present_image_views = present_images
            .iter()
            .map(|image| {
                let create_view_info = vk::ImageViewCreateInfo::builder()
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
                unsafe { device.create_image_view(&create_view_info, None).unwrap() }
            })
            .collect::<Vec<_>>();
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(p_device) };
        let depth_image_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D16_UNORM)
            .extent(surface_resolution.into())
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let depth_image = unsafe { device.create_image(&depth_image_create_info, None).unwrap() };
        let depth_image_req = unsafe { device.get_image_memory_requirements(depth_image) };
        let depth_image_memory_index = find_memory_type_index(
            &depth_image_req,
            &device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .expect("failed to find suitable memory index for depth index");
        let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(depth_image_req.size)
            .memory_type_index(depth_image_memory_index);
        let depth_image_memory = unsafe {
            device
                .allocate_memory(&depth_image_allocate_info, None)
                .unwrap()
        };

        unsafe {
            device
                .bind_image_memory(depth_image, depth_image_memory, 0)
                .expect("failed to bind depth buffer")
        };
        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let draw_commands_reuse_fence = unsafe {
            device
                .create_fence(&fence_create_info, None)
                .expect("fence create failed")
        };
        let setup_commands_reuse_fence = unsafe {
            device
                .create_fence(&fence_create_info, None)
                .expect("fence create failed")
        };
        unsafe {
            record_submit_commandbuffer(
                &device,
                setup_command_buffer,
                setup_commands_reuse_fence,
                present_queue,
                &[],
                &[],
                &[],
                |device, setup_command_buffer| {
                    println!("submitting img transfer");
                    let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                        .image(depth_image)
                        .dst_access_mask(
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        )
                        .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .layer_count(1)
                                .level_count(1)
                                .build(),
                        )
                        .build();

                    device.cmd_pipeline_barrier(
                        setup_command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barriers],
                    )
                },
            );
        }
        let depth_image_view_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(depth_image)
            .format(depth_image_create_info.format)
            .view_type(vk::ImageViewType::TYPE_2D);
        let depth_image_view = unsafe {
            device
                .create_image_view(&depth_image_view_info, None)
                .expect("failed to create depth image view")
        };
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let present_complete_semaphore = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .expect("failed to create semaphore")
        };
        let rendering_complete_semaphore = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .expect("failed to create semaphore")
        };
        Base {
            event_loop: RefCell::new(event_loop),
            entry,
            instance,
            device,
            queue_family_index,
            p_device,
            device_memory_properties,
            window,
            surface_loader,
            surface_format,
            present_queue,
            surface_resolution,
            swapchain_loader,
            swapchain,
            present_images,
            present_image_views,
            pool,
            draw_command_buffer,
            setup_command_buffer,
            depth_image,
            depth_image_view,
            present_complete_semaphore,
            rendering_complete_semaphore,
            draw_commands_reuse_fence,
            setup_commands_reuse_fence,
            surface,
            debug_callback,
            debug_utils_loader,
            depth_image_memory,

            window_width,
            window_height,
            extension_manager,
        }
    }
    pub fn num_swapchain_images(&self) -> usize {
        self.present_image_views.len()
    }
    pub fn render_loop<F: Fn(usize)>(&self, f: F) {
        let mut frame_counter = 0;
        self.event_loop
            .borrow_mut()
            .run_return(|event, _target, controll_flow| {
                *controll_flow = ControlFlow::Poll;
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        println!("exit");
                        *controll_flow = ControlFlow::Exit
                    }
                    Event::WindowEvent {
                        event: WindowEvent::KeyboardInput { input, .. },
                        ..
                    } => {
                        println!("keyboard input, input: {:?}", input);
                    }
                    Event::MainEventsCleared => {
                        f(frame_counter);
                        self.window.request_redraw();

                        frame_counter += 1;
                    }
                    _ => {}
                };
            });
    }
}

impl Drop for Base {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device
                .destroy_semaphore(self.present_complete_semaphore, None);
            self.device
                .destroy_semaphore(self.rendering_complete_semaphore, None);
            self.device
                .destroy_fence(self.draw_commands_reuse_fence, None);
            self.device
                .destroy_fence(self.setup_commands_reuse_fence, None);
            self.device.free_memory(self.depth_image_memory, None);
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            for &image_view in self.present_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.device.destroy_command_pool(self.pool, None);
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_callback, None);
            self.instance.destroy_instance(None);
        }
    }
}
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );
    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        let bt = backtrace::Backtrace::new();
        println!("{:?}", bt);
        panic!()
    }

    vk::FALSE
}
