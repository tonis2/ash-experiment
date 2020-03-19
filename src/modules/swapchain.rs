use super::context::Context;
use std::sync::Arc;

use ash::{version::DeviceV1_0, vk};
use std::ptr;

use super::device::query_swapchain_support;
use winit::window::Window;

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
    pub fn new(context: Arc<Context>, window: &Window) -> Swapchain {
        unsafe {
            let swapchain_support =
                query_swapchain_support(context.physical_device, &context.surface);

            let surface_format = choose_swapchain_format(&swapchain_support.formats);
            let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
            let extent = choose_swapchain_extent(&swapchain_support.capabilities, window);

            let image_count = swapchain_support.capabilities.min_image_count + 1;
            let image_count = if swapchain_support.capabilities.max_image_count > 0 {
                image_count.min(swapchain_support.capabilities.max_image_count)
            } else {
                image_count
            };

            let queue_family = &context.queue_family;

            let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
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

            let swapchain_create_info = vk::SwapchainCreateInfoKHR {
                s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
                p_next: ptr::null(),
                flags: vk::SwapchainCreateFlagsKHR::empty(),
                surface: context.surface.surface,
                min_image_count: image_count,
                image_color_space: surface_format.color_space,
                image_format: surface_format.format,
                image_extent: extent,
                image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                image_sharing_mode,
                p_queue_family_indices: queue_family_indices.as_ptr(),
                queue_family_index_count,
                pre_transform: swapchain_support.capabilities.current_transform,
                composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                present_mode,
                clipped: vk::TRUE,
                old_swapchain: vk::SwapchainKHR::null(),
                image_array_layers: 1,
            };

            let swapchain_loader =
                ash::extensions::khr::Swapchain::new(&context.instance, &context.device);
            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!");

            let swapchain_images = swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.");

            let swapchain_imageviews: Vec<vk::ImageView> = swapchain_images
                .iter()
                .map(|&image| {
                    let imageview_create_info = vk::ImageViewCreateInfo {
                        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                        p_next: ptr::null(),
                        flags: vk::ImageViewCreateFlags::empty(),
                        view_type: vk::ImageViewType::TYPE_2D,
                        format: surface_format.format,
                        components: vk::ComponentMapping {
                            r: vk::ComponentSwizzle::IDENTITY,
                            g: vk::ComponentSwizzle::IDENTITY,
                            b: vk::ComponentSwizzle::IDENTITY,
                            a: vk::ComponentSwizzle::IDENTITY,
                        },
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        image,
                    };

                    context
                        .device
                        .create_image_view(&imageview_create_info, None)
                        .expect("Failed to create Image View!")
                })
                .collect();

            Swapchain {
                swapchain,
                images: swapchain_images,
                swapchain_loader,
                format: surface_format.format,
                extent,
                image_views: swapchain_imageviews,
                context,
            }
        }
    }

    pub fn build_color_buffer(
        &self,
        render_pass: vk::RenderPass,
        attachments: Vec<vk::ImageView>,
    ) -> vk::Framebuffer {
        let framebuffer_create_info = vk::FramebufferCreateInfo {
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: self.extent.width,
            height: self.extent.height,
            layers: 1,
            ..Default::default()
        };

        return unsafe {
            self.context
                .device
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create Framebuffer!")
        };
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
        surface_stuff: &super::surface::VkSurface,
    ) -> SwapchainSupport {
        unsafe {
            let capabilities = surface_stuff
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface_stuff.surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface_stuff
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface_stuff.surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface_stuff
                .surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface_stuff.surface)
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

pub fn choose_swapchain_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    window: &winit::window::Window,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::max_value() {
        capabilities.current_extent
    } else {
        use num::clamp;

        let window_size = window.inner_size();
        println!(
            "\t\tInner Window Size: ({}, {})",
            window_size.width, window_size.height
        );

        vk::Extent2D {
            width: clamp(
                window_size.width as u32,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: clamp(
                window_size.height as u32,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
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
