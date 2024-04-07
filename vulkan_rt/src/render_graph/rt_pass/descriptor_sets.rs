use super::Base;
use ash::vk;
pub struct RayTracingDescriptorSets {
    pool: Option<vk::DescriptorPool>,
    layout: Option<vk::DescriptorSetLayout>,
}
impl RayTracingDescriptorSets {
    pub fn new(base: &Base) -> Self {
        let descriptor_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 1,
            },
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&descriptor_sizes)
            .max_sets(600);
        let pool = unsafe {
            base.device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("failed to get descriptor pool")
        };
        let desc_layout_bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                .build(),
        ];
        let descriptor_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&desc_layout_bindings);
        let layout = unsafe {
            base.device
                .create_descriptor_set_layout(&descriptor_info, None)
                .expect("failed to create descriptor set layout")
        };
        Self {
            pool: Some(pool),
            layout: Some(layout),
        }
    }
    pub fn get_layout(&self) -> vk::DescriptorSetLayout {
        self.layout.expect("descriptor set already freed")
    }
    pub fn free(&mut self, base: &Base) {
        unsafe {
            base.device.device_wait_idle().expect("failed to wait idle");
            base.device.destroy_descriptor_set_layout(
                self.layout.take().expect("device already freed"),
                None,
            );
            base.device
                .destroy_descriptor_pool(self.pool.take().expect("device already freed"), None)
        }
    }
}
impl Drop for RayTracingDescriptorSets {
    fn drop(&mut self) {
        if self.pool.is_some() || self.layout.is_some() {
            panic!("descriptor sets dropped before free called")
        }
    }
}
