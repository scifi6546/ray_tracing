mod graph;
mod output_pass;
mod solid_texture;

use super::{prelude::*, record_submit_commandbuffer, Base, GraphicsApp};
use ash::{util::read_spv, vk};
use gpu_allocator::{vulkan::*, AllocatorDebugSettings};
use graph::{RenderGraph, RenderPass};
use output_pass::OutputPass;
use std::{
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
}

pub struct RenderPassApp {
    graph: RenderGraph<Box<dyn VulkanPass>>,
    allocator: Arc<Mutex<Allocator>>,
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
        let pass_base = PassBase {
            base,
            allocator: allocator.clone(),
        };
        let solid_texture: Box<dyn VulkanPass> =
            Box::new(solid_texture::SolidTexturePass::new(&pass_base));
        let (solid_pass_id, solid_pass_output) = graph.insert_pass(solid_texture, Vec::new());
        let pass: Box<dyn VulkanPass> = Box::new(OutputPass::new(&pass_base));
        //    let pass: Box<dyn VulkanPass> = Box::new(BasicVulkanPass::new(&pass_base));
        graph.insert_output_pass(pass, solid_pass_output);

        Self { graph, allocator }
    }
}
impl GraphicsApp for RenderPassApp {
    fn run_frame(&mut self, base: Rc<Base>, frame_number: u32) {
        let pass_base = PassBase {
            base,
            allocator: self.allocator.clone(),
        };
        self.graph.run_graph(&pass_base);
    }

    fn process_event(&mut self, elapsed_time: Duration) {}

    fn handle_event(&mut self, base: Rc<Base>, event: &Event<()>) {}

    fn free_resources(self, base: Rc<Base>) {
        {
            let pass_base = PassBase {
                base,
                allocator: self.allocator.clone(),
            };
            self.graph.free_passes(&pass_base);
        }

        drop(self.allocator);
    }
}
