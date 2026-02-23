use app::App;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};
mod app;
mod utils;
#[repr(C)]
#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
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
