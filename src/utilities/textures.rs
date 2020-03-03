use ash::version::DeviceV1_0;
use ash::vk;

use std::ptr;

use super::tools::find_memory_type;
use crate::VkInstance;

pub struct Image {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
}

impl Image {
    pub fn create_image(
        image_info: vk::ImageCreateInfo,
        required_memory_properties: vk::MemoryPropertyFlags,
        vulkan: &VkInstance,
    ) -> Image {
        let texture_image = unsafe {
            vulkan
                .device
                .create_image(&image_info, None)
                .expect("Failed to create Texture Image!")
        };

        let image_memory_requirement =
            unsafe { vulkan.device.get_image_memory_requirements(texture_image) };
        let memory_allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: image_memory_requirement.size,
            memory_type_index: find_memory_type(
                image_memory_requirement.memory_type_bits,
                required_memory_properties,
                &vulkan.get_physical_device_memory_properties(),
            ),
        };

        let texture_image_memory = unsafe {
            vulkan
                .device
                .allocate_memory(&memory_allocate_info, None)
                .expect("Failed to allocate Texture Image memory!")
        };

        unsafe {
            vulkan
                .device
                .bind_image_memory(texture_image, texture_image_memory, 0)
                .expect("Failed to bind Image Memmory!");
        };

        Image {
            image: texture_image,
            memory: texture_image_memory,
        }
    }

    pub fn destroy(&self, vulkan: &VkInstance) {
        unsafe {
            vulkan.device.destroy_image(self.image, None);
            vulkan.device.free_memory(self.memory, None);
        }
    }
}
