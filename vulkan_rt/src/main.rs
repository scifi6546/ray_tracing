mod base;
mod extension_manager;
mod hello_many_meshes;
mod hello_push;
mod hello_scenelib;
mod hello_texture;
mod hello_triangle;
pub mod prelude;
mod render_graph;

use ash::{vk, Device};
use base::Base;
use std::{
    rc::Rc,
    time::{Duration, Instant},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    platform::run_return::EventLoopExtRunReturn,
};

fn find_memory_type_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
/// Submits command buffer, wait mash len must be the same as wait semaphores len
pub unsafe fn record_submit_commandbuffer<F: FnOnce(&Device, vk::CommandBuffer)>(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    command_buffer_reuse_fence: vk::Fence,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphore: &[vk::Semaphore],
    f: F,
) {
    assert_eq!(wait_mask.len(), wait_semaphores.len());
    device
        .wait_for_fences(&[command_buffer_reuse_fence], true, u64::MAX)
        .expect("failed to wait for fence");
    device
        .reset_fences(&[command_buffer_reuse_fence])
        .expect("failed to reset fence");
    device
        .reset_command_buffer(
            command_buffer,
            vk::CommandBufferResetFlags::RELEASE_RESOURCES,
        )
        .expect("failed to reset command buffer");
    let command_buffer_begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    device
        .begin_command_buffer(command_buffer, &command_buffer_begin_info)
        .expect("failed to begin command buffer");
    f(device, command_buffer);
    device
        .end_command_buffer(command_buffer)
        .expect("failed to end command buffer");
    let command_buffers = vec![command_buffer];

    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(wait_semaphores)
        .wait_dst_stage_mask(wait_mask)
        .command_buffers(&command_buffers)
        .signal_semaphores(signal_semaphore)
        .build();
    device
        .queue_submit(submit_queue, &[submit_info], command_buffer_reuse_fence)
        .expect("failed to submit queue")
}

pub trait GraphicsApp {
    fn run_frame(&mut self, base: Rc<Base>, frame_number: u32);
    fn update_delta_time(&mut self, elapsed_time: Duration);
    fn handle_event(&mut self, base: Rc<Base>, event: &winit::event::Event<()>);
    fn free_resources(self, base: Rc<Base>);
}
struct GraphicsAppRunner<App: GraphicsApp> {
    base: Rc<Base>,
    app: App,
    last_update_time: Instant,
}
impl<App: GraphicsApp> GraphicsAppRunner<App> {
    pub fn drain_base(self) -> Base {
        self.app.free_resources(self.base.clone());
        let unwrap_res = Rc::try_unwrap(self.base);
        match unwrap_res {
            Ok(r) => r,
            Err(_) => panic!("failed to unwrap base rc"),
        }
    }

    pub fn run(&mut self) {
        let mut frame_counter = 0;
        self.base
            .event_loop
            .borrow_mut()
            .run_return(|event, _target, controll_flow| {
                *controll_flow = ControlFlow::Poll;
                self.app.handle_event(self.base.clone(), &event);
                match event {
                    Event::NewEvents(_) => {
                        let now = Instant::now();
                        self.app.update_delta_time(now - self.last_update_time);
                        self.last_update_time = Instant::now();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        println!("exit");
                        *controll_flow = ControlFlow::Exit
                    }

                    Event::MainEventsCleared => {
                        self.app.run_frame(self.base.clone(), frame_counter);
                        self.base.window.request_redraw();

                        frame_counter += 1;
                    }
                    _ => {}
                };
            });
    }
}
fn main() {
    let window_width = 1000;
    let window_height = 1000;
    let base = Base::new(window_width, window_height);
    println!("hello rendergraph");
    let base = {
        let base = Rc::new(base);
        let mut runner = GraphicsAppRunner {
            app: render_graph::RenderPassApp::new(base.clone()),
            base,
            last_update_time: Instant::now(),
        };
        runner.run();
        runner.drain_base()
    };
    println!("hello scenelib");
    let base = {
        let mut runner = GraphicsAppRunner {
            app: hello_scenelib::App::new(&base),
            base: Rc::new(base),
            last_update_time: Instant::now(),
        };
        runner.run();
        runner.drain_base()
    };

    println!("hello many meshes");
    hello_many_meshes::run(&base);
    println!("hello push constant");
    hello_push::run(&base);
    println!("hello texture");
    hello_texture::run(&base);
    println!("hello triangle");
    hello_triangle::run(&base);
}
