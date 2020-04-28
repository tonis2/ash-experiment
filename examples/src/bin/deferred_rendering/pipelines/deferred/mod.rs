use vulkan::{
    offset_of, prelude::*, utilities::Shader, Buffer, Descriptor, DescriptorSet, Framebuffer,
    Image, Pipeline, Renderpass, Swapchain, VkThread,
};

use std::{default::Default, ffi::CString, mem, path::Path};

use examples::Quad;

pub struct Deferred {
    pub pipeline_descriptor: Descriptor,
    pub framebuffers: Vec<Framebuffer>,
    pub pipeline: Pipeline,
    pub renderpass: Renderpass,
    pub quad_vertex: Buffer,
    pub quad_index: Buffer,
}

impl Deferred {
    pub fn build(images: &Vec<&Image>, swapchain: &Swapchain, vulkan: &VkThread) -> Self {
        //Create descriptors for the gbuffer images
        let pipeline_descriptor = Descriptor::new(
            images
                .iter()
                .enumerate()
                .map(|(index, image)| {
                    DescriptorSet {
                        bind_index: index as u32,
                        flag: vk::ShaderStageFlags::FRAGMENT,
                        bind_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        image_info: Some(vec![vk::DescriptorImageInfo {
                            sampler: image.sampler(),
                            image_view: image.view(),
                            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        }]),
                        ..Default::default()
                    }
                })
                .collect(),
            vulkan.context(),
        );

        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };

        let subpasses = vk::SubpassDescription::builder()
            .color_attachments(&[vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            }])
            .build();

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain.width() as f32,
            height: swapchain.height() as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        }];
        let renderpass = Renderpass::new(
            vk::RenderPassCreateInfo::builder()
                .subpasses(&[subpasses])
                .dependencies(&[vk::SubpassDependency {
                    src_subpass: vk::SUBPASS_EXTERNAL,
                    dst_subpass: 0,
                    src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    src_access_mask: vk::AccessFlags::empty(),
                    dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    dependency_flags: vk::DependencyFlags::empty(),
                }])
                .attachments(&[vk::AttachmentDescription {
                    flags: vk::AttachmentDescriptionFlags::empty(),
                    format: swapchain.format,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: vk::AttachmentLoadOp::CLEAR,
                    store_op: vk::AttachmentStoreOp::STORE,
                    stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                    stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                    initial_layout: vk::ImageLayout::UNDEFINED,
                    final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                }])
                .build(),
            vulkan.context(),
        );

        let framebuffers: Vec<Framebuffer> = swapchain
            .image_views
            .iter()
            .map(|image| {
                Framebuffer::new(
                    vk::FramebufferCreateInfo::builder()
                        .layers(1)
                        .render_pass(renderpass.pass())
                        .attachments(&[*image])
                        .width(swapchain.width())
                        .height(swapchain.height())
                        .build(),
                    vulkan.context(),
                )
            })
            .collect();

        let attributes = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Quad, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Quad, uv) as u32,
            },
        ];
        let shader_name = CString::new("main").unwrap();
        let pipeline = Pipeline::new(
            vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&[pipeline_descriptor.layout])
                .build(),
            vk::GraphicsPipelineCreateInfo::builder()
                .stages(&[
                    Shader::new(
                        &Path::new("src/bin/deferred_rendering/shaders/deferred.vert.spv"),
                        vk::ShaderStageFlags::VERTEX,
                        &shader_name,
                        vulkan.context(),
                    )
                    .info(),
                    Shader::new(
                        &Path::new("src/bin/deferred_rendering/shaders/deferred.frag.spv"),
                        vk::ShaderStageFlags::FRAGMENT,
                        &shader_name,
                        vulkan.context(),
                    )
                    .info(),
                ])
                .vertex_input_state(
                    &vk::PipelineVertexInputStateCreateInfo::builder()
                        .vertex_binding_descriptions(&[vk::VertexInputBindingDescription {
                            binding: 0,
                            stride: mem::size_of::<Quad>() as u32,
                            input_rate: vk::VertexInputRate::VERTEX,
                        }])
                        .vertex_attribute_descriptions(&attributes)
                        .build(),
                )
                .input_assembly_state(&vk::PipelineInputAssemblyStateCreateInfo {
                    topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                    ..Default::default()
                })
                .viewport_state(
                    &vk::PipelineViewportStateCreateInfo::builder()
                        .scissors(&scissors)
                        .viewports(&viewports),
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
                    &vk::PipelineDynamicStateCreateInfo::builder()
                        .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]),
                )
                .render_pass(renderpass.pass()),
            vulkan.context(),
        );
        Self {
            pipeline_descriptor,
            renderpass,
            framebuffers,
            pipeline,
            quad_vertex: Quad::vertex_buffer(&vulkan),
            quad_index: Quad::index_buffer(&vulkan),
        }
    }
}
