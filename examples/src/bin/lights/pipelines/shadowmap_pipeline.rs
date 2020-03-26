use std::sync::Arc;
use vulkan::{prelude::*, Context, Framebuffer, Image, Swapchain};
const DEPTH_FORMAT: vk::Format = vk::Format::D16_UNORM;

pub struct Pipeline {
    pub framebuffer: Framebuffer,
    pub renderpass: vk::RenderPass,
    pub sampler: vk::Sampler,
    pub image: Image,
    pub context: Arc<Context>,
}

impl Pipeline {
    pub fn new(swapchain: &Swapchain, context: Arc<Context>) -> Self {
        let mut shadow_map_image = Image::create_image(
            vk::ImageCreateInfo {
                s_type: vk::StructureType::IMAGE_CREATE_INFO,
                image_type: vk::ImageType::TYPE_2D,
                format: DEPTH_FORMAT,
                extent: vk::Extent3D {
                    width: swapchain.extent.width,
                    height: swapchain.extent.height,
                    depth: 1,
                },
                mip_levels: 1,
                array_layers: 1,
                samples: vk::SampleCountFlags::TYPE_1,
                tiling: vk::ImageTiling::OPTIMAL,
                usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            },
            vk_mem::MemoryUsage::GpuOnly,
            context.clone(),
        );

        shadow_map_image.attach_view(vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            view_type: vk::ImageViewType::TYPE_2D,
            format: DEPTH_FORMAT,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            image: shadow_map_image.image,
            ..Default::default()
        });

        let sampler = unsafe {
            context
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo {
                        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
                        mag_filter: vk::Filter::LINEAR,
                        min_filter: vk::Filter::LINEAR,
                        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
                        address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
                        address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
                        address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
                        max_lod: 1.0,
                        border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
                        mip_lod_bias: 0.0,
                        anisotropy_enable: vk::TRUE,
                        max_anisotropy: 16.0,
                        ..Default::default()
                    },
                    None,
                )
                .expect("Failed to create Sampler!")
        };

        let renderpass = Self::create_render_pass(context.clone());
        let framebuffer = Framebuffer::new(
            vk::FramebufferCreateInfo::builder()
                .layers(1)
                .render_pass(renderpass)
                .attachments(&[shadow_map_image.view()])
                .width(swapchain.width())
                .height(swapchain.height())
                .build(),
            context.clone(),
        );

        let descriptor_info = vk::DescriptorImageInfo {
            sampler: sampler,
            image_view: shadow_map_image.view(),
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        Pipeline {
            framebuffer,
            renderpass,
            sampler,
            image: shadow_map_image,
            context,
        }
    }

    fn create_render_pass(context: Arc<Context>) -> vk::RenderPass {
        let depth_attachment = vk::AttachmentDescription {
            flags: vk::AttachmentDescriptionFlags::empty(),
            format: DEPTH_FORMAT,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpasses = vk::SubpassDescription::builder()
            .depth_stencil_attachment(&vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            })
            .build();

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[depth_attachment])
            .subpasses(&[subpasses])
            .dependencies(&[
                vk::SubpassDependency {
                    src_subpass: vk::SUBPASS_EXTERNAL,
                    dst_subpass: 0,
                    src_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                    dst_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                    src_access_mask: vk::AccessFlags::SHADER_READ,
                    dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    dependency_flags: vk::DependencyFlags::BY_REGION,
                },
                vk::SubpassDependency {
                    src_subpass: 0,
                    dst_subpass: vk::SUBPASS_EXTERNAL,
                    src_stage_mask: vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                    src_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    dependency_flags: vk::DependencyFlags::BY_REGION,
                },
            ])
            .build();

        unsafe {
            context
                .device
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        }
    }
}
