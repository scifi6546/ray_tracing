mod command_buffer;
mod present_pass;
mod voxel_renderer;
mod world;
use command_buffer::{DrawCommandBuffer, SetupCommandBuffer};
use present_pass::{
    ForeignTextureInput, PresentDescriptors, PresentModel, PresentModelInfo, PresentPass,
    PresentRectangle, PresentTexture,
};
use voxel_renderer::VoxelPass;
use world::World;

use super::utils::vulkan_debug_callback;
use ash::{Device, Entry, Instance, ext::debug_utils, khr, vk};

use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::iter;
use winit::{
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};
/// Contents are ordered in how they should be freed
/// Things there is a v2 of:
/// ImageMemoryBarriar
pub struct App {
    world: World,
    voxel_output_surfaces: Vec<PresentModel>,
    voxel_pass: Option<VoxelPass>,
    present_pass: Option<PresentPass>,
    present_models: Vec<PresentModel>,

    descriptors: PresentDescriptors,
    allocator: Option<Allocator>,
    setup_command_buffer: SetupCommandBuffer,
    draw_commands_reuse_fence: [vk::Fence; Self::MAX_FRAME_LATENCY],

    present_semaphores: [vk::Semaphore; Self::MAX_FRAME_LATENCY],

    command_pool: vk::CommandPool,
    swapchain: vk::SwapchainKHR,
    device: Device,
    surface: vk::SurfaceKHR,
    surface_instance: khr::surface::Instance,
    debug_call_back: vk::DebugUtilsMessengerEXT,
    debug_utils_loader: debug_utils::Instance,
    instance: Instance,

    frame_index: usize,

