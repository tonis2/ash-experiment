use super::instance::VkInstance;
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
}
impl Swapchain {
    pub fn new(vulkan: &VkInstance, window: &Window) -> Swapchain {
        unsafe {
            let swapchain_support =
                query_swapchain_support(vulkan.physical_device, &vulkan.surface);

            let surface_format = choose_swapchain_format(&swapchain_support.formats);
            let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
            let extent = choose_swapchain_extent(&swapchain_support.capabilities, window);

            let image_count = swapchain_support.capabilities.min_image_count + 1;
            let image_count = if swapchain_support.capabilities.max_image_count > 0 {
                image_count.min(swapchain_support.capabilities.max_image_count)
            } else {
                image_count
            };

            let queue_family = &vulkan.queue.queue_family_indices;

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
                surface: vulkan.surface.surface,
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
                ash::extensions::khr::Swapchain::new(&vulkan.instance, &vulkan.device);
            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!");

            let swapchain_images = swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.");

            let swapchain_imageviews: Vec<vk::ImageView> = swapchain_images
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
                images: swapchain_images,
                swapchain_loader,
                format: surface_format.format,
                extent,
                image_views: swapchain_imageviews,
            }
        }
    }

    pub fn create_frame_buffers(
        &self,
        render_pass: &vk::RenderPass,
        vulkan: &VkInstance,
    ) -> Vec<vk::Framebuffer> {
        let mut frame_buffers = vec![];

        for &image_view in self.image_views.iter() {
            let attachments = [image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass: *render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: self.extent.width,
                height: self.extent.height,
                layers: 1,
            };

            let framebuffer = unsafe {
                vulkan
                    .device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create Framebuffer!")
            };

            frame_buffers.push(framebuffer);
        }
        frame_buffers
    }

    pub fn destroy(&self, vulkan: &VkInstance) {
        for image in self.image_views.iter() {
            unsafe {
                vulkan.device.destroy_image_view(*image, None);
            }
        }
    }

    pub fn create_render_pass<D: DeviceV1_0>(&self, device: &D) -> vk::RenderPass {
        let color_attachment = vk::AttachmentDescription {
            format: self.format,
            flags: vk::AttachmentDescriptionFlags::empty(),
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpasses = [vk::SubpassDescription {
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: ptr::null(),
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        }];

        let render_pass_attachments = [color_attachment];

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let renderpass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            flags: vk::RenderPassCreateFlags::empty(),
            p_next: ptr::null(),
            attachment_count: render_pass_attachments.len() as u32,
            p_attachments: render_pass_attachments.as_ptr(),
            subpass_count: subpasses.len() as u32,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
        };

        unsafe {
            device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
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
