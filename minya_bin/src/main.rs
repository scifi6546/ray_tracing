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
    time::Instant,
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
        let mut ray_tracer = RayTracer::new(None, None, None);
        let (message_sender, message_reciever) = channel();
        let (image_sender, image_reciever) = channel();
        let info = ray_tracer.get_info();
        thread::spawn(move || {
            let total_time = Instant::now();
            let mut num_samples = 1usize;
            let mut par_img = ParallelImage::new_black(1000, 1000);
            loop {
                if let Ok(message) = message_reciever.try_recv() {
                    match message {
                        Message::LoadScenario(scenario) => {
                            info!("loading scenario: {}", scenario);
                            par_img = ParallelImage::new_black(1000, 1000);
                            num_samples = 1;
                            ray_tracer.load_scenario(scenario);
                        }
                        Message::SaveFile(path) => par_img.save_image(path, num_samples),
                        Message::SetShader(s) => {
                            par_img = ParallelImage::new_black(1000, 1000);
                            ray_tracer.set_shader(s);
                        }
                    }
                }

                ray_tracer.trace_image(&mut par_img);
                let mut process_image = par_img.clone() / num_samples as f32;
                ray_tracer.post_process(&mut process_image);

                image_sender
                    .send(Image::from_parallel_image(&process_image))
                    .expect("channel failed");
                let average_time_s = total_time.elapsed().as_secs_f32() / (num_samples) as f32;
                info!(
                    "frame: {}, average time per frame: {} (s)",
                    num_samples, average_time_s
                );
                num_samples += 1;
            }
        });
        Self {
            pipeline,
            bindings,
            image_reciever,
            gui: GuiCtx::new(ctx, &info, message_sender),
        }
    }
}
impl EventHandler for Handler {
    fn update(&mut self, ctx: &mut Context) {
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
            window_width: 800,
            window_height: 800,
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
