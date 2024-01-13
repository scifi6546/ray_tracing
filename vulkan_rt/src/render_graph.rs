mod diffuse_pass;
mod graph;
mod mesh_descriptors;
mod output_pass;
mod rt_pass;
mod solid_texture;

use super::{prelude::*, Base, GraphicsApp};
use crate::prelude::voxel::{Voxel, VoxelChunk};
use crate::render_graph::mesh_descriptors::MeshDescriptors;
use ash::{extensions::khr::AccelerationStructure, vk};
use diffuse_pass::DiffusePass;
use gpu_allocator::{vulkan::*, AllocationSizes, AllocatorDebugSettings};
use graph::{RenderGraph, RenderPass};
use output_pass::OutputPass;
use rt_pass::RtPass;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};
use winit::event::Event;
pub fn get_semaphores(outputs: &[&VulkanOutput]) -> Vec<vk::Semaphore> {
    outputs
        .iter()
        .map(|o| match o {
            &VulkanOutput::Framebuffer {
                write_semaphore, ..
            } => Some(write_semaphore.clone()),
            &VulkanOutput::Empty => None,
        })
        .filter_map(|s| s)
        .filter_map(|s| s)
        .collect()
}
/// Possible outputs of renderpass
pub enum VulkanOutput {
    /// view that a pass draws to
    Framebuffer {
        descriptor_set: vk::DescriptorSet,
        /// signaled when safe to write to, if present must be consumed
        write_semaphore: Option<vk::Semaphore>,
    },
    #[allow(dead_code)]
    Empty,
}
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VulkanOutputType {
    FrameBuffer,
    /// Temporary pass used to mark dependency
    Empty,
}
pub trait VulkanPass {
    fn handle_event(&mut self, _base: &PassBase, _event: &winit::event::Event<()>) {}
    fn update_delta_time(&mut self, _elapsed_time: Duration) {}
    ///optional function, called before render graph on each frame. To be used if graph layer needs additional processing
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
pub struct RayTracingState {
    pub acceleration_structure: AccelerationStructure,
}
impl RayTracingState {
    pub fn new(base: Rc<Base>) -> Self {
        let acceleration_structure = AccelerationStructure::new(&base.instance, &base.device);
        Self {
            acceleration_structure,
        }
    }
}
#[derive(Clone)]
pub struct PassBase {
    pub base: Rc<Base>,
    pub raytracing_state: Rc<RayTracingState>,
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
        imgui_context.set_ini_filename(None);

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
    raytracing_state: Rc<RayTracingState>,
    engine_entities: Rc<RefCell<EngineEntities>>,
    voxels: VoxelChunk,
}
impl RenderPassApp {
    pub fn new(base: Rc<Base>) -> Self {
        let voxels = VoxelChunk::new(|x, y, z| Voxel::Solid);
        let mut graph = RenderGraph::new();
        let allocator = Arc::new(Mutex::new(
            Allocator::new(&AllocatorCreateDesc {
                instance: base.instance.clone(),
                device: base.device.clone(),
                physical_device: base.p_device.clone(),
                debug_settings: AllocatorDebugSettings::default(),
                buffer_device_address: true,
                allocation_sizes: AllocationSizes::default(),
            })
            .expect("created allocator"),
        ));
        let (scene_state, engine_entities) = SceneState::new(base.clone(), allocator.clone());
        let scene_state = Rc::new(RefCell::new(scene_state));
        let engine_entities = Rc::new(RefCell::new(engine_entities));
        let raytracing_state = Rc::new(RayTracingState::new(base.clone()));
        let mut pass_base = PassBase {
            base,
            allocator: allocator.clone(),
            scene_state: scene_state.clone(),
            engine_entities: engine_entities.clone(),
            raytracing_state: raytracing_state.clone(),
        };
        let solid_texture: Box<dyn VulkanPass> =
            Box::new(solid_texture::SolidTexturePass::new(&pass_base));

        let (_solid_pass_id, solid_pass_output) = graph.insert_pass(solid_texture, Vec::new());
        let pass: Box<dyn VulkanPass> = Box::new(OutputPass::new(&mut pass_base, 2));
        let diffuse_pass: Box<dyn VulkanPass> = Box::new(DiffusePass::new(pass_base.clone()));
        let (_pass_id, rt_output) = graph.insert_pass(
            Box::new(RtPass::new(&pass_base).expect("failed to build renderpass")),
            Vec::new(),
        );
        let (_diffuse_pass_id, diffuse_pass_deps) = graph.insert_pass(diffuse_pass, vec![]);
        graph.insert_output_pass(
            pass,
            vec![solid_pass_output[0].clone(), diffuse_pass_deps[0].clone()],
        );

        Self {
            graph,
            allocator,
            scene_state,
            engine_entities,
            raytracing_state,
            voxels,
        }
    }
}
impl GraphicsApp for RenderPassApp {
    fn run_frame(&mut self, base: Rc<Base>, _frame_number: u32) {
        {
            let pass_base = PassBase {
                base: base.clone(),
                allocator: self.allocator.clone(),
                scene_state: self.scene_state.clone(),
                engine_entities: self.engine_entities.clone(),
                raytracing_state: self.raytracing_state.clone(),
            };
            for pass in self.graph.iter_mut() {
                pass.prepare_render(&pass_base);
            }
        }

        let pass_base = PassBase {
            base,
            allocator: self.allocator.clone(),
            scene_state: self.scene_state.clone(),
            engine_entities: self.engine_entities.clone(),
            raytracing_state: self.raytracing_state.clone(),
        };
        self.graph.run_graph(&pass_base);
    }

    fn update_delta_time(&mut self, elapsed_time: Duration) {
        for layer in self.graph.iter_mut() {
            layer.update_delta_time(elapsed_time)
        }

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
            raytracing_state: self.raytracing_state.clone(),
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
                raytracing_state: self.raytracing_state.clone(),
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

            let scene_state = self.scene_state.as_ref().borrow_mut();

            scene_state.mesh_descriptors.free(base.clone())
        }
        drop(self.allocator);
    }
}
