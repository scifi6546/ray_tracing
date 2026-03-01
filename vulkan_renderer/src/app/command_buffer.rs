use std::u64;

use ash::{Device, vk};
pub struct SetupCommandBuffer {
    pub command_buffer: vk::CommandBuffer,
    /// fence that indicates that the buffer is in use. wait for it to finish in order to reuse the command buffer
    pub wait_fence: Option<vk::Fence>,
}
impl SetupCommandBuffer {
    pub fn new(command_buffer: vk::CommandBuffer) -> Self {
        Self {
            command_buffer,
            wait_fence: None,
        }
    }
    pub fn record_command_buffer<F: FnOnce(&Device, vk::CommandBuffer)>(
        &mut self,
        device: &Device,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
        f: F,
    ) {
        unsafe {
            // waiting for command buffer to become available
            if let Some(fence) = self.wait_fence {
                device
                    .wait_for_fences(&[fence], true, u64::MAX)
                    .expect("failed to wait for fence")
            }
            device
                .reset_command_buffer(
                    self.command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("reset command buffer failed");
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)
                .expect("failed to begin command buffer");
            f(device, self.command_buffer);
            device
                .end_command_buffer(self.command_buffer)
                .expect("failed to create command buffer");
            let command_buffers = vec![self.command_buffer];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_mask)
                .command_buffers(&command_buffers)
                .signal_semaphores(signal_semaphores);
            // resetting fence if it already exists.
            // this is safe as we have already waited for it in the beginning of the function
            if let Some(fence) = self.wait_fence {
                device
                    .reset_fences(&[fence])
                    .expect("failed to wait for fence");
            } else {
                self.wait_fence = Some(
                    device
                        .create_fence(&vk::FenceCreateInfo::default(), None)
                        .expect("failed to create fence"),
                );
            }

            device
                .queue_submit(submit_queue, &[submit_info], self.wait_fence.unwrap())
                .expect("failed to submit queue");
        }
    }
    pub fn free(&self, device: &Device) {
        unsafe {
            device.device_wait_idle().expect("failed to wait idle");
            if let Some(fence) = self.wait_fence {
                device.destroy_fence(fence, None);
            }

            device
                .reset_command_buffer(
                    self.command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("failed to reset buffer");
        }
    }
}
