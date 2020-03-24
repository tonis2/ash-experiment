use crate::Context;
use ash::version::DeviceV1_0;
use ash::vk;
use std::sync::Arc;

pub struct Image {
    pub image: vk::Image,
    pub allocation: vk_mem::Allocation,
    pub allication_info: vk_mem::AllocationInfo,
    image_view: Option<vk::ImageView>,
    context: Arc<Context>,
}

impl Image {
    pub fn create_image(
        image_info: vk::ImageCreateInfo,
        usage: vk_mem::MemoryUsage,
        context: Arc<Context>,
    ) -> Image {
        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage,
            ..Default::default()
        };
        let (image, allocation, info) = context
            .memory
            .create_image(&image_info, &allocation_create_info)
            .expect("Failed to create image");

        Image {
            image,
            allocation,
            allication_info: info,
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
            self.context
                .memory
                .destroy_image(self.image, &self.allocation)
                .expect("Failed to destroy image!");
            if self.image_view.is_some() {
                self.context
                    .device
                    .destroy_image_view(self.image_view.unwrap(), None);
            }
        }
    }
}
