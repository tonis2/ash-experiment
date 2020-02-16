use crate::base::VkInstance;
use ash::{extensions::khr, version::DeviceV1_0, vk};
use std::ptr;

pub struct Swapchain<'a> {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub vulkan: &'a VkInstance,
}

impl<'a> Swapchain<'a> {
    pub fn new(vulkan: &'a VkInstance, width: u32, height: u32) -> Swapchain {
        unsafe {
            let surface_formats = vulkan
                .surface
                .get_physical_device_surface_formats(vulkan.physical_device, vulkan.surface_khr)
                .unwrap();
            let surface_format = surface_formats
                .iter()
                .map(|sfmt| match sfmt.format {
                    vk::Format::UNDEFINED => vk::SurfaceFormatKHR {
                        format: vk::Format::B8G8R8_UNORM,
                        color_space: sfmt.color_space,
                    },
                    _ => *sfmt,
                })
                .nth(0)
                .expect("Unable to find suitable surface format.");
            let surface_capabilities = vulkan
                .surface
                .get_physical_device_surface_capabilities(
                    vulkan.physical_device,
                    vulkan.surface_khr,
                )
                .unwrap();
            let mut desired_image_count = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0
                && desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = surface_capabilities.max_image_count;
            }
            let surface_resolution = match surface_capabilities.current_extent.width {
                std::u32::MAX => vk::Extent2D { width, height },
                _ => surface_capabilities.current_extent,
            };
            let pre_transform = if surface_capabilities
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_capabilities.current_transform
            };
            let present_modes = vulkan
                .surface
                .get_physical_device_surface_present_modes(
                    vulkan.physical_device,
                    vulkan.surface_khr,
                )
                .unwrap();
            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);
            let swapchain_loader = khr::Swapchain::new(&vulkan.instance, &vulkan.device);

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(vulkan.surface_khr)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_extent(surface_resolution)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);

            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();

            let swapchain_images = swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.");

            let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
                .iter()
                .map(|&image| {
                    create_image_view(
                        &vulkan.device,
                        image,
                        surface_format.format,
                        vk::ImageAspectFlags::COLOR,
                        1,
                    )
                })
                .collect();

            Swapchain {
                swapchain,
                swapchain_images,
                swapchain_loader,
                swapchain_format: surface_format.format,
                swapchain_extent: surface_resolution,
                swapchain_image_views,
                vulkan,
            }
        }
    }

    pub fn create_frame(&self) {}
}

impl<'a> Drop for Swapchain<'a> {
    fn drop(&mut self) {
        unsafe {
            for &imageview in self.swapchain_image_views.iter() {
                self.vulkan.device.destroy_image_view(imageview, None);
            }
        }
    }
}

pub fn create_image_view(
    device: &ash::Device,
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
    mip_levels: u32,
) -> vk::ImageView {
    let imageview_create_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ImageViewCreateFlags::empty(),
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        },
        image,
    };

    unsafe {
        device
            .create_image_view(&imageview_create_info, None)
            .expect("Failed to create Image View!")
    }
}
