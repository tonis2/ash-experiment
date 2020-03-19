use crate::Context;
use ash::vk;
use std::sync::Arc;

use std::ptr;

use super::tools::find_memory_type;
use ash::version::DeviceV1_0;

pub struct Image {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub image_view: Option<vk::ImageView>,
    pub context: Arc<Context>,
}

impl Image {
    pub fn create_image(
        image_info: vk::ImageCreateInfo,
        required_memory_properties: vk::MemoryPropertyFlags,
        context: Arc<Context>,
    ) -> Image {
        let texture_image = unsafe {
            context
                .device
                .create_image(&image_info, None)
                .expect("Failed to create Texture Image!")
        };

        let image_memory_requirement =
            unsafe { context.device.get_image_memory_requirements(texture_image) };

        let memory_allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: image_memory_requirement.size,
            memory_type_index: find_memory_type(
                image_memory_requirement.memory_type_bits,
                required_memory_properties,
                &context.get_physical_device_memory_properties(),
            ),
        };

        let texture_image_memory = unsafe {
            context
                .device
                .allocate_memory(&memory_allocate_info, None)
                .expect("Failed to allocate Texture Image memory!")
        };

        unsafe {
            context
                .device
                .bind_image_memory(texture_image, texture_image_memory, 0)
                .expect("Failed to bind Image Memmory!");
        };

        Image {
            image: texture_image,
            memory: texture_image_memory,
            image_view: None,
            context: context.clone(),
        }
    }

    pub fn attach_view(&mut self, image_info: vk::ImageViewCreateInfo) {
        self.image_view = Some(unsafe {
            self.context
                .device
                .create_image_view(&image_info, None)
                .expect("Failed to create Image View!")
        });
    }

    pub fn view(&self) -> vk::ImageView {
        self.image_view.expect("No image attached")
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.context.device.destroy_image(self.image, None);
            self.context.device.free_memory(self.memory, None);
        }
    }
}
