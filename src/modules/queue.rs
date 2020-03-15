use super::context::Context;
use ash::{version::DeviceV1_0, vk};
use std::sync::Arc;

use super::swapchain::Swapchain;
use crate::constants::MAX_FRAMES_IN_FLIGHT;
use std::ptr;
#[derive(Debug)]
pub struct Frame {
    pub wait_fences: Vec<vk::Fence>,
    pub signal_semaphores: Vec<vk::Semaphore>,
    pub wait_semaphores: Vec<vk::Semaphore>,
    pub image_index: usize,
    pub is_sub_optimal: bool,
    pub wait_stages: Vec<vk::PipelineStageFlags>,
}

#[derive(Debug, Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

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

    pub fn build_frame<F: Fn(vk::CommandBuffer, &ash::Device)>(
        &self,
        command_buffer: vk::CommandBuffer,
        render_area: vk::Rect2D,
        clear_values: &[vk::ClearValue],
        attachments: Vec<vk::ImageView>,
        render_pass: vk::RenderPass,
        swapchain: &Swapchain,
        apply: F,
    ) {
        //Build frame buffer
        let attachment_count = attachments.len() as u32;
        let framebuffer_create_info = vk::FramebufferCreateInfo {
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass,
            attachment_count,
            p_attachments: attachments.as_ptr(),
            width: swapchain.extent.width,
            height: swapchain.extent.height,
            layers: 1,
            ..Default::default()
        };

        let framebuffer = unsafe {
            self.context
                .device
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create Framebuffer!")
        };

        //build command buffer

        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            self.context
                .device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass,
                framebuffer,
                render_area: render_area,
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };

            self.context.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            apply(command_buffer, &self.context.device);

            self.context.device.cmd_end_render_pass(command_buffer);

            self.context
                .device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }

    pub fn render_frame(
        &mut self,
        frame: &Frame,
        swapchain: &Swapchain,
        command_buffer: vk::CommandBuffer,
        context: Arc<Context>,
    ) {
        let submit_infos = vec![vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: frame.wait_semaphores.len() as u32,
            p_wait_semaphores: frame.wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: frame.wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: frame.signal_semaphores.len() as u32,
            p_signal_semaphores: frame.signal_semaphores.as_ptr(),
        }];

        unsafe {
            context
                .device
                .reset_fences(&frame.wait_fences)
                .expect("Failed to reset Fence!");
            context
                .device
                .queue_submit(
                    context.graphics_queue,
                    &submit_infos,
                    self.inflight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [swapchain.swapchain];
        let image_index = frame.image_index as u32;
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: frame.signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        unsafe {
            swapchain
                .swapchain_loader
                .queue_present(context.present_queue, &present_info)
                .expect("Failed to execute queue present.");
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn next_frame(&mut self, swapchain: &Swapchain) -> Frame {
        let wait_fences = vec![self.inflight_fences[self.current_frame]];

        let (image_index, is_sub_optimal) = unsafe {
            self.context
                .device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");

            swapchain
                .swapchain_loader
                .acquire_next_image(
                    swapchain.swapchain,
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.")
        };

        let wait_semaphores = vec![self.image_available_semaphores[self.current_frame]];
        let wait_stages = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = vec![self.render_finished_semaphores[self.current_frame]];

        Frame {
            wait_fences,
            image_index: image_index as usize,
            wait_semaphores,
            wait_stages,
            is_sub_optimal,
            signal_semaphores,
        }
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
