use super::Vertex;
use std::{default::Default, ffi::CString, mem, path::Path, sync::Arc};
use vulkan::{offset_of, prelude::*, utilities::Shader, Context, Framebuffer, Image, Swapchain};

const DEPTH_FORMAT: vk::Format = vk::Format::D16_UNORM;
pub struct Pipeline {
    pub framebuffer: Framebuffer,
    pub renderpass: vk::RenderPass,
    pub render_pass_info: vk::RenderPassBeginInfo,
    pub sampler: vk::Sampler,
    pub image: Image,
    pub pipeline: vk::Pipeline,
    pub context: Arc<Context>,
}

impl Pipeline {
    pub fn new(swapchain: &Swapchain, context: Arc<Context>, layout: vk::PipelineLayout) -> Self {
        //Create shadow pipeline stuff

        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };

        let viewports = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissors = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        };

        let renderpass = Self::create_render_pass(context.clone());
        let shader_name = CString::new("main").unwrap();
        let pipeline = unsafe {
            context
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[vk::GraphicsPipelineCreateInfo::builder()
                        .stages(&[Shader::new(
                            &Path::new("src/bin/lights/shaders/offscreen.vert.spv"),
                            vk::ShaderStageFlags::VERTEX,
                            &shader_name,
                            context.clone(),
                        )
                        .info()])
                        .vertex_input_state(
                            &vk::PipelineVertexInputStateCreateInfo::builder()
                                .vertex_binding_descriptions(&[vk::VertexInputBindingDescription {
                                    binding: 0,
                                    stride: mem::size_of::<Vertex>() as u32,
                                    input_rate: vk::VertexInputRate::VERTEX,
                                }])
                                .vertex_attribute_descriptions(&[
                                    vk::VertexInputAttributeDescription {
                                        binding: 0,
                                        location: 0,
                                        format: vk::Format::R32G32B32_SFLOAT,
                                        offset: offset_of!(Vertex, pos) as u32,
                                    },
                                ])
                                .build(),
                        )
                        .input_assembly_state(&vk::PipelineInputAssemblyStateCreateInfo {
                            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                            ..Default::default()
                        })
                        .viewport_state(
                            &vk::PipelineViewportStateCreateInfo::builder()
                                .scissors(&[scissors])
                                .viewports(&[viewports]),
                        )
                        .rasterization_state(&vk::PipelineRasterizationStateCreateInfo {
                            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                            line_width: 1.0,
                            polygon_mode: vk::PolygonMode::FILL,
                            ..Default::default()
                        })
                        .multisample_state(&vk::PipelineMultisampleStateCreateInfo {
                            rasterization_samples: vk::SampleCountFlags::TYPE_1,
                            ..Default::default()
                        })
                        .depth_stencil_state(&vk::PipelineDepthStencilStateCreateInfo {
                            depth_test_enable: 1,
                            depth_write_enable: 1,
                            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
                            front: noop_stencil_state,
                            back: noop_stencil_state,
                            max_depth_bounds: 1.0,
                            ..Default::default()
                        })
                        .color_blend_state(
                            &vk::PipelineColorBlendStateCreateInfo::builder()
                                .logic_op(vk::LogicOp::CLEAR)
                                .attachments(&[vk::PipelineColorBlendAttachmentState {
                                    blend_enable: 0,
                                    src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
                                    dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
                                    color_blend_op: vk::BlendOp::ADD,
                                    src_alpha_blend_factor: vk::BlendFactor::ZERO,
                                    dst_alpha_blend_factor: vk::BlendFactor::ZERO,
                                    alpha_blend_op: vk::BlendOp::ADD,
                                    color_write_mask: vk::ColorComponentFlags::all(),
                                }]),
                        )
                        .dynamic_state(
                            &vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&[
                                vk::DynamicState::VIEWPORT,
                                vk::DynamicState::SCISSOR,
                            ]),
                        )
                        .layout(layout)
                        .render_pass(renderpass)
                        .build()],
                    None,
                )
                .expect("Unable to create graphics pipeline")
        }[0];

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

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .framebuffer(framebuffer.buffer())
            .render_pass(renderpass)
            .render_area(scissors)
            .clear_values(&[vk::ClearValue {
                // clear value for depth buffer
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            }])
            .build();

        Pipeline {
            framebuffer,
            renderpass,
            sampler,
            pipeline,
            render_pass_info,
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

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.context.wait_idle();
            self.context.device.destroy_pipeline(self.pipeline, None);
            self.context.device.destroy_sampler(self.sampler, None);
            self.context
                .device
                .destroy_render_pass(self.renderpass, None);
        }
    }
}
