use super::{descriptor_sets::RayTracingDescriptorSets, Base, RayTracingState};
use crate::prelude::load_bytes;

use ash::{prelude::*, util::read_spv, vk};

use std::{ffi::CStr, io::Cursor, mem::size_of, path::Path};

const RAY_SIZE: usize = size_of::<f32>() * 4;
pub struct RayTracingPipeline {
    pipeline: Option<vk::Pipeline>,
    pipeline_layout: Option<vk::PipelineLayout>,
    descriptor_set_layout: RayTracingDescriptorSets,
    raygen_module: Option<vk::ShaderModule>,
    closest_hit_module: Option<vk::ShaderModule>,
    any_miss_module: Option<vk::ShaderModule>,
}

impl RayTracingPipeline {
    pub fn new(base: &Base, ray_tracing_state: &RayTracingState) -> Self {
        fn load_module(path: impl AsRef<Path>, base: &Base) -> vk::ShaderModule {
            let mut spv_file = Cursor::new(load_bytes(path));
            let shader_code = read_spv(&mut spv_file).expect("failed to read raygen spv file");
            let shader_info = vk::ShaderModuleCreateInfo::builder().code(&shader_code);
            unsafe {
                base.device
                    .create_shader_module(&shader_info, None)
                    .expect("vertex shader compile info")
            }
        }
        let descriptor_sets = RayTracingDescriptorSets::new(base);
        let raygen_module = load_module(
            "./vulkan_rt/shaders/bin/raytracing/raytracing.rgen.spv",
            base,
        );
        let closest_hit_module = load_module(
            "./vulkan_rt/shaders/bin/raytracing/raytracing.rchit.spv",
            base,
        );
        let any_miss_module = load_module(
            "./vulkan_rt/shaders/bin/raytracing/raytracing.rmiss.spv",
            base,
        );
        let shader_entry_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };
        let raygen_shader_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::RAYGEN_KHR)
            .module(raygen_module)
            .name(shader_entry_name)
            .build();
        let hit_shader_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::CLOSEST_HIT_KHR)
            .module(closest_hit_module)
            .name(shader_entry_name)
            .build();
        let miss_shader_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::MISS_KHR)
            .module(any_miss_module)
            .name(shader_entry_name)
            .build();
        let default_shader_group = vk::RayTracingShaderGroupCreateInfoKHR::builder()
            .any_hit_shader(vk::SHADER_UNUSED_KHR)
            .closest_hit_shader(vk::SHADER_UNUSED_KHR)
            .intersection_shader(vk::SHADER_UNUSED_KHR);

        let mut raygen_shader_group = default_shader_group.clone();
        raygen_shader_group.general_shader = 0;

        let mut closest_hit_shader_group = default_shader_group.clone();
        closest_hit_shader_group.closest_hit_shader = 1;
        closest_hit_shader_group.ty = vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP;

        let mut miss_shader_group = default_shader_group.clone();
        miss_shader_group.general_shader = 2;

        let push_constant_range = [vk::PushConstantRange::builder()
            .size(RAY_SIZE as u32)
            .stage_flags(
                vk::ShaderStageFlags::RAYGEN_KHR
                    | vk::ShaderStageFlags::CLOSEST_HIT_KHR
                    | vk::ShaderStageFlags::MISS_KHR,
            )
            .build()];
        let descriptor_set_layouts = [descriptor_sets.get_layout()];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&push_constant_range)
            .set_layouts(&descriptor_set_layouts);
        let pipeline_layout = unsafe {
            base.device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
        }
        .expect("failed to create layout");
        let info = [vk::RayTracingPipelineCreateInfoKHR::builder()
            .flags(vk::PipelineCreateFlags::empty())
            .stages(&[
                raygen_shader_create_info,
                hit_shader_create_info,
                miss_shader_create_info,
            ])
            .groups(&[
                raygen_shader_group,
                closest_hit_shader_group,
                miss_shader_group,
            ])
            .layout(pipeline_layout)
            .max_pipeline_ray_recursion_depth(3)
            .build()];
        unsafe {
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
                pipeline_layout: Some(pipeline_layout),
                descriptor_set_layout: descriptor_sets,
                raygen_module: Some(raygen_module),
                closest_hit_module: Some(closest_hit_module),
                any_miss_module: Some(any_miss_module),
            }
        }
    }
    pub fn free(&mut self, base: &Base) {
        unsafe {
            base.device.destroy_pipeline(
                self.pipeline
                    .take()
                    .expect("raytracing pipeline already freed"),
                None,
            );
            base.device.destroy_pipeline_layout(
                self.pipeline_layout
                    .take()
                    .expect("raytracing pipeline already freed"),
                None,
            );
            base.device.destroy_shader_module(
                self.raygen_module
                    .take()
                    .expect("raytracing pipeline already freed"),
                None,
            );
            base.device.destroy_shader_module(
                self.closest_hit_module
                    .take()
                    .expect("raytracing pipeline already freed"),
                None,
            );
            base.device.destroy_shader_module(
                self.any_miss_module
                    .take()
                    .expect("raytracing pipeline already freed"),
                None,
            );
            self.descriptor_set_layout.free(base);
        }
    }
    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline.unwrap()
    }
    pub fn pipeline_bind_point(&self) -> vk::PipelineBindPoint {
        vk::PipelineBindPoint::RAY_TRACING_KHR
    }
    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout.get_layout()
    }
    pub fn pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout.unwrap()
    }
}
impl Drop for RayTracingPipeline {
    fn drop(&mut self) {
        if self.pipeline.is_some() {
            panic!("raytracing pipeline dropped before free called")
        }
    }
}
