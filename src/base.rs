use crate::modules::{
    debug::create_debugger,
    device::{self, Queue},
    surface,
};
use ash::{
    extensions::khr::Surface,
    version::{DeviceV1_0, InstanceV1_0},
    vk, Device, Entry, Instance,
};

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
    pub queue_family_index: u32,
    pub queue: Queue,
}

impl VkInstance {
    pub fn new(window: &Window) -> Self {
        unsafe {
            let (entry, instance) = crate::modules::create_entry();
            let debugger = create_debugger(&entry, &instance);
            let surface = Surface::new(&entry, &instance);
            let surface_khr = surface::create_surface(&entry, &instance, &window).unwrap();

            let (pdevice, queue_family_index) =
                device::create_physical_device(&instance, &surface, surface_khr);
            let (device, queue) = device::create_device(queue_family_index, &instance, pdevice);

            Self {
                entry,
                debugger,
                instance,
                surface,
                surface_khr,
                physical_device: pdevice,
                device: device,
                queue_family_index,
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
                .queue_family_index(self.queue_family_index);

            self.device
                .create_command_pool(&pool_create_info, None)
                .unwrap()
        }
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.destroy_device(None);
        }
    }
}
