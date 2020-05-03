use vulkan::{
    prelude::*, utilities::as_byte_slice, Buffer, Descriptor, DescriptorSet, Framebuffer, Image,
    Pipeline, Renderpass, Shader, Swapchain, VkThread,
};

use std::{default::Default, ffi::CString, path::Path};

use super::definitions::SpecializationData;
use examples::utils::gltf_importer::{Light, Scene};
use std::mem;

pub struct Deferred {
    pub pipeline_descriptor: Descriptor,
    pub framebuffers: Vec<Framebuffer>,
    pub pipeline: Pipeline,
    pub renderpass: Renderpass,
    pub light_buffer: Buffer,
}

impl Deferred {
    pub fn build(
        images: &Vec<&Image>,
        scene: &Scene,
        swapchain: &Swapchain,
        vulkan: &VkThread,
    ) -> Self {
        //Light buffer stuff

        let light_buffer = vulkan.create_gpu_buffer(
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            &scene
                .get_lights()
                .map_or(vec![Light::default()], |lights| lights.clone()),
        );

        let light_buffer_bindings: Vec<vk::DescriptorBufferInfo> = scene.get_lights().map_or(
            vec![vk::DescriptorBufferInfo {
                buffer: light_buffer.buffer,
                offset: 0,
                range: light_buffer.size,
            }],
            |lights| {
                lights
                    .iter()
                    .enumerate()
                    .map(|(index, _material)| vk::DescriptorBufferInfo {
                        buffer: light_buffer.buffer,
                        offset: (index * mem::size_of::<Light>() as usize) as u64,
                        range: light_buffer.size,
                    })
                    .collect()
            },
        );

        //Create descriptors for the gbuffer images
        let mut descriptors: Vec<DescriptorSet> = images
            .iter()
            .enumerate()
            .map(|(index, image)| DescriptorSet {
                bind_index: index as u32,
                flag: vk::ShaderStageFlags::FRAGMENT,
                bind_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                image_info: Some(vec![vk::DescriptorImageInfo {
                    sampler: image.sampler(),
                    image_view: image.view(),
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                }]),
                ..Default::default()
            })
            .collect();

        //Light descriptor
        descriptors.push(DescriptorSet {
            bind_index: images.len() as u32,
            flag: vk::ShaderStageFlags::FRAGMENT,
            bind_type: vk::DescriptorType::UNIFORM_BUFFER,
            buffer_info: Some(light_buffer_bindings),
            ..Default::default()
        });

        let pipeline_descriptor = Descriptor::new(descriptors, vulkan.context());
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };

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
                .subpasses(&[vk::SubpassDescription::builder()
                    .color_attachments(&[vk::AttachmentReference {
                        attachment: 0,
                        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    }])
                    .build()])
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

        //Shader specialization constants
        let specialization_data = SpecializationData {
            materials_amount: scene.materials.len() as u32,
            textures_amount: scene.textures.len() as u32,
            lights_amount: scene.lights.len() as u32,
        };

        let specialization_info = unsafe {
            vk::SpecializationInfo::builder()
                .map_entries(&specialization_data.specialization_map_entries())
                .data(as_byte_slice(&specialization_data))
                .build()
        };

        let shader_name = CString::new("main").unwrap();
        let mut pipeline = Pipeline::new(vulkan.context());

        pipeline.add_layout(
            vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&[pipeline_descriptor.layout])
                .build(),
        );
        pipeline.add_pipeline(
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
                    .use_specialization(specialization_info)
                    .info(),
                ])
                .vertex_input_state(
                    &vk::PipelineVertexInputStateCreateInfo::builder()
                        .vertex_binding_descriptions(&[])
                        .vertex_attribute_descriptions(&[])
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
                .render_pass(renderpass.pass())
                .layout(pipeline.layout(0))
                .build()
        );
        Self {
            pipeline_descriptor,
            renderpass,
            framebuffers,
            pipeline,
            light_buffer,
        }
    }
}
