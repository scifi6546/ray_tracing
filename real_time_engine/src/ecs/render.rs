use super::super::{mesh, texture, voxel, Camera, CameraUniform};
use crate::mesh::RuntimeMesh;
use crate::voxel::VoxelGrid;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use cgmath::Vector3;
use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let winit_state = WinitState::new(app);
        let render_state = RenderState::new(&winit_state);
        let render_state = pollster::block_on(render_state);
        let mut schedule = Schedule::new();
        schedule.configure_sets(
            (
                RenderSet::PreRender,
                RenderSet::Render
                    .after(RenderSet::PreRender)
                    .before(RenderSet::PostRender),
                RenderSet::PostRender.after(RenderSet::Render),
            )
                .chain(),
        );
        let mut startup_schedule = Schedule::new();
        startup_schedule.configure_set(RenderSet::OnStartup);
        app.outer_schedule_label = Box::new(StartupScheduleLabel);
        app.default_schedule_label = Box::new(StartupScheduleLabel);
        app.set_runner(move |app| winit_state.runner(app))
            .insert_resource(render_state)
            .add_schedule(RenderScheduleLabel, schedule)
            .add_schedule(StartupScheduleLabel, startup_schedule)
            .add_systems((
                test_pre_render.in_set(RenderSet::PreRender),
                test_render.in_set(RenderSet::Render),
                test_post_render.in_set(RenderSet::PostRender),
            ))
            .add_system(render_system.in_set(RenderSet::Render))
            .add_system(build_voxel_mesh.in_set(RenderSet::PreRender))
            .add_system(insert_voxel_grid.in_set(RenderSet::OnStartup));
    }
}
fn test_pre_render() {
    //println!("0. pre render!")
}
fn test_render() {
    //println!("1. in render!")
}
fn test_post_render() {
    //println!("2. post render!")
}
struct WinitState {
    window: Window,
}
impl WinitState {
    pub fn new(app: &mut App) -> Self {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("APP!")
            .build(&event_loop)
            .unwrap();
        app.insert_non_send_resource(event_loop);
        Self { window }
    }
    pub fn runner(&self, mut app: App) {
        for _ in 0..10 {
            app.update()
        }
        let mut event_loop = app
            .world
            .remove_non_send_resource::<EventLoop<()>>()
            .expect("failed to get event loop");

        event_loop.run_return(|event, target, control_flow| match event {
            Event::RedrawRequested(window_id) => {
                if window_id == self.window.id() {
                    app.update();
                    //println!("todo render");
                }
            }
            Event::MainEventsCleared => {
                //println!("todo render");
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        });
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
}
#[derive(Resource)]
struct RenderState {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    diffuse_bind_group: wgpu::BindGroup,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    render_pipeline: wgpu::RenderPipeline,
    cube: mesh::RuntimeMesh,
}
impl RenderState {
    pub async fn new(winit_state: &WinitState) -> Self {
        let size = winit_state.window().inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(winit_state.window()) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        for form in surface_caps.formats.iter() {
            println!("format: {:?}", form)
        }
        for present_mode in surface_caps.present_modes.iter() {
            println!("present mode: {:#?}", present_mode);
        }
        let present_mode = surface_caps
            .present_modes
            .iter()
            .copied()
            .filter(|mode| match mode {
                wgpu::PresentMode::Mailbox => true,
                _ => false,
            })
            .next()
            .unwrap_or(surface_caps.present_modes[0].clone());
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let diffuse_bytes = include_bytes!("../../test_texture/tree.png");

        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png");

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/shader.wgsl").into()),
        });
        let camera = Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 40.0, 20.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        };
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",           // 1.
                buffers: &[mesh::Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });
        let cube = mesh::Mesh::cube();
        let cube = mesh::RuntimeMesh::from_mesh(&cube, &device);
        Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            diffuse_bind_group,
            camera_bind_group,
            depth_texture,
            render_pipeline,
            cube,
        }
    }
    fn render(&mut self, query: Query<&RuntimeMesh, ()>) -> Result<(), wgpu::SurfaceError> {
        let current_texture = self.surface.get_current_texture()?;
        let view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);
            {
                /*
                render_pass.set_vertex_buffer(0, self.cube.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.cube.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                render_pass.draw_indexed(0..self.cube.num_indices as u32, 0, 0..1);

                 */
            }
            println!("start render?");
            for model in query.iter() {
                println!("rendering model");
                render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                render_pass.draw_indexed(0..model.num_indices as u32, 0, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        current_texture.present();
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum RenderSet {
    /// Ran on startup
    OnStartup,
    /// Steps taken before rendering
    PreRender,
    /// Setps taken during rendering
    Render,
    /// After render
    PostRender,
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ScheduleLabel)]
pub struct RenderScheduleLabel;
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ScheduleLabel)]
pub struct StartupScheduleLabel;
fn render_system(mut render_state: ResMut<RenderState>, query: Query<&RuntimeMesh, ()>) {
    render_state
        .render(query)
        .expect("failed to render, todo: handle error");
}
fn build_voxel_mesh(
    mut commands: Commands,
    render_state: Res<RenderState>,
    mut query: Query<(Entity, &ECSVoxelGrid), Without<RuntimeMesh>>,
) {
    println!("start build mesh");
    for (entity, grid) in query.iter_mut() {
        println!("updating mesh?");
        let grid = &grid.0;
        let voxel_mesh = grid.build_mesh();
        let device = &render_state.device;
        let run_time = mesh::RuntimeMesh::from_mesh(&voxel_mesh, device);
        commands.entity(entity).insert(run_time);
    }
}
#[derive(Component)]
struct ECSVoxelGrid(VoxelGrid<bool>);
fn insert_voxel_grid(mut commands: Commands) {
    println!("insert voxel grid");
    let voxel_grid = voxel::VoxelGrid::new(Vector3::new(3, 3, 10), true);

    commands.spawn(ECSVoxelGrid(voxel_grid));
}
