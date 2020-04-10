use super::context::Context;
use std::sync::Arc;

use ash::{extensions::khr::Surface, version::DeviceV1_0, vk};

use super::device::query_swapchain_support;

pub struct Framebuffer {
    buffer: vk::Framebuffer,
    context: Arc<Context>,
}

impl Framebuffer {
    pub fn buffer(&self) -> vk::Framebuffer {
        self.buffer
    }

    pub fn new(info: vk::FramebufferCreateInfo, context: Arc<Context>) -> Framebuffer {
        let buffer = unsafe {
            context
                .device
                .create_framebuffer(&info, None)
                .expect("Failed to create Framebuffer!")
        };

        Framebuffer {
            buffer,
            context: context.clone(),
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.destroy_framebuffer(self.buffer, None);
        }
    }
}

pub struct Swapchain {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub context: Arc<Context>,
}

impl Swapchain {
    pub fn new(context: Arc<Context>) -> Swapchain {
        unsafe {
            let swapchain_support = query_swapchain_support(
                context.physical_device,
                &context.surface_loader,
                context.surface,
            );

            let swapchain_loader =
                ash::extensions::khr::Swapchain::new(&context.instance, &context.device);

            let surface_format = choose_swapchain_format(&swapchain_support.formats);
            let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
   
            let queue_family = &context.queue_family;

            let (image_sharing_mode, _queue_family_index_count, queue_family_indices) =
                if queue_family.graphics_family != queue_family.present_family {
                    (
                        vk::SharingMode::CONCURRENT,
                        2,
                        vec![
                            queue_family.graphics_family.unwrap(),
                            queue_family.present_family.unwrap(),
                        ],
                    )
                } else {
                    (vk::SharingMode::EXCLUSIVE, 0, vec![])
                };
            let extent = swapchain_support.capabilities.current_extent;
            let swapchain = swapchain_loader
                .create_swapchain(
                    &vk::SwapchainCreateInfoKHR {
                        surface: context.surface,
                        min_image_count: context.image_count,
                        image_color_space: surface_format.color_space,
                        image_format: surface_format.format,
                        image_extent: extent,
                        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                        image_sharing_mode,
                        p_queue_family_indices: queue_family_indices.as_ptr(),
                        queue_family_index_count: queue_family_indices.len() as u32,
                        pre_transform: swapchain_support.capabilities.current_transform,
                        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                        present_mode,
                        clipped: vk::TRUE,
                        old_swapchain: vk::SwapchainKHR::null(),
                        image_array_layers: 1,
                        ..Default::default()
                    },
                    None,
                )
                .expect("Failed to create Swapchain!");

            let swapchain_images = swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.");

            let swapchain_imageviews: Vec<vk::ImageView> = swapchain_images
                .iter()
                .map(|&image| {
                    let imageview_create_info = vk::ImageViewCreateInfo::builder()
                        .format(surface_format.format)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::IDENTITY,
                            g: vk::ComponentSwizzle::IDENTITY,
                            b: vk::ComponentSwizzle::IDENTITY,
                            a: vk::ComponentSwizzle::IDENTITY,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image)
                        .build();

                    context
                        .device
                        .create_image_view(&imageview_create_info, None)
                        .expect("Failed to create Swapchain image view!")
                })
                .collect();

            Swapchain {
                swapchain,
                swapchain_loader,
                images: swapchain_images,
                format: surface_format.format,
                extent,
                image_views: swapchain_imageviews,
                context,
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.extent.width
    }

    pub fn height(&self) -> u32 {
        self.extent.height
    }

    pub fn get_image(&self, image_index: usize) -> vk::ImageView {
        self.image_views[image_index]
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.context.wait_idle();
            for image in self.image_views.iter() {
                self.context.device.destroy_image_view(*image, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface_loader: &Surface,
        surface: vk::SurfaceKHR,
    ) -> SwapchainSupport {
        unsafe {
            let capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Failed to query for surface present mode.");

            SwapchainSupport {
                capabilities,
                formats,
                present_modes,
            }
        }
    }
}

pub fn choose_swapchain_present_mode(
    available_present_modes: &Vec<vk::PresentModeKHR>,
) -> vk::PresentModeKHR {
    for &available_present_mode in available_present_modes.iter() {
        if available_present_mode == vk::PresentModeKHR::MAILBOX {
            return available_present_mode;
        }
    }

    vk::PresentModeKHR::FIFO
}

pub fn choose_swapchain_format(
    available_formats: &Vec<vk::SurfaceFormatKHR>,
) -> vk::SurfaceFormatKHR {
    for available_format in available_formats {
        if available_format.format == vk::Format::B8G8R8A8_UNORM
            && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return available_format.clone();
        }
    }

    return available_formats.first().unwrap().clone();
}
