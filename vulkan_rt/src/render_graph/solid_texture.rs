use super::{PassBase, VulkanOutput, VulkanOutputType, VulkanPass};
use crate::prelude::*;

use std::ops::DerefMut;

use ash::vk;

/// outputs a solid texture to future render passes
pub struct SolidTexturePass {
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    texture: Option<RenderTexture>,
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
        let desc_layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build()];
        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&desc_layout_bindings);
        let descriptor_set_layout = unsafe {
            base.base
                .device
                .create_descriptor_set_layout(&descriptor_info, None)
                .expect("failed to create descriptor set layout")
        };
        let texture = RenderTexture::new(
            &image::RgbaImage::from_fn(100, 100, |x, y| {
                let r = (x % 255) as u8;
                let g = (y % 255) as u8;
                let b = r + g;
                image::Rgba([r, g, b, 255])
            }),
            base.base.as_ref(),
            base.allocator
                .lock()
                .expect("failed to get allocator")
                .deref_mut(),
            &descriptor_pool,
            &[descriptor_set_layout],
        );
        Self {
            descriptor_pool,
            descriptor_set_layout,
            texture: Some(texture),
        }
    }
}
impl VulkanPass for SolidTexturePass {
    fn get_dependencies(&self) -> Vec<VulkanOutputType> {
        vec![]
    }

    fn get_output(&self) -> Vec<VulkanOutputType> {
        vec![VulkanOutputType::FrameBuffer]
    }

    fn process(&mut self, _base: &PassBase, _input: Vec<&VulkanOutput>) -> Vec<VulkanOutput> {
        vec![VulkanOutput::Framebuffer {
            descriptor_set: self
                .texture
                .as_ref()
                .expect("already freed")
                .descriptor_set
                .clone(),
            write_semaphore: None,
        }]
    }

    fn free(&mut self, base: &PassBase) {
        let tex = self.texture.take().expect("pass already freed");

        unsafe {
            tex.free_resources(
                base.base.as_ref(),
                &mut base.allocator.lock().expect("failed to get allocator"),
            );
            base.base
                .device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            base.base
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
