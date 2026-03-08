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
    pub fn wait_for_command_completion(&self, device: &Device) {
        if let Some(fence) = self.wait_fence {
            unsafe {
                device
                    .wait_for_fences(&[fence], true, u64::MAX)
                    .expect("failed to wait for fence")
            }
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
#[derive(PartialEq, Eq, Debug)]
enum DrawCommandBufferState {
    Initial,
    Recording,
    Submitted,
}
pub struct DrawCommandBuffer {
    pub command_buffer: vk::CommandBuffer,
    pub command_buffer_reuse_fence: Option<vk::Fence>,
    rendering_complete_semaphore: vk::Semaphore,
    state: DrawCommandBufferState,
}
impl DrawCommandBuffer {
    pub fn new(device: &Device, command_buffer: vk::CommandBuffer) -> Self {
        let rendering_complete_semaphore = unsafe {
            let create_info = vk::SemaphoreCreateInfo::default();
            device.create_semaphore(&create_info, None)
        }
        .expect("failed to create semaphore");
        Self {
            command_buffer,
            command_buffer_reuse_fence: None,
            rendering_complete_semaphore,
            state: DrawCommandBufferState::Initial,
        }
    }
    /// starts the command buffer and waits for the completion of the previous iteration of the command buffer
    /// command buffer **MUST** be started before the next image is acquired
    pub fn start_command_buffer(&mut self, device: &Device) {
        self.state = match self.state {
            DrawCommandBufferState::Initial => DrawCommandBufferState::Recording,
            DrawCommandBufferState::Recording => {
                panic!("invalid command buffer state must be initial")
            }
            DrawCommandBufferState::Submitted => DrawCommandBufferState::Recording,
        };
        unsafe {
            if let Some(fence) = self.command_buffer_reuse_fence {
                device
                    .wait_for_fences(&[fence], true, u64::MAX)
                    .expect("failed to wait for command buffer reuse fence");
                device
                    .reset_fences(&[fence])
                    .expect("failed to reset fence")
            } else {
                let fence_info = vk::FenceCreateInfo::default();

                self.command_buffer_reuse_fence = Some(
                    device
                        .create_fence(&fence_info, None)
                        .expect("failed to create command buffer reuse fence"),
                );
            }

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)
                .expect("failed to begin command buffer");
        }
    }
    pub fn record_command_buffer<F: FnOnce(&Device, vk::CommandBuffer)>(
        &self,
        device: &Device,
        run_fn: F,
    ) {
        run_fn(device, self.command_buffer);
    }
    pub fn submit_command_buffer(
        &mut self,
        device: &Device,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
    ) {
        self.state = match self.state {
            DrawCommandBufferState::Initial => panic!("recording must be started"),
            DrawCommandBufferState::Recording => DrawCommandBufferState::Submitted,
            DrawCommandBufferState::Submitted => panic!("must start new command buffer"),
        };
        let signal_semaphores = [self.rendering_complete_semaphore];
        unsafe {
            device
                .end_command_buffer(self.command_buffer)
                .expect("failed to create command buffer");
            let command_buffers = [self.command_buffer];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_mask)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            device
                .queue_submit(
                    submit_queue,
                    &[submit_info],
                    self.command_buffer_reuse_fence.expect("should exist"),
                )
                .expect("failed to submit queue");
        }
    }
    pub fn rendering_complete_semaphore(&self) -> vk::Semaphore {
        self.rendering_complete_semaphore
    }
    pub fn free(&self, device: &Device) {
        unsafe {
            device.device_wait_idle().expect("failed to wait idle");
            if let Some(fence) = self.command_buffer_reuse_fence {
                device.destroy_fence(fence, None);
            }
            device.destroy_semaphore(self.rendering_complete_semaphore, None);
            device
                .reset_command_buffer(
                    self.command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("failed to reset buffer");
        }
    }
}
