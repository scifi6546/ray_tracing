use super::PassBase;
use crate::prelude::*;
use ash::vk;
use std::thread::current;
pub struct SemaphoreBuffer {
    semaphores: [vk::Semaphore; 2],
    current_idx: usize,
    first_render: bool,
}
impl SemaphoreBuffer {
    pub fn new(base: PassBase) -> Self {
        let create_info = vk::SemaphoreCreateInfo::builder();
        let sem_0 = unsafe { base.base.device.create_semaphore(&create_info, None) }
            .expect("failed to create rendering complete semaphore");
        let sem_1 = unsafe { base.base.device.create_semaphore(&create_info, None) }
            .expect("failed to create rendering complete semaphore");
        Self {
            semaphores: [sem_0, sem_1],
            current_idx: 0,
            first_render: true,
        }
    }
    pub fn get_semaphore(&mut self) -> SemaphoreInfo {
        if self.first_render {
            let info = SemaphoreInfo {
                signal_semaphore: self.semaphores[0].clone(),
                wait_semaphore: None,
            };
            self.first_render = false;
            self.current_idx = 1;
            info
        } else {
            let info = if self.current_idx == 0 {
                self.current_idx = 1;
                SemaphoreInfo {
                    signal_semaphore: self.semaphores[0].clone(),
                    wait_semaphore: Some(self.semaphores[1].clone()),
                }
            } else if self.current_idx == 1 {
                self.current_idx = 0;
                SemaphoreInfo {
                    signal_semaphore: self.semaphores[1].clone(),
                    wait_semaphore: Some(self.semaphores[0].clone()),
                }
            } else {
                panic!("invalid idx")
            };
            info
        }
    }
}
#[derive(Clone, Debug)]
pub struct SemaphoreInfo {
    pub signal_semaphore: vk::Semaphore,
    pub wait_semaphore: Option<vk::Semaphore>,
}
