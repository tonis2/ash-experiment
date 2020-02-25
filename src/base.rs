use crate::modules::{
    debug::create_debugger,
    device::{self, Queue},
    surface,
    swapchain::Frame,
};

use ash::{
    extensions::khr::Surface,
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

use crate::definitions::MAX_FRAMES_IN_FLIGHT;

use std::ptr;
use winit::window::Window;

#[allow(dead_code)]
pub struct VkInstance {
    entry: Entry,
    debugger: vk::DebugReportCallbackEXT,
    pub instance: Instance,
    pub surface: Surface,
    pub surface_khr: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub queue: Queue,
}

impl VkInstance {
    pub fn new(window: &Window) -> Self {
        unsafe {
            let (entry, instance) = crate::modules::create_entry();
            let debugger = create_debugger(&entry, &instance);
            let surface = Surface::new(&entry, &instance);
            let surface_khr = surface::create_surface(&entry, &instance, &window).unwrap();

            let (pdevice, queue_family) =
                device::create_physical_device(&instance, &surface, surface_khr);
            let (device, queue) = device::create_device(queue_family, &instance, pdevice);

            Self {
                entry,
                debugger,
                instance,
                surface,
                surface_khr,
                physical_device: pdevice,
                device: device,
                queue,
            }
        }
    }
}

impl VkInstance {
    pub fn get_physical_device_memory_properties(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        }
    }

    pub fn wait_idle(&self) -> std::result::Result<(), vk::Result> {
        unsafe { self.device.device_wait_idle() }
    }

    pub fn create_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        amount: u32,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: amount,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        unsafe {
            self.device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        }
    }

    pub fn create_command_pool(&self) -> vk::CommandPool {
        unsafe {
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(self.queue.queue_family_indices.graphics_family.unwrap());

            self.device
                .create_command_pool(&pool_create_info, None)
                .unwrap()
        }
    }

    pub fn draw_frame(&mut self, frame: Frame, command_buffers: &Vec<vk::CommandBuffer>) {
        let wait_fences = [self.queue.inflight_fences[self.queue.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");

            frame
                .swapchain
                .swapchain_loader
                .acquire_next_image(
                    frame.swapchain.swapchain,
                    std::u64::MAX,
                    self.queue.image_available_semaphores[self.queue.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.")
        };

        let wait_semaphores = [self.queue.image_available_semaphores[self.queue.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.queue.render_finished_semaphores[self.queue.current_frame]];
   
        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];
        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .queue_submit(
                    self.queue.graphics_queue,
                    &submit_infos,
                    self.queue.inflight_fences[self.queue.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [frame.swapchain.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = unsafe {
            frame
                .swapchain
                .swapchain_loader
                .queue_present(self.queue.present_queue, &present_info)
        };

        let _is_resized = match result {
            Ok(_) => false,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };

        self.queue.current_frame = (self.queue.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        self.wait_idle().unwrap();
        frame.destroy(self);
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.queue.destroy(&self.device);
            self.device.destroy_device(None);
        }
    }
}
