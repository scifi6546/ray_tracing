use super::Base;
use ash::vk;
use std::rc::Rc;

pub struct MeshDescriptors {
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layouts: [vk::DescriptorSetLayout; 1],
}
impl MeshDescriptors {
    pub fn new(base: Rc<Base>) -> Self {
        let descriptor_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
        }];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&descriptor_sizes)
            .max_sets(1000);
        let descriptor_pool = unsafe {
            base.device
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
        let descriptor_set_layouts = unsafe {
            [base
                .device
                .create_descriptor_set_layout(&descriptor_info, None)
                .expect("failed to create descriptor set layout")]
        };

        Self {
            descriptor_pool,
            descriptor_set_layouts,
        }
    }
    pub fn free(&self, base: Rc<Base>) {
        unsafe {
            base.device.device_wait_idle().expect("failed to wait idle");

            base.device
                .destroy_descriptor_set_layout(self.descriptor_set_layouts[0].clone(), None);
            base.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
