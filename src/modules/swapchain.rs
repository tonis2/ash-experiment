use crate::base::VkInstance;
use ash::{extensions::khr, vk};

pub struct Swapchain {
    swapchain: vk::SwapchainKHR,
}

impl Swapchain {
    pub fn new(vulkan: &VkInstance, width: u32, height: u32) -> Swapchain {
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

            Swapchain { swapchain }
        }
    }
}
