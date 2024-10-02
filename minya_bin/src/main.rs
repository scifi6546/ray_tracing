mod graphics_context;
mod gui;
mod messages;

use cgmath::{InnerSpace, Vector2, Vector3};
use graphics_context::GraphicsContext;
use gui::GuiCtx;
use lib_minya::{
    prelude::*,
    ray_tracer::{ray_tracer_info::RayTracerInfo, CurrentShader, LogMessage, RayTracer},
    Image,
};
use log::info;
use messages::GuiPushMessage;
use miniquad::{
    conf, Bindings, BufferLayout, BufferSource, BufferType, BufferUsage, EventHandler, Pipeline,
    PipelineParams, RenderingBackend, ShaderSource, UniformsSource, VertexAttribute, VertexFormat,
};
use std::{
    sync::mpsc::{channel, Receiver},
    thread,
    thread::JoinHandle,
};

pub fn vec_near_zero(v: Vector3<f32>) -> bool {
    v.dot(v) < 1e-8
}
fn make_miniquad_texture(image: &Image, context: &mut dyn RenderingBackend) -> miniquad::TextureId {
    context.new_texture_from_rgba8(
        image.width() as u16,
        image.height() as u16,
        image.buffer_rgba8(),
    )
}

#[repr(C)]
struct Vertex {
    pos: Vector2<f32>,
    uv: Vector2<f32>,
}

struct Handler {
    context: GraphicsContext,
    ctx: Box<dyn RenderingBackend>,
    image_receiver: Receiver<Image>,
    join_handle: JoinHandle<()>,
    gui: GuiCtx,
}
impl Handler {
    pub fn new() -> Self {
        let mut ctx: Box<dyn RenderingBackend> = miniquad::window::new_rendering_backend();
        let mut context = GraphicsContext::new(ctx.as_mut());

        let (message_sender, message_reciever) = channel();
        let (image_sender, image_receiver) = channel();
        let ray_tracer = RayTracer::new(None, None, None);
        let info = ray_tracer.get_info();
        let join_handle = thread::spawn(move || {
            let mut par_img = ParallelImage::new_black(1000, 1000);

            let mut receiver = ray_tracer
                .clone()
                .threaded_render(ParallelImage::new_black(1000, 1000));
            loop {
                if let Ok(message) = message_reciever.try_recv() {
                    match message {
                        GuiPushMessage::LoadScenario(scenario) => {
                            info!("loading scenario: {}", scenario);
                            receiver.load_scenario(scenario);
                            par_img = ParallelImage::new_black(1000, 1000);
                        }
                        GuiPushMessage::SaveFile(path) => receiver.save_file(path),
                        GuiPushMessage::SetShader(s) => {
                            receiver.set_shader(s);
                        }
                        GuiPushMessage::SetEntityData => todo!("set entity data"),
                        GuiPushMessage::SetCameraData((key, value)) => {
                            receiver.set_camera_data(key, value);
                        }
                    }
                }

                if let Some(img) = receiver.receive() {
                    par_img = img;
                }
                let mut process_image = par_img.clone();

                ray_tracer.post_process(&mut process_image);

                image_sender
                    .send(Image::from_parallel_image(&process_image))
                    .expect("channel failed");
            }
        });

        let gui = GuiCtx::new(ctx.as_mut(), &info, message_sender);
        Self {
            context,
            ctx,
            image_receiver,
            join_handle,
            gui,
        }
    }
}
impl EventHandler for Handler {
    fn update(&mut self) {
        if self.join_handle.is_finished() {
            println!("FINISHED!!!");
            miniquad::window::order_quit();
        }
        if let Ok(img) = self.image_receiver.try_recv() {
            self.context.update_image(self.ctx.as_mut(), img);
        }

        self.gui.update(self.ctx.as_mut());
    }

    fn draw(&mut self) {
        self.context.draw_plane(self.ctx.as_mut());
        self.gui.draw(self.ctx.as_mut());
        self.ctx.commit_frame();
    }
    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.gui.mouse_motion_event(self.ctx.as_mut(), x, y);
    }
    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.gui.mouse_wheel_event(self.ctx.as_mut(), x, y);
    }
    fn mouse_button_down_event(&mut self, mb: miniquad::MouseButton, x: f32, y: f32) {
        self.gui
            .mouse_button_down_event(self.ctx.as_mut(), mb, x, y);
    }

    fn mouse_button_up_event(&mut self, mb: miniquad::MouseButton, x: f32, y: f32) {
        self.gui.mouse_button_up_event(self.ctx.as_mut(), mb, x, y);
    }

    fn char_event(&mut self, character: char, keymods: miniquad::KeyMods, repeat: bool) {
        self.gui
            .char_event(self.ctx.as_mut(), character, keymods, repeat);
    }

    fn key_down_event(
        &mut self,

        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        repeat: bool,
    ) {
        self.gui
            .key_down_event(self.ctx.as_mut(), keycode, keymods, repeat);
    }
    fn key_up_event(&mut self, keycode: miniquad::KeyCode, keymods: miniquad::KeyMods) {
        self.gui.key_up_event(self.ctx.as_mut(), keycode, keymods);
    }
}

fn main() {
    miniquad::start(
        conf::Conf {
            window_width: 1000,
            window_height: 1000,
            ..Default::default()
        },
        || Box::new(Handler::new()),
    );
}
mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;
    uniform vec2 offset;
    varying lowp vec2 texcoord;
    void main() {
        gl_Position = vec4(pos + offset, 0, 1);
        texcoord = uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;
    uniform sampler2D tex;
    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![miniquad::UniformDesc::new("offset", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}
