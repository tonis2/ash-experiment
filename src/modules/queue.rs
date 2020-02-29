use ash::{
    version::{DeviceV1_0},
    vk,
};

use crate::definitions::MAX_FRAMES_IN_FLIGHT;


use std::ptr;


pub struct Frame {
    pub wait_fences: Vec<vk::Fence>,
    pub signal_semaphores: Vec<vk::Semaphore>,
    pub wait_semaphores: Vec<vk::Semaphore>,
    pub image_index: u32,
    pub is_sub_optimal: bool,
    pub wait_stages: Vec<vk::PipelineStageFlags>,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
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
pub struct Queue {
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub queue_family_indices: QueueFamilyIndices,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
    pub current_frame: usize,
}

impl Queue {
    pub fn new<D: DeviceV1_0>(device: &D, queue_family_indices: QueueFamilyIndices) -> Self {
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
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let inflight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object!");

                image_available_semaphores.push(image_available_semaphore);
                render_finished_semaphores.push(render_finished_semaphore);
                inflight_fences.push(inflight_fence);
            }
        }
        unsafe {
            Self {
                graphics_queue: device
                    .get_device_queue(queue_family_indices.graphics_family.unwrap(), 0),
                present_queue: device
                    .get_device_queue(queue_family_indices.present_family.unwrap(), 0),
                queue_family_indices,
                image_available_semaphores,
                render_finished_semaphores,
                inflight_fences,
                current_frame: 0,
            }
        }
    }

    pub fn next_frame(&self, vulkan: &crate::VkInstance, swapchain: &crate::Swapchain) -> Frame {
        let wait_fences = vec![self.inflight_fences[self.current_frame]];

        let (image_index, is_sub_optimal) = unsafe {
            vulkan.device
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
        let signal_semaphores =
            vec![self.render_finished_semaphores[self.current_frame]];

            Frame {
                wait_fences,
                image_index,
                wait_semaphores,
                wait_stages,
                is_sub_optimal,
                signal_semaphores
            }
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                device.destroy_semaphore(self.image_available_semaphores[i], None);
                device.destroy_semaphore(self.render_finished_semaphores[i], None);
                device.destroy_fence(self.inflight_fences[i], None);
            }
        }
    }
}
