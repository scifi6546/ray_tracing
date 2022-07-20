mod graph;
mod mesh_descriptors;
mod output_pass;
mod solid_texture;

use super::{prelude::*, record_submit_commandbuffer, Base, GraphicsApp};
use crate::render_graph::mesh_descriptors::MeshDescriptors;
use ash::{util::read_spv, vk};
use gpu_allocator::{vulkan::*, AllocatorDebugSettings};
use graph::{RenderGraph, RenderPass};
use output_pass::OutputPass;
use std::{
    cell::RefCell,
    ffi::CStr,
    io::Cursor,
    mem::size_of,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};
use winit::event::Event;

/// Possible outputs of renderpass, todo: garbage collector
pub enum VulkanOutput {
    /// view that a pass draws to
    Framebuffer {
        descriptor_set: vk::DescriptorSet,
    },
    Empty,
}
#[derive(PartialEq, Eq, Clone)]
pub enum VulkanOutputType {
    FrameBuffer,
    /// Temporary pass used to mark dependency
    Empty,
}
pub trait VulkanPass {
    fn handle_event(&mut self, _base: &PassBase, _event: &winit::event::Event<()>) {}
    fn prepare_render(&mut self, _base: &PassBase) {}
    fn get_dependencies(&self) -> Vec<VulkanOutputType>;
    fn get_output(&self) -> Vec<VulkanOutputType>;
    fn process(&mut self, base: &PassBase, input: Vec<&VulkanOutput>) -> Vec<VulkanOutput>;
    /// frees resources, will only be called once
    fn free(&mut self, base: &PassBase);
}
impl RenderPass for Box<dyn VulkanPass> {
    type Base = PassBase;
    type RenderPassOutputMarker = VulkanOutputType;
    type RenderPassOutput = VulkanOutput;

    fn get_dependencies(&self) -> Vec<Self::RenderPassOutputMarker> {
        VulkanPass::get_dependencies(self.as_ref())
    }

    fn get_output(&self) -> Vec<Self::RenderPassOutputMarker> {
        VulkanPass::get_output(self.as_ref())
    }

    fn process(
        &mut self,
        base: &Self::Base,
        input: Vec<&Self::RenderPassOutput>,
    ) -> Vec<Self::RenderPassOutput> {
        VulkanPass::process(self.as_mut(), base, input)
    }

    fn free(mut self, base: &Self::Base) {
        VulkanPass::free(self.as_mut(), base)
    }
}
pub struct PassBase {
    pub base: Rc<Base>,
    pub allocator: Arc<Mutex<Allocator>>,
    pub scene_state: Rc<RefCell<SceneState>>,
    pub engine_entities: Rc<RefCell<EngineEntities>>,
}
pub struct SceneState {
    pub imgui_context: imgui::Context,
    pub mesh_descriptors: MeshDescriptors,
}
impl SceneState {
    pub fn new(base: Rc<Base>, allocator: Arc<Mutex<Allocator>>) -> (Self, EngineEntities) {
        let mut imgui_context = imgui::Context::create();
        let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
        let hidipi_factor = imgui_platform.hidpi_factor();
        imgui_platform.attach_window(
            imgui_context.io_mut(),
            &base.window,
            imgui_winit_support::HiDpiMode::Rounded,
        );
        imgui_context.io_mut().font_global_scale = (1.0 / hidipi_factor as f32);
        let mesh_descriptors = MeshDescriptors::new(base.clone());
        let engine_entities = EngineEntities::new(
            base.as_ref(),
            allocator,
            &mesh_descriptors.descriptor_pool,
            &mesh_descriptors.descriptor_set_layouts,
        );
        (
            Self {
                imgui_context,

                mesh_descriptors,
            },
            engine_entities,
        )
    }
}
pub struct RenderPassApp {
    graph: RenderGraph<Box<dyn VulkanPass>>,
    allocator: Arc<Mutex<Allocator>>,
    scene_state: Rc<RefCell<SceneState>>,
    engine_entities: Rc<RefCell<EngineEntities>>,
}
impl RenderPassApp {
    pub fn new(base: Rc<Base>) -> Self {
        let mut graph = RenderGraph::new();
        let allocator = Arc::new(Mutex::new(
            Allocator::new(&AllocatorCreateDesc {
                instance: base.instance.clone(),
                device: base.device.clone(),
                physical_device: base.p_device.clone(),
                debug_settings: AllocatorDebugSettings::default(),
                buffer_device_address: false,
            })
            .expect("created allocator"),
        ));
        let (scene_state, engine_entities) = SceneState::new(base.clone(), allocator.clone());
        let scene_state = Rc::new(RefCell::new(scene_state));
        let engine_entities = Rc::new(RefCell::new(engine_entities));
        let mut pass_base = PassBase {
            base,
            allocator: allocator.clone(),
            scene_state: scene_state.clone(),
            engine_entities: engine_entities.clone(),
        };
        let solid_texture: Box<dyn VulkanPass> =
            Box::new(solid_texture::SolidTexturePass::new(&pass_base));
        let (solid_pass_id, solid_pass_output) = graph.insert_pass(solid_texture, Vec::new());
        let pass: Box<dyn VulkanPass> = Box::new(OutputPass::new(&mut pass_base));

        graph.insert_output_pass(pass, solid_pass_output);

        Self {
            graph,
            allocator,
            scene_state,
            engine_entities,
        }
    }
}
impl GraphicsApp for RenderPassApp {
    fn run_frame(&mut self, base: Rc<Base>, frame_number: u32) {
        {
            let pass_base = PassBase {
                base: base.clone(),
                allocator: self.allocator.clone(),
                scene_state: self.scene_state.clone(),
                engine_entities: self.engine_entities.clone(),
            };
            for pass in self.graph.iter_mut() {
                pass.prepare_render(&pass_base);
            }
        }
        {
            let mut scene_state = self.scene_state.as_ref().borrow_mut();
            let frame = scene_state.imgui_context.frame();
            frame.button("Test!!!");
        }
        let pass_base = PassBase {
            base,
            allocator: self.allocator.clone(),
            scene_state: self.scene_state.clone(),
            engine_entities: self.engine_entities.clone(),
        };
        self.graph.run_graph(&pass_base);
    }

    fn update_delta_time(&mut self, elapsed_time: Duration) {
        let mut scene_state = self.scene_state.as_ref().borrow_mut();
        scene_state
            .imgui_context
            .io_mut()
            .update_delta_time(elapsed_time)
    }

    fn handle_event(&mut self, base: Rc<Base>, event: &Event<()>) {
        let pass_base = PassBase {
            base,
            allocator: self.allocator.clone(),
            scene_state: self.scene_state.clone(),
            engine_entities: self.engine_entities.clone(),
        };
        for pass in self.graph.iter_mut() {
            pass.handle_event(&pass_base, event)
        }
    }

    fn free_resources(self, base: Rc<Base>) {
        {
            let pass_base = PassBase {
                base: base.clone(),
                allocator: self.allocator.clone(),
                scene_state: self.scene_state.clone(),
                engine_entities: self.engine_entities.clone(),
            };
            self.graph.free_passes(&pass_base);
        }
        {
            unsafe {
                self.engine_entities
                    .as_ref()
                    .borrow_mut()
                    .free_resources(base.as_ref(), self.allocator.clone());
            }

            let mut scene_state = self.scene_state.as_ref().borrow_mut();

            scene_state.mesh_descriptors.free(base.clone())
        }
        drop(self.allocator);
    }
}