    draw_command_buffers: Vec<DrawCommandBuffer>,
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
    /// The maximum number of frames we allow to be in flight at any given time
    const MAX_FRAME_LATENCY: usize = 3;
    pub fn new(event_loop: &ActiveEventLoop, window_width: u32, window_height: u32) -> Self {
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
                .api_version(vk::make_api_version(0, 1, 4, 0));
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
            let synchronization = c"VK_KHR_synchronization2";
            let device_extension_names_raw =
                [khr::swapchain::NAME.as_ptr(), synchronization.as_ptr()];
            let priorities = [1.0];
            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };
            {
                let extensions = instance
                    .enumerate_device_extension_properties(physical_device)
                    .expect("failed to get physical device");
                for e in extensions {
                    let extension_name = e
                        .extension_name_as_c_str()
                        .expect("failed to get extension name")
                        .to_str()
                        .expect("failed to convert to string");
                    println!("supported extension: {}", extension_name);
                }
            }

            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);
            let device = instance
                .create_device(physical_device, &device_create_info, None)
                .expect("failed to create device");
            let present_queue = device.get_device_queue(queue_family_index, 0);
            let all_formats = surface_instance
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("failed to get formats");
            println!("availible surface formats: {:#?}", all_formats);
            let surface_format = all_formats[0];
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
                .command_buffer_count(2 + Self::MAX_FRAME_LATENCY as u32)
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY);
            let command_buffers = device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("failed to create command buffers");

            let mut setup_command_buffer = SetupCommandBuffer::new(command_buffers[0]);
            let app_setup_command_buffer = command_buffers[1];

            let draw_command_buffers = command_buffers[2..]
                .iter()
                .copied()
                .map(|command_buffer| DrawCommandBuffer::new(&device, command_buffer))
                .collect();
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_semaphores = std::array::from_fn(|_| {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("failed to create present complete semaphore")
            });

            let fence_create_info =
                vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            let draw_commands_reuse_fence = std::array::from_fn(|_| {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("failed to create fence")
            });
            let mut allocator = Allocator::new(&AllocatorCreateDesc {
                instance: instance.clone(),
                device: device.clone(),
                physical_device,
                debug_settings: Default::default(),
                buffer_device_address: false,
                allocation_sizes: Default::default(),
            })
            .expect("failed to create allocator");
            let descriptors = PresentDescriptors::new(&device);
            let mut present_vk_info = PresentModelInfo {
                device: &device,
                allocator: &mut allocator,
                setup_command_buffer: &mut setup_command_buffer,
                present_queue: &present_queue,
                descriptors: &descriptors,
            };

            let triangle_model = PresentModel::new_rectangle_with_buffer(
                PresentRectangle {
                    min_x: -1.,
                    min_y: -1.,
                    max_x: 1.,
                    max_y: 1.,
                    z_index: 0.6,
                },
                include_bytes!("../temp_assets/rocket.png"),
                &mut present_vk_info,
            );

            let ui_element = PresentModel::new_rectangle_with_buffer(
                PresentRectangle {
                    min_x: -1.,
                    min_y: 0.7,
                    max_x: 0.3,
                    max_y: 1.0,
                    z_index: 0.1,
                },
                include_bytes!("../temp_assets/UI.png"),
                &mut present_vk_info,
            );

            let present_pass = PresentPass::new(
                &device,
                &mut setup_command_buffer,
                &instance,
                swapchain,
                present_queue,
                &mut allocator,
                &descriptors,
                surface_resolution,
                surface_format,
            );
            let voxel_pass = VoxelPass::new(
                &device,
                &mut allocator,
                &mut setup_command_buffer,
                desired_image_count,
                present_queue,
                surface_resolution,
            );
            let voxel_output_surfaces = voxel_pass
                .textures()
                .into_iter()
                .map(|data| {
                    let mut vulkan_info = PresentModelInfo {
                        device: &device,
                        allocator: &mut allocator,

                        setup_command_buffer: &mut setup_command_buffer,
                        present_queue: &present_queue,
                        descriptors: &descriptors,
                    };
                    let texture = PresentTexture::from_foreign_data(&vulkan_info, data);
                    PresentModel::new_rectangle(
                        PresentRectangle {
                            min_x: -1.,
                            max_x: 1.,
                            min_y: -1.,
                            max_y: 1.,
                            z_index: 0.2,
                        },
                        texture,
                        &mut vulkan_info,
                    )
                })
                .collect::<Vec<_>>();
            let world = World::default();
            Self {
                world,
                voxel_output_surfaces,
                voxel_pass: Some(voxel_pass),
                frame_index: 0,
                descriptors,
                present_models: vec![triangle_model, ui_element],
                present_pass: Some(present_pass),
                allocator: Some(allocator),
                draw_commands_reuse_fence,
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
    pub fn request_redraw(&mut self) {
        self.world.update();
        let current_frame_index = self.frame_index % Self::MAX_FRAME_LATENCY;

        unsafe {
            let present_complete_semaphore = self.present_semaphores[current_frame_index];
            let draw_command_buffer = &mut self.draw_command_buffers[current_frame_index];
            draw_command_buffer.start_command_buffer(&self.device);
            let (present_index, _) = self
                .swapchain_device
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    present_complete_semaphore,
                    vk::Fence::null(),
                )
                .expect("failed to acquire next image");

            self.voxel_pass
                .as_mut()
                .expect("voxel pass already freed")
                .draw(
                    &self.world,
                    &self.device,
                    draw_command_buffer,
                    current_frame_index,
                );
            {
                let voxel_model = iter::once(&self.voxel_output_surfaces[current_frame_index]);

                let models = self.present_models.iter().chain(voxel_model);

                self.present_pass
                    .as_ref()
                    .expect("should not be freed")
                    .draw(
                        &self.device,
                        models,
                        draw_command_buffer,
                        current_frame_index,
                    );
            }

            draw_command_buffer.submit_command_buffer(
                &self.device,
                self.present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[present_complete_semaphore],
            );
            let wait_semaphores = [draw_command_buffer.rendering_complete_semaphore()];
            let swapchains = [self.swapchain];
            let present_indices = [present_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&wait_semaphores)
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
            for model in self.voxel_output_surfaces.drain(..) {
                model.free(
                    &self.device,
                    self.allocator.as_mut().expect("allocator already freed"),
                );
            }
            self.voxel_pass
                .take()
                .expect("voxel pass already freed")
                .free(
                    &self.device,
                    self.allocator.as_mut().expect("allocator already freed"),
                );
            self.present_pass
                .take()
                .expect("should not already be freed")
                .free(
                    &self.device,
                    self.allocator.as_mut().expect("already dropped"),
                );
            for model in self.present_models.drain(..) {
                model.free(
                    &self.device,
                    self.allocator.as_mut().expect("allocator already freed"),
                );
            }

            self.descriptors.free(&self.device);
            for fence in self.draw_commands_reuse_fence {
                self.device.destroy_fence(fence, None);
            }
            for buffer in self.draw_command_buffers.drain(..) {
                buffer.free(&self.device);
            }

            for semaphore in self.present_semaphores {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.setup_command_buffer.free(&self.device);

            self.device.destroy_command_pool(self.command_pool, None);
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
            drop(self.allocator.take().expect("is already freed"));
            self.device.destroy_device(None);
            self.surface_instance.destroy_surface(self.surface, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}
