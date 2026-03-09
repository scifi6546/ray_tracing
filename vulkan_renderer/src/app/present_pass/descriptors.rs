use ash::{Device, vk};
pub struct PresentDescriptors {
    pub pool: vk::DescriptorPool,
    pub layout: vk::DescriptorSetLayout,
}
impl PresentDescriptors {
    const MAX_SETS: u32 = 100;
    pub fn new(device: &Device) -> Self {
        unsafe {
            let descriptor_sizes = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLER,
                    descriptor_count: 1,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLED_IMAGE,
                    descriptor_count: 1,
                },
            ];
            let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&descriptor_sizes)
                .max_sets(Self::MAX_SETS);
            let pool = device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("failed to create");
            let layout_bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .descriptor_count(1),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .descriptor_count(1),
            ];
            let layout_info =
                vk::DescriptorSetLayoutCreateInfo::default().bindings(&layout_bindings);
            let layout = device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("failed to create descriptor set layout");
            Self { pool, layout }
        }
    }
    pub fn free(&self, device: &Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool, None);
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
