use crate::Context;
use ash::version::DeviceV1_0;
use ash::vk;
use std::sync::Arc;

pub struct Image {
    image: vk::Image,
    pub allocation: vk_mem::Allocation,
    pub allication_info: vk_mem::AllocationInfo,
    pub format: vk::Format,
    image_view: Option<vk::ImageView>,
    sampler: Option<vk::Sampler>,
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
            format: image_info.format,
            image_view: None,
            sampler: None,
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

    pub fn attach_sampler(&mut self, sampler_info: vk::SamplerCreateInfo) {
        self.sampler = Some(unsafe {
            self.context
                .device
                .create_sampler(&sampler_info, None)
                .expect("Failed to create Sampler!")
        });
    }

    pub fn image(&self) -> vk::Image {
        self.image
    }

    pub fn view(&self) -> vk::ImageView {
        self.image_view.expect("No image attached")
    }

    pub fn sampler(&self) -> vk::Sampler {
        self.sampler.expect("No sampler attached")
    }

    pub fn create_sampler(
        context: Arc<Context>,
        sampler_info: vk::SamplerCreateInfo,
    ) -> vk::Sampler {
        unsafe {
            context
                .device
                .create_sampler(&sampler_info, None)
                .expect("Failed to create Sampler!")
        }
    }

    pub fn create_view(
        context: Arc<Context>,
        image_info: vk::ImageViewCreateInfo,
    ) -> vk::ImageView {
        unsafe {
            context
                .device
                .create_image_view(&image_info, None)
                .expect("Failed to create Image View!")
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.context.wait_idle();
            self.context
                .memory
                .destroy_image(self.image, &self.allocation)
                .expect("Failed to destroy image!");
            if self.image_view.is_some() {
                self.context
                    .device
                    .destroy_image_view(self.image_view.unwrap(), None);
            }
            if self.sampler.is_some() {
                self.context
                    .device
                    .destroy_sampler(self.sampler.unwrap(), None);
            }
        }
    }
}
