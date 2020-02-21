use crate::base::VkInstance;
use ash::{extensions::khr, version::DeviceV1_0, vk};
use std::ptr;

pub struct Swapchain {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}

pub struct Frame {
    frame_buffers: Vec<vk::Framebuffer>,
    command_buffers: Vec<vk::CommandBuffer>,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
}

impl Frame {
    pub fn finish<D: DeviceV1_0, F: Fn(vk::CommandBuffer, &D)>(&self, device: &D, apply: F) {
        unsafe {
            for (i, &command_buffer) in self.command_buffers.iter().enumerate() {
                let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                    s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                    p_next: ptr::null(),
                    p_inheritance_info: ptr::null(),
                    flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                };

                device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Failed to begin recording Command Buffer at beginning!");

                let clear_values = [vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                }];

                let render_pass_begin_info = vk::RenderPassBeginInfo {
                    s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                    p_next: ptr::null(),
                    render_pass: self.render_pass,
                    framebuffer: self.frame_buffers[i],
                    render_area: vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.extent,
                    },
                    clear_value_count: clear_values.len() as u32,
                    p_clear_values: clear_values.as_ptr(),
                };

                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                apply(command_buffer, &device);
                device.cmd_end_render_pass(command_buffer);

                device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }
    }

    pub fn destroy(&self, vulkan: &VkInstance) {
        for &framebuffer in self.frame_buffers.iter() {
            unsafe {
                vulkan.device.destroy_framebuffer(framebuffer, None);
            }
        }
    }
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
                images: swapchain_images,
                swapchain_loader,
                format: surface_format.format,
                extent: surface_resolution,
                image_views: swapchain_image_views,
            }
        }
    }

    pub fn build_next_frame<D: DeviceV1_0>(
        &self,
        command_buffers: Vec<vk::CommandBuffer>,
        render_pass: vk::RenderPass,
        device: &D,
    ) -> Frame {
        let mut frame_buffers = vec![];

        for &image_view in self.image_views.iter() {
            let attachments = [image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: self.extent.width,
                height: self.extent.height,
                layers: 1,
            };

            let framebuffer = unsafe {
                device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create Framebuffer!")
            };

            frame_buffers.push(framebuffer);
        }

        Frame {
            command_buffers,
            frame_buffers,
            render_pass,
            extent: self.extent,
        }
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
