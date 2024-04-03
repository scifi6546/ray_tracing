use super::{Base, RayTracingState};
use ash::{prelude::*, vk};
pub struct RayTracingPipeline {
    pipeline: Option<vk::Pipeline>,
}
impl RayTracingPipeline {
    pub fn new(base: &Base, ray_tracing_state: &RayTracingState) -> Self {
        unsafe {
            let info = [vk::RayTracingPipelineCreateInfoKHR::builder()
                .flags(vk::PipelineCreateFlags::empty())
                .stages(todo!("shader stages"))
                .groups(todo!("groups"))
                .library_info(todo!("library"))
                .library_interface(todo!("library interface"))
                .dynamic_state(todo!("dynamic state"))
                .layout(todo!("layout"))
                .base_pipeline_handle(todo!("base pipeline"))
                .base_pipeline_index(todo!("base pipeline index"))
                .build()];
            let pipeline = ray_tracing_state
                .raytracing_pipeline
                .create_ray_tracing_pipelines(
                    vk::DeferredOperationKHR::null(),
                    vk::PipelineCache::null(),
                    &info,
                    None,
                )
                .expect("failed to create")[0];
            Self {
                pipeline: Some(pipeline),
            }
        }
    }
    pub fn free(&mut self, base: &Base) {
        unsafe {
            base.device.destroy_pipeline(
                self.pipeline
                    .take()
                    .expect("raytracing pipelie already freed"),
                None,
            )
        }
    }
}
