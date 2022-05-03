mod ray_tracer;

use cgmath::Vector2;
use miniquad::{
    conf, date, Bindings, Buffer, BufferLayout, BufferType, Context, EventHandler, Pipeline,
    Shader, Texture, UserData, VertexAttribute, VertexFormat,
};
use std::{
    ops::{Add, AddAssign, Div, Mul},
    sync::mpsc::Receiver,
};

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
#[derive(Clone)]
pub struct RgbImage {
    buffer: Vec<RgbColor>,
    width: u32,
    height: u32,
}
impl RgbImage {
    pub fn new_black(width: u32, height: u32) -> Self {
        let buffer = (0..(width as usize * height as usize))
            .map(|_| RgbColor {
                red: 0.0,
                blue: 0.0,
                green: 0.0,
            })
            .collect();

        RgbImage {
            buffer,
            width,
            height,
        }
    }
    pub fn add_xy(&mut self, x: u32, y: u32, color: RgbColor) {
        self.buffer[y as usize * self.width as usize + x as usize] += color;
    }
}
impl Div<f32> for RgbImage {
    type Output = RgbImage;

    fn div(mut self, rhs: f32) -> Self::Output {
        Self {
            buffer: self.buffer.drain(..).map(|c| c / rhs).collect(),
            width: self.width,
            height: self.height,
        }
    }
}
#[repr(C)]
struct Vertex {
    pos: Vector2<f32>,
    uv: Vector2<f32>,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}
impl RgbColor {
    pub fn to_rgba_u8(&self) -> [u8; 4] {
        let r = (clamp(self.red.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let g = (clamp(self.green.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let b = (clamp(self.blue.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        [r, g, b, 0xff]
    }
}
impl Mul<f32> for RgbColor {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}
impl Mul<RgbColor> for f32 {
    type Output = RgbColor;
    fn mul(self, rhs: RgbColor) -> Self::Output {
        rhs * self
    }
}
impl Div<f32> for RgbColor {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            red: self.red / rhs,
            green: self.green / rhs,
            blue: self.blue / rhs,
        }
    }
}
impl Add for RgbColor {
    type Output = RgbColor;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}
impl AddAssign for RgbColor {
    fn add_assign(&mut self, rhs: Self) {
        self.red += rhs.red;
        self.green += rhs.green;
        self.blue += rhs.blue;
    }
}
impl std::iter::Sum for RgbColor {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(
            RgbColor {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            },
            |acc, x| acc + x,
        )
    }
}
struct Handler {
    pipeline: Pipeline,
    bindings: Bindings,
    image_channel: Receiver<Image>,
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
            |x, y| [((y as f32 / 100.0) * 255.0) as u8, 0, 200, 0xff],
            100,
            100,
        );
        let texture = img.make_texture(ctx);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![texture],
        };
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta());
        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
        );
        let image_channel = ray_tracer::RayTracer::new();
        Self {
            pipeline,
            bindings,
            image_channel,
        }
    }
}
impl EventHandler for Handler {
    fn update(&mut self, ctx: &mut Context) {
        if let Ok(img) = self.image_channel.try_recv() {
            let tex = img.make_texture(ctx);
            self.bindings.images = vec![tex];
            println!("recieved image");
        }
    }

    fn draw(&mut self, ctx: &mut Context) {
        let t = date::now();

        ctx.begin_default_pass(Default::default());

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);

        ctx.apply_uniforms(&shader::Uniforms { offset: (0.0, 0.0) });
        ctx.draw(0, 6, 1);

        ctx.end_render_pass();

        ctx.commit_frame();
    }
}
pub struct Image {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
}
impl Image {
    pub fn from_rgb_image(image: &RgbImage) -> Self {
        let buffer = image.buffer.iter().flat_map(|c| c.to_rgba_u8()).collect();
        Self {
            buffer,
            width: image.width,
            height: image.height,
        }
    }
    pub fn new(buffer: Vec<u8>, width: u32, height: u32) -> Self {
        assert_eq!(buffer.len(), width as usize * height as usize * 4);
        Self {
            buffer,
            width,
            height,
        }
    }
    pub fn from_fn(ctor: fn(u32, u32) -> [u8; 4], width: u32, height: u32) -> Self {
        Self {
            buffer: (0..width)
                .flat_map(|y| (0..height).flat_map(move |x| ctor(x, y)))
                .collect(),
            width,
            height,
        }
    }
    pub fn set_xy(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        let offset = (self.width * y + x) * 4;
        for i in 0..4 {
            self.buffer[(offset + i) as usize] = pixel[i as usize];
        }
    }
    pub fn set_xy_color(&mut self, x: u32, y: u32, pixel: RgbColor) {
        self.set_xy(x, y, pixel.to_rgba_u8());
    }
    pub fn make_texture(&self, ctx: &mut Context) -> Texture {
        Texture::from_rgba8(ctx, self.width as u16, self.height as u16, &self.buffer)
    }
}
fn main() {
    miniquad::start(
        conf::Conf {
            window_width: 800,
            window_height: 800,
            ..Default::default()
        },
        |mut ctx| UserData::owning(Handler::new(&mut ctx), ctx),
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
            images: &["tex"],
            uniforms: UniformBlockLayout {
                uniforms: &[("offset", UniformType::Float2)],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub offset: (f32, f32),
    }
}
