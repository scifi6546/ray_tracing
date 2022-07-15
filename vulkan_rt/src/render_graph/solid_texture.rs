use super::{PassBase, VulkanOutput, VulkanOutputType, VulkanPass};
use crate::prelude::*;

use ash::vk;

/// outputs a solid texture to future render passes
pub struct SolidTexturePass {
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
}
impl SolidTexturePass {
    pub fn new(base: &PassBase) -> Self {
        let descriptor_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
        }];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&descriptor_sizes)
            .max_sets(100);
        let descriptor_pool = unsafe {
            base.base
                .device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("failed to get descriptor pool")
        };
        let desc_layout_bindings = unsafe {
            [vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()]
        };
        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&desc_layout_bindings);
        let descriptor_set_layout = unsafe {
            base.base
                .device
                .create_descriptor_set_layout(&descriptor_info, None)
                .expect("failed to create descriptor set layout")
        };
        Self {
            descriptor_pool,
            descriptor_set_layout,
        }
    }
}
impl VulkanPass for SolidTexturePass {
    fn get_dependencies(&self) -> Vec<VulkanOutputType> {
        vec![]
    }

    fn get_output(&self) -> Vec<VulkanOutputType> {
        vec![VulkanOutputType::Empty]
    }

    fn process(&mut self, base: &PassBase, input: Vec<&VulkanOutput>) -> Vec<VulkanOutput> {
        println!("processing solid texture pass, todo: make do stuff");
        vec![VulkanOutput::Empty]
    }

    fn free(&mut self, base: &PassBase) {
        unsafe {
            base.base
                .device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            base.base
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
