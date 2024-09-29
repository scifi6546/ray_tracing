use crate::{gui::GuiCtx, make_miniquad_texture, shader, Vertex};
use cgmath::Vector2;
use lib_minya::Image;
use miniquad::{
    Bindings, BufferLayout, BufferSource, BufferType, BufferUsage, Pipeline, PipelineParams,
    RenderingBackend, ShaderSource, UniformsSource, VertexAttribute, VertexFormat,
};

pub struct GraphicsContext {
    pipeline: Pipeline,
    bindings: Bindings,
}
impl GraphicsContext {
    pub fn new(ctx: &mut dyn RenderingBackend) -> Self {
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

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let texture = {
            const IMAGE_X: usize = 100;
            const IMAGE_Y: usize = 100;
            let data = [0xffu8; IMAGE_X * IMAGE_Y * 4];
            ctx.new_texture_from_rgba8(IMAGE_X as u16, IMAGE_Y as u16, &data)
        };

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![texture],
        };

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: shader::FRAGMENT,
                },
                shader::meta(),
            )
            .expect("failed to create shader for frontend");

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        );
        Self { pipeline, bindings }
    }

    pub fn update_image(&mut self, ctx: &mut dyn RenderingBackend, image: Image) {
        let tex = make_miniquad_texture(&image, ctx);

        self.bindings.images = vec![tex];
    }

    pub fn draw_plane(&mut self, ctx: &mut dyn RenderingBackend) {
        ctx.begin_default_pass(Default::default());
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);

        ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
            offset: (0.0, 0.0),
        }));
        ctx.draw(0, 6, 1);

        ctx.end_render_pass();
    }
}
