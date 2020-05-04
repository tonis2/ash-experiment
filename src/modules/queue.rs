use super::context::Context;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

use super::swapchain::Swapchain;
use crate::constants::MAX_FRAMES_IN_FLIGHT;
use std::ptr;

#[derive(Debug, Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub compute_family: Option<u32>,
}

//Queue contains all the functionality neccesary to get frame and draw onto it
pub struct Queue {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
    pub context: Arc<Context>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
            compute_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

impl Queue {
    pub fn new(context: Arc<Context>) -> Self {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
        };

        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut inflight_fences = vec![];

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = context
                    .device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let render_finished_semaphore = context
                    .device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let inflight_fence = context
                    .device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object!");

                image_available_semaphores.push(image_available_semaphore);
                render_finished_semaphores.push(render_finished_semaphore);
                inflight_fences.push(inflight_fence);
            }
        }

        Self {
            image_available_semaphores,
            render_finished_semaphores,
            inflight_fences,
            current_frame: 0,
            context,
        }
    }

    pub fn wait_queue_idle(&self) {
        unsafe {
            self.context
                .device
                .queue_wait_idle(self.context.graphics_queue)
                .expect("Failed to wait graphics queue");
        }
    }

    pub fn load_next_frame(&self, swapchain: &Swapchain) -> Result<(u32, bool), vk::Result> {
        unsafe {
            self.context
                .device
                .wait_for_fences(
                    &[self.inflight_fences[self.current_frame]],
                    true,
                    std::u64::MAX,
                )
                .expect("Failed to wait for Fence!");

            swapchain.swapchain_loader.acquire_next_image(
                swapchain.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )
        }
    }

    pub fn render_frame(
        &mut self,
        swapchain: &Swapchain,
        command_buffer: vk::CommandBuffer,
        image: u32,
    ) {
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[self.image_available_semaphores[self.current_frame]])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[command_buffer])
            .signal_semaphores(&[self.render_finished_semaphores[self.current_frame]])
            .build();

        unsafe {
            self.context
                .device
                .reset_fences(&[self.inflight_fences[self.current_frame]])
                .expect("Failed to reset Fence!");

            self.context
                .device
                .queue_submit(
                    self.context.graphics_queue,
                    &[submit_info],
                    self.inflight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[self.render_finished_semaphores[self.current_frame]])
            .swapchains(&[swapchain.swapchain])
            .image_indices(&[image])
            .build();

        unsafe {
            let swapchain_presentation_result = swapchain
                .swapchain_loader
                .queue_present(self.context.present_queue, &present_info);

            match swapchain_presentation_result {
                Ok(is_suboptimal) if is_suboptimal => {
                    // TODO: Recreate the swapchain
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    // TODO: Recreate the swapchain
                }
                Err(error) => panic!("Failed to present queue. Cause: {}", error),
                _ => {}
            }
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.context.wait_idle();
                self.context
                    .device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.context
                    .device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.context
                    .device
                    .destroy_fence(self.inflight_fences[i], None);
            }
        }
    }
}
