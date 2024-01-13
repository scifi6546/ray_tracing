mod gui;

use cgmath::{InnerSpace, Vector2, Vector3};
use gui::GuiCtx;
use lib_minya::{
    prelude::*,
    ray_tracer::{CurrentShader, LogMessage, RayTracer, RayTracerInfo},
    Image,
};
use miniquad::{
    conf, Bindings, Buffer, BufferLayout, BufferType, Context, EventHandler, Pipeline, Shader,
    VertexAttribute, VertexFormat,
};

use log::info;
use std::{
    sync::mpsc::{channel, Receiver},
    thread,
    thread::JoinHandle,
};
pub enum Message {
    LoadScenario(String),
    SaveFile(std::path::PathBuf),
    SetShader(CurrentShader),
}
pub fn vec_near_zero(v: Vector3<f32>) -> bool {
    v.dot(v) < 1e-8
}

#[repr(C)]
struct Vertex {
    pos: Vector2<f32>,
    uv: Vector2<f32>,
}

struct Handler {
    pipeline: Pipeline,
    bindings: Bindings,
    image_reciever: Receiver<Image>,
    join_handle: JoinHandle<()>,
    gui: GuiCtx,
}
impl Handler {
    pub fn new(ctx: &mut Context) -> Self {
        let vertices: [Vertex; 4] = [
            Vertex {
                pos: Vector2 { x: -1.0, y: -1.0 },
                uv: Vector2 { x: 0., y: 0. },
            },
            Vertex {
                pos: Vector2 { x: 1.0, y: -1.0 },
                uv: Vector2 { x: 1., y: 0. },
            },
            Vertex {
                pos: Vector2 { x: 1.0, y: 1.0 },
                uv: Vector2 { x: 1., y: 1. },
            },
            Vertex {
                pos: Vector2 { x: -1.0, y: 1.0 },
                uv: Vector2 { x: 0., y: 1. },
            },
        ];
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        let img = Image::from_fn(
            |_x, y| [((y as f32 / 100.0) * 255.0) as u8, 0, 200, 0xff],
            100,
            100,
        );
        let texture = img.make_texture(ctx);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![texture],
        };
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta())
            .expect("failed to compile");
        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );
        let ray_tracer = RayTracer::new(None, None, None);
        let (message_sender, message_reciever) = channel();
        let (image_sender, image_reciever) = channel();
        let info = ray_tracer.get_info();
        let join_handle = thread::spawn(move || {
            let mut par_img = ParallelImage::new_black(1000, 1000);
            let mut receiver = ray_tracer
                .clone()
                .threaded_render(ParallelImage::new_black(1000, 1000));
            loop {
                if let Ok(message) = message_reciever.try_recv() {
                    match message {
                        Message::LoadScenario(scenario) => {
                            info!("loading scenario: {}", scenario);
                            receiver.load_scenario(scenario);
                            par_img = ParallelImage::new_black(1000, 1000);
                        }
                        Message::SaveFile(path) => receiver.save_file(path),
                        Message::SetShader(s) => {
                            receiver.set_shader(s);
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

        Self {
            pipeline,
            bindings,
            image_reciever,
            join_handle,
            gui: GuiCtx::new(ctx, &info, message_sender),
        }
    }
}
impl EventHandler for Handler {
    fn update(&mut self, ctx: &mut Context) {
        if self.join_handle.is_finished() {
            println!("FINISHED!!!");
        }
        if let Ok(img) = self.image_reciever.try_recv() {
            let tex = img.make_texture(ctx);
            self.bindings.images = vec![tex];
        }

        self.gui.update(ctx);
    }

    fn draw(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(Default::default());
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);

        ctx.apply_uniforms(&shader::Uniforms { offset: (0.0, 0.0) });
        ctx.draw(0, 6, 1);

        ctx.end_render_pass();
        self.gui.draw(ctx);

        ctx.commit_frame();
    }
    fn mouse_motion_event(&mut self, ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.gui.mouse_motion_event(ctx, x, y);
    }
    fn mouse_wheel_event(&mut self, ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.gui.mouse_wheel_event(ctx, x, y);
    }
    fn mouse_button_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.gui.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.gui.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        ctx: &mut miniquad::Context,
        character: char,
        keymods: miniquad::KeyMods,
        repeat: bool,
    ) {
        self.gui.char_event(ctx, character, keymods, repeat);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        repeat: bool,
    ) {
        self.gui.key_down_event(ctx, keycode, keymods, repeat);
    }
    fn key_up_event(
        &mut self,
        ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
    ) {
        self.gui.key_up_event(ctx, keycode, keymods);
    }
}

fn main() {
    miniquad::start(
        conf::Conf {
            window_width: 1000,
            window_height: 1000,
            ..Default::default()
        },
        |mut ctx| Box::new(Handler::new(&mut ctx)),
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
