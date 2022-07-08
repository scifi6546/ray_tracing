use super::PassBase;
use crate::prelude::*;
use ash::vk;
/// Describes renderpass that render a framebuffer to screen
pub struct OutputPass {
    imgui_context: imgui::Context,
    imgui_renderer: imgui_rs_vulkan_renderer::Renderer,
    imgui_platform: imgui_winit_support::WinitPlatform,
    render_plane: RenderModel,
    framebuffers: Vec<vk::Framebuffer>,
    renderpass: vk::RenderPass,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layouts: [vk::DescriptorSetLayout; 1],
    fragment_shader_module: vk::ShaderModule,
    vertex_shader_module: vk::ShaderModule,
    graphics_pipeline: vk::Pipeline,
    mesh_list: Vec<RenderModel>,
    pipeline_layout: vk::PipelineLayout,
    scissors: [vk::Rect2D; 1],
    viewports: [vk::Viewport; 1],
}
